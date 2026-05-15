use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use actix_web::{web, HttpRequest, HttpResponse};
use awc::Client;
use tokio::sync::Mutex;

struct RouteConfig {
    upstream: String,
    rate_limit: u32,
    max_body_bytes: usize,
}

fn build_route_table() -> HashMap<&'static str, RouteConfig> {
    let mut m = HashMap::new();
    m.insert(
        "/api/auth",
        RouteConfig {
            upstream: "http://auth-svc:3001".into(),
            rate_limit: 100,
            max_body_bytes: 1_048_576,
        },
    );
    m.insert(
        "/api/core",
        RouteConfig {
            upstream: "http://core-svc:3002".into(),
            rate_limit: 300,
            max_body_bytes: 10_485_760,
        },
    );
    m.insert(
        "/api/judging",
        RouteConfig {
            upstream: "http://judging-svc:3003".into(),
            rate_limit: 100,
            max_body_bytes: 1_048_576,
        },
    );
    m.insert(
        "/api/leaderboard",
        RouteConfig {
            upstream: "http://leaderboard-svc:3004".into(),
            rate_limit: 300,
            max_body_bytes: 1_048_576,
        },
    );
    m.insert(
        "/api/mail",
        RouteConfig {
            upstream: "http://mail-svc:3005".into(),
            rate_limit: 50,
            max_body_bytes: 5_242_880,
        },
    );
    m.insert(
        "/api/notify",
        RouteConfig {
            upstream: "http://notify-svc:3006".into(),
            rate_limit: 50,
            max_body_bytes: 1_048_576,
        },
    );
    m.insert(
        "/api/ai",
        RouteConfig {
            upstream: "http://ai-svc:3007".into(),
            rate_limit: 30,
            max_body_bytes: 1_048_576,
        },
    );
    m.insert(
        "/api/analytics",
        RouteConfig {
            upstream: "http://analytics-svc:3008".into(),
            rate_limit: 100,
            max_body_bytes: 1_048_576,
        },
    );
    m.insert(
        "/api/sponsors",
        RouteConfig {
            upstream: "http://sponsors-svc:3009".into(),
            rate_limit: 50,
            max_body_bytes: 1_048_576,
        },
    );
    m.insert(
        "/api/media",
        RouteConfig {
            upstream: "http://media-svc:3010".into(),
            rate_limit: 50,
            max_body_bytes: 104_857_600,
        },
    );
    m.insert(
        "/api/discord-bot",
        RouteConfig {
            upstream: "http://discord-bot-svc:3011".into(),
            rate_limit: 50,
            max_body_bytes: 1_048_576,
        },
    );
    m
}

/// Per-IP rate limiter tracking request counts within a sliding window.
///
/// # Expected Behavior
///
/// Maintains a map of client IP addresses to (count, `first_seen_timestamp`) pairs.
/// An IP is allowed up to `limit` requests within a 60-second sliding window.
/// If the limit is exceeded, returns false. Counts reset when 60 seconds elapse
/// since the first request in the current window.
///
/// # Errors
///
/// None. Returns a bool indicating whether the request is allowed.
///
/// # Side Effects
///
/// - Acquires the internal Mutex lock.
/// - Mutates the internal `HashMap` (inserts or resets entries).
pub struct RateLimiter {
    state: Arc<Mutex<HashMap<String, (u32, std::time::Instant)>>>,
    window_secs: u64,
}

impl RateLimiter {
    /// Create a new rate limiter with a 60-second window.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            window_secs: 60,
        }
    }

    /// Create a new rate limiter with a custom window duration in seconds.
    ///
    /// # Expected Behavior
    ///
    /// Same as `new()` but with a configurable sliding window. Intended for
    /// testing so that window expiration can be verified without waiting 60
    /// seconds. A `window_secs` of 1 means the counter resets after
    /// approximately 2 seconds (since the comparison is `duration > window`).
    ///
    /// # Errors
    ///
    /// None. Always returns a valid `RateLimiter`.
    ///
    /// # Side Effects
    ///
    /// - Heap-allocates the internal `Arc<Mutex<HashMap>>`.
    #[cfg(test)]
    pub fn new_with_window(window_secs: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            window_secs,
        }
    }

    /// Check if a client IP is allowed to make a request within the given limit.
    pub async fn allow(&self, ip: &str, limit: u32) -> bool {
        let mut state = self.state.lock().await;
        let now = std::time::Instant::now();
        if let Some((count, start)) = state.get_mut(ip) {
            if now.duration_since(*start).as_secs() > self.window_secs {
                *count = 1;
                *start = now;
                true
            } else {
                *count += 1;
                *count <= limit
            }
        } else {
            state.insert(ip.to_string(), (1, now));
            true
        }
    }
}

/// Shared proxy state accessible to all handlers.
///
/// # Expected Behavior
///
/// Holds the route table (prefix -> upstream URL + rate limit + max body size),
/// the HTTP client for forwarding requests, and the rate limiter.
/// The route table can be overridden via the `ROUTES_OVERRIDE` environment variable
/// which is a semicolon-separated list of `prefix=upstream_url` pairs.
pub struct ProxyState {
    routes: HashMap<&'static str, RouteConfig>,
    client: Client,
    pub limiter: RateLimiter,
}

impl ProxyState {
    /// Build proxy state from the default route table with optional env overrides.
    pub fn new() -> Self {
        let mut routes = build_route_table();
        if let Ok(override_str) = std::env::var("ROUTES_OVERRIDE") {
            for pair in override_str.split(';') {
                let parts: Vec<&str> = pair.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let prefix = parts[0].trim();
                    let url = parts[1].trim();
                    if let Some(cfg) = routes.get_mut(prefix) {
                        log::info!("Route override: {prefix} -> {url}");
                        cfg.upstream = url.to_string();
                    }
                }
            }
        }

        let client = Client::builder().timeout(Duration::from_secs(60)).finish();

        Self {
            routes,
            client,
            limiter: RateLimiter::new(),
        }
    }

    /// Extract client IP from the request, preferring X-Forwarded-For.
    pub fn client_ip(req: &HttpRequest) -> String {
        if let Some(xff) = req.headers().get("x-forwarded-for") {
            if let Ok(val) = xff.to_str() {
                if let Some(ip) = val.split(',').next() {
                    return ip.trim().to_string();
                }
            }
        }
        req.peer_addr()
            .map_or_else(|| "unknown".to_string(), |addr| addr.ip().to_string())
    }

    /// Match a request path to the longest registered route prefix.
    ///
    /// # Expected Behavior
    ///
    /// Given a full request path (e.g. `/api/auth/login`), iterates all
    /// registered route prefixes and returns the longest prefix that is
    /// a prefix of the path. Returns `None` if no prefix matches. For
    /// example, `/api/auth/login` matches `/api/auth`; `/api/media/upload`
    /// matches `/api/media`; `/unknown` matches nothing.
    ///
    /// # Errors
    ///
    /// None. Returns `Option<&str>`.
    ///
    /// # Side Effects
    ///
    /// None. Pure lookup with no I/O.
    #[cfg(test)]
    pub fn match_route(&self, path: &str) -> Option<&'static str> {
        self.routes
            .keys()
            .filter(|prefix| path.starts_with(*prefix))
            .max_by_key(|prefix| prefix.len())
            .copied()
    }

    /// Check whether a request body length is within the allowed limit for a route.
    ///
    /// # Expected Behavior
    ///
    /// Looks up the route by `prefix`. If found, returns `true` when
    /// `body_len <= max_body_bytes` (within limit) and `false` when
    /// `body_len > max_body_bytes` (exceeds limit). If no route matches
    /// the prefix, returns `true` (no body-size violation; the request
    /// would instead receive a 404 for an unknown route).
    ///
    /// # Errors
    ///
    /// None. Returns a `bool`.
    ///
    /// # Side Effects
    ///
    /// None. Pure lookup with no I/O.
    #[cfg(test)]
    pub fn is_body_within_limit(&self, prefix: &str, body_len: usize) -> bool {
        self.routes
            .get(prefix)
            .is_none_or(|config| body_len <= config.max_body_bytes)
    }

    /// Forward a request to the given upstream, enforcing rate limit and body size.
    ///
    /// # Expected Behavior
    ///
    /// Checks rate limit for the client IP against `rate_limit`. If exceeded,
    /// returns 429. If `body.len()` exceeds `max_body_bytes`, returns 413.
    /// Otherwise, constructs the upstream URL by prepending `upstream` to the
    /// request path, preserves method/query/headers (stripping hop-by-hop),
    /// adds X-Forwarded-For and X-Forwarded-Proto, and streams the response back.
    ///
    /// # Errors
    ///
    /// Returns 429 on rate limit, 413 on body too large, 502 on upstream failure.
    ///
    /// # Side Effects
    ///
    /// - Acquires the rate limiter Mutex lock.
    /// - Makes an outbound HTTP request to the upstream service (network I/O).
    pub async fn forward(&self, req: HttpRequest, body: web::Bytes, prefix: &str) -> HttpResponse {
        let Some(config) = self.routes.get(prefix) else {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "not_found",
                "message": format!("No route for prefix: {prefix}")
            }));
        };

        let ip = Self::client_ip(&req);
        if !self.limiter.allow(&ip, config.rate_limit).await {
            log::warn!("Rate limit exceeded for {ip} on {prefix}");
            return HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "rate_limit_exceeded",
                "message": format!("Rate limit of {} requests per minute exceeded", config.rate_limit)
            }));
        }

        if body.len() > config.max_body_bytes {
            log::warn!("Body too large for {prefix}: {} bytes", body.len());
            return HttpResponse::PayloadTooLarge().json(serde_json::json!({
                "error": "payload_too_large",
                "message": format!("Request body exceeds maximum of {} bytes", config.max_body_bytes)
            }));
        }

        let upstream_url = format!("{}{}", config.upstream, req.path());
        let query = req.query_string();
        let upstream_url = if query.is_empty() {
            upstream_url
        } else {
            format!("{upstream_url}?{query}")
        };

        let mut fwd = self.client.request(req.method().clone(), &upstream_url);

        for (key, value) in req.headers() {
            let key_lower = key.as_str().to_lowercase();
            if matches!(
                key_lower.as_str(),
                "host"
                    | "connection"
                    | "transfer-encoding"
                    | "upgrade"
                    | "proxy-connection"
                    | "keep-alive"
                    | "te"
                    | "trailers"
            ) {
                continue;
            }
            fwd = fwd.insert_header((key.as_str(), value.clone()));
        }
        fwd = fwd.insert_header(("x-forwarded-for", ip.clone()));
        fwd = fwd.insert_header(("x-forwarded-proto", "http"));

        let resp = fwd.send_body(body).await;

        match resp {
            Ok(upstream_resp) => {
                let status = upstream_resp.status();
                let mut builder = HttpResponse::build(status);
                for (key, value) in upstream_resp.headers() {
                    let key_lower = key.as_str().to_lowercase();
                    if matches!(
                        key_lower.as_str(),
                        "connection" | "transfer-encoding" | "keep-alive" | "upgrade"
                    ) {
                        continue;
                    }
                    builder.insert_header((key.as_str(), value.clone()));
                }
                builder.streaming(upstream_resp)
            }
            Err(e) => {
                log::error!("Upstream error for {}: {e}", req.path());
                HttpResponse::BadGateway().json(serde_json::json!({
                    "error": "bad_gateway",
                    "message": "Upstream service unavailable"
                }))
            }
        }
    }
}

macro_rules! proxy_handler {
    ($name:ident, $prefix:literal) => {
        pub async fn $name(
            state: web::Data<ProxyState>,
            req: HttpRequest,
            body: web::Bytes,
        ) -> HttpResponse {
            state.forward(req, body, $prefix).await
        }
    };
}

proxy_handler!(proxy_auth, "/api/auth");
proxy_handler!(proxy_core, "/api/core");
proxy_handler!(proxy_judging, "/api/judging");
proxy_handler!(proxy_leaderboard, "/api/leaderboard");
proxy_handler!(proxy_mail, "/api/mail");
proxy_handler!(proxy_notify, "/api/notify");
proxy_handler!(proxy_ai, "/api/ai");
proxy_handler!(proxy_analytics, "/api/analytics");
proxy_handler!(proxy_sponsors, "/api/sponsors");
proxy_handler!(proxy_media, "/api/media");
proxy_handler!(proxy_discord_bot, "/api/discord-bot");

/// Health check via proxy — checks auth-svc /health.
pub async fn proxy_health(state: web::Data<ProxyState>) -> HttpResponse {
    let url = "http://auth-svc:3001/health";
    match state.client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let mut builder = HttpResponse::build(status);
            for (key, value) in resp.headers() {
                let key_lower = key.as_str().to_lowercase();
                if matches!(key_lower.as_str(), "connection" | "transfer-encoding") {
                    continue;
                }
                builder.insert_header((key.as_str(), value.clone()));
            }
            builder.streaming(resp)
        }
        Err(e) => {
            log::error!("Health check upstream error: {e}");
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "service_unavailable",
                "message": "Health check failed"
            }))
        }
    }
}

/// Spawn a periodic cleanup task for the rate limiter.
///
/// # Expected Behavior
///
/// Runs a loop that prunes expired rate limiter entries every 60 seconds.
///
/// # Side Effects
///
/// - Spawns a Tokio task that runs every 60 seconds.
/// - Acquires the rate limiter Mutex lock every 60 seconds.
pub fn spawn_limiter_cleanup(limiter: &RateLimiter) {
    let state = limiter.state.clone();
    let window = limiter.window_secs;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let mut map = state.lock().await;
            let now = std::time::Instant::now();
            map.retain(|_, (_, start)| now.duration_since(*start).as_secs() <= window);
        }
    });
}

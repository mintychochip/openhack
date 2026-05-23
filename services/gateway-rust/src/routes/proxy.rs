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
    m.insert(
        "/api/ai/brand-extract",
        RouteConfig {
            upstream: "http://ai-scraper-svc:3012".into(),
            rate_limit: 10,
            max_body_bytes: 65_536,
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

        self.do_forward(req, body, prefix, config, &ip).await
    }

    pub async fn forward_with_redis(
        &self,
        req: HttpRequest,
        body: web::Bytes,
        prefix: &str,
        limiter: &crate::rate_limiter::RedisRateLimiter,
    ) -> HttpResponse {
        let Some(config) = self.routes.get(prefix) else {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "not_found",
                "message": format!("No route for prefix: {prefix}")
            }));
        };

        let ip = Self::client_ip(&req);
        let route_key = prefix.trim_start_matches("/api/");
        if !limiter.allow(&ip, route_key, config.rate_limit).await {
            log::warn!("Rate limit exceeded for {ip} on {prefix}");
            return HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "rate_limit_exceeded",
                "message": format!("Rate limit of {} requests per minute exceeded", config.rate_limit)
            }));
        }

        self.do_forward(req, body, prefix, config, &ip).await
    }

    async fn do_forward(
        &self,
        req: HttpRequest,
        body: web::Bytes,
        prefix: &str,
        config: &RouteConfig,
        ip: &str,
    ) -> HttpResponse {

        if body.len() > config.max_body_bytes {
            log::warn!("Body too large for {prefix}: {} bytes", body.len());
            return HttpResponse::PayloadTooLarge().json(serde_json::json!({
                "error": "payload_too_large",
                "message": format!("Request body exceeds maximum of {} bytes", config.max_body_bytes)
            }));
        }

        let path = req.path();
        let upstream_url = format!("{}{}", config.upstream, path);
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
        fwd = fwd.insert_header(("x-forwarded-for", ip));
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
            limiter: web::Data<crate::rate_limiter::RedisRateLimiter>,
            req: HttpRequest,
            body: web::Bytes,
        ) -> HttpResponse {
            state.forward_with_redis(req, body, $prefix, limiter.get_ref()).await
        }
    };
}

proxy_handler!(auth_handler, "/api/auth");
proxy_handler!(core_handler, "/api/core");
proxy_handler!(judging_handler, "/api/judging");
proxy_handler!(leaderboard_handler, "/api/leaderboard");
proxy_handler!(mail_handler, "/api/mail");
proxy_handler!(notify_handler, "/api/notify");
proxy_handler!(ai_handler, "/api/ai");
proxy_handler!(analytics_handler, "/api/analytics");
proxy_handler!(sponsors_handler, "/api/sponsors");
proxy_handler!(media_handler, "/api/media");
proxy_handler!(discord_bot_handler, "/api/discord-bot");
proxy_handler!(brand_extract_handler, "/api/ai/brand-extract");

/// Composite health check — checks all downstream services.
pub async fn proxy_health(state: web::Data<ProxyState>) -> HttpResponse {
    let services = [
        ("auth", "http://auth-svc:3001/health"),
        ("core", "http://core-svc:3002/health"),
        ("judging", "http://judging-svc:3003/health"),
        ("leaderboard", "http://leaderboard-svc:3004/health"),
        ("mail", "http://mail-svc:3005/health"),
        ("notify", "http://notify-svc:3006/health"),
        ("ai", "http://ai-svc:3007/health"),
        ("analytics", "http://analytics-svc:3008/health"),
        ("sponsors", "http://sponsors-svc:3009/health"),
        ("media", "http://media-svc:3010/health"),
        ("discord_bot", "http://discord-bot-svc:3011/health"),
        ("ai_scraper", "http://ai-scraper-svc:3012/health"),
    ];

    let mut results = serde_json::Map::new();
    let mut all_healthy = true;

    for (name, url) in &services {
        match state.client.get(*url).timeout(Duration::from_secs(5)).send().await {
            Ok(resp) if resp.status().is_success() => {
                results.insert((*name).to_string(), serde_json::Value::String("healthy".to_string()));
            }
            Ok(resp) => {
                results.insert((*name).to_string(), serde_json::json!({ "status": "unhealthy", "code": resp.status().as_u16() }));
                all_healthy = false;
            }
            Err(e) => {
                log::warn!("Health check failed for {name}: {e}");
                results.insert((*name).to_string(), serde_json::Value::String("unreachable".to_string()));
                all_healthy = false;
            }
        }
    }

    if all_healthy {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "service": "gateway",
            "downstream": results
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "degraded",
            "service": "gateway",
            "downstream": results
        }))
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

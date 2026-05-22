use redis::aio::MultiplexedConnection;
use redis::Client;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Redis-backed rate limiter using sliding window counter.
///
/// # Expected Behavior
///
/// Uses Redis INCR + EXPIRE to track request counts per IP per route.
/// Key format: `ratelimit:gateway:{route}:{ip}`
/// Window: 60 seconds (configurable)
///
/// When a request comes in:
/// 1. Get current Unix timestamp second
/// 2. INCR the key for this IP+route
/// 3. If this is the first request (count=1), set EXPIRE to window_secs
/// 4. If count > limit, deny the request
///
/// # Side Effects
///
/// - Redis network calls (INCR, EXPIRE)
/// - No local state mutation
pub struct RedisRateLimiter {
    redis_client: Arc<Option<Client>>,
    connection: Arc<Mutex<Option<MultiplexedConnection>>>,
    window_secs: u64,
}

impl RedisRateLimiter {
    pub fn new_with_url(redis_url: Option<String>) -> Self {
        let redis_client = redis_url
            .as_ref()
            .and_then(|url| Client::open(url.as_str()).ok());

        Self {
            redis_client: Arc::new(redis_client),
            connection: Arc::new(Mutex::new(None)),
            window_secs: 60,
        }
    }

    pub fn with_window(redis_url: Option<String>, window_secs: u64) -> Self {
        let redis_client = redis_url
            .as_ref()
            .and_then(|url| Client::open(url.as_str()).ok());

        Self {
            redis_client: Arc::new(redis_client),
            connection: Arc::new(Mutex::new(None)),
            window_secs,
        }
    }

    async fn get_connection(&self) -> Option<MultiplexedConnection> {
        let client = (*self.redis_client).as_ref()?;

        let mut conn_guard = self.connection.lock().await;

        if let Some(cached) = conn_guard.as_ref() {
            return Some(cached.clone());
        }

        match client.get_multiplexed_tokio_connection().await {
            Ok(conn) => {
                *conn_guard = Some(conn.clone());
                Some(conn)
            }
            Err(e) => {
                log::warn!("Failed to get Redis connection: {e}");
                None
            }
        }
    }

    /// Check if a request is allowed for the given IP and route.
    ///
    /// # Expected Behavior
    ///
    /// Returns `true` if the request is within the rate limit.
    /// Returns `false` if the limit is exceeded.
    /// If Redis is unavailable, returns `true` (fail-open).
    ///
    /// # Side Effects
    ///
    /// - Redis INCR and EXPIRE commands if Redis is available
    pub async fn allow(&self, ip: &str, route: &str, limit: u32) -> bool {
        let Some(mut conn) = self.get_connection().await else {
            log::warn!("Redis not available for rate limiting, allowing request");
            return true;
        };

        let key = format!("ratelimit:gateway:{}:{}", route, ip);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window_start = now / self.window_secs;
        let windowed_key = format!("{}:{}", key, window_start);

        match redis::cmd("INCR")
            .arg(&windowed_key)
            .query_async::<_, i64>(&mut conn)
            .await
        {
            Ok(count) => {
                if count == 1 {
                    let _: () = redis::cmd("EXPIRE")
                        .arg(&windowed_key)
                        .arg(self.window_secs as i64)
                        .query_async::<_, ()>(&mut conn)
                        .await
                        .unwrap_or(());
                }

                count <= limit as i64
            }
            Err(e) => {
                log::warn!("Redis rate limit check failed: {e}, allowing request");
                true
            }
        }
    }
}

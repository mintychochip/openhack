use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

use crate::errors::AuthError;

/// Check if a request is rate-limited for a given route and IP.
///
/// # Expected Behavior
///
/// If a Redis connection is provided, increments the Redis counter at key
/// `ratelimit:auth:{route}:{ip}`. If the counter is 1 (first request in the
/// window), sets the TTL to `window_secs`. If the counter exceeds `max_requests`,
/// returns `AuthError::RateLimited`. If Redis is None (connection unavailable),
/// rate limiting is skipped and the request is allowed through — the service
/// degrades gracefully without Redis.
///
/// # Errors
///
/// Returns `AuthError::RateLimited` if the request count exceeds the
/// maximum. Returns `AuthError::RedisError` on Redis connection or
/// command failures.
///
/// # Side Effects
///
/// - Increments Redis key `ratelimit:auth:{route}:{ip}` (write).
/// - Sets TTL on the key if it is newly created (write).
pub async fn check_rate_limit(
    conn: Option<&MultiplexedConnection>,
    route: &str,
    ip: &str,
    max_requests: i64,
    window_secs: i64,
) -> Result<(), AuthError> {
    let Some(conn) = conn else {
        return Ok(());
    };

    let mut c = conn.clone();

    let key = format!("ratelimit:auth:{route}:{ip}");
    let count: i64 = c.incr(&key, 1).await.map_err(AuthError::RedisError)?;

    if count == 1 {
        let _: () = c
            .expire(&key, window_secs)
            .await
            .map_err(AuthError::RedisError)?;
    }

    if count > max_requests {
        return Err(AuthError::RateLimited);
    }

    Ok(())
}

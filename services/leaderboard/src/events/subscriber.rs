use redis::AsyncCommands;
use sqlx::PgPool;

/// Redis subscriber that listens for recalculation triggers using a polling
/// approach on a Redis list acting as a message queue.
///
/// # Expected Behavior
///
/// Connects to Redis using the `REDIS_URL` environment variable, then
/// enters a loop calling BLPOP on the `leaderboard:recalculate` list
/// with a 5-second timeout. When a message is received, invokes
/// `RankingService::recalculate_rankings` with the active formula.
/// On each message, acquires a new Redis multiplexed connection for
/// cache invalidation during recalculation. Continues listening until
/// a fatal error occurs. If `REDIS_URL` is not set or the Redis
/// connection cannot be established, logs a warning and returns
/// immediately (no subscriber task runs).
///
/// # Errors
///
/// Returns early (logs warning) if `REDIS_URL` is not set, if the
/// Redis client cannot be created, or the connection cannot be
/// established. Individual message processing errors are logged and
/// the subscriber continues listening.
///
/// # Side Effects
///
/// - Establishes a Redis connection (network connection).
/// - Calls BLPOP on `leaderboard:recalculate` list (Redis read with blocking).
/// - On each message: acquires a new Redis multiplexed connection,
///   recalculates rankings (database reads/writes, Redis cache
///   invalidation), and logs results.
/// - Logs at INFO on startup and on each message received.
/// - Logs at WARN if `REDIS_URL` is not set or connection fails.
/// - Logs at ERROR on BLPOP errors and recalculation errors.
pub async fn run_subscriber(pool: PgPool) {
    let Some(redis_url) = std::env::var("REDIS_URL").ok() else {
        log::warn!("REDIS_URL not set, skipping Redis subscriber");
        return;
    };

    let client = match redis::Client::open(redis_url) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to create Redis client for subscriber: {e}");
            return;
        }
    };

    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to create Redis connection for subscriber: {e}");
            return;
        }
    };

    log::info!("Redis subscriber started, listening on leaderboard:recalculate queue");

    loop {
        let result: Option<(String, String)> =
            match conn.blpop("leaderboard:recalculate", 5.0).await {
                Ok(r) => r,
                Err(e) => {
                    log::error!("BLPOP error: {e}");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

        if let Some((_, payload)) = result {
            log::info!("Received recalculate trigger: {payload}");

            let redis_conn = match client.get_multiplexed_async_connection().await {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to get Redis connection for recalculation: {e}");
                    continue;
                }
            };

            if let Err(e) = crate::services::ranking::RankingService::recalculate_rankings(
                &pool,
                Some(&redis_conn),
                None,
            )
            .await
            {
                log::error!("Failed to recalculate rankings from subscriber: {e}");
            }
        }
    }
}

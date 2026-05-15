use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::models::booth::Booth;

/// Get a cached booth from Redis by ID.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `sponsor:booth:{id}`. Returns
/// `Ok(Some(Booth))` if the key exists and can be deserialized. Returns
/// `Ok(None)` if the key does not exist, has expired, or if the Redis
/// connection is not available (`conn` is `None`). The cached value
/// is a JSON-serialized `Booth` struct.
///
/// # Errors
///
/// Returns `redis::RedisError` on any Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `sponsor:booth:{id}` (read-only).
pub async fn get_booth_from_cache(
    conn: Option<MultiplexedConnection>,
    booth_id: &Uuid,
) -> Result<Option<Booth>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let key = format!("sponsor:booth:{booth_id}");
    let val: Option<String> = conn.get(&key).await?;
    match val {
        Some(json) => {
            let booth: Booth = serde_json::from_str(&json).unwrap_or_else(|_| {
                log::warn!("Failed to deserialize cached booth {booth_id}");
                Booth {
                    id: *booth_id,
                    sponsor_id: None,
                    sponsor_name: String::new(),
                    tagline: None,
                    description: None,
                    logo_url: None,
                    banner_url: None,
                    website_url: None,
                    careers_url: None,
                    api_docs_url: None,
                    technologies: None,
                    contact_email: None,
                    discord_channel: None,
                    theme_colors: None,
                    published: None,
                    view_count: None,
                    created_at: None,
                    updated_at: None,
                }
            });
            Ok(Some(booth))
        }
        None => Ok(None),
    }
}

/// Cache a booth in Redis with a 5-minute TTL.
///
/// # Expected Behavior
///
/// Serializes the booth as JSON and writes it to the Redis STRING key
/// `sponsor:booth:{id}` with a 300-second (5-minute) TTL. Overwrites
/// any existing value. If `conn` is `None`, returns `Ok(())` immediately
/// (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `sponsor:booth:{id}` (write).
/// - Sets 300-second TTL on the key (write).
pub async fn set_booth_cache(
    conn: Option<MultiplexedConnection>,
    booth: &Booth,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let key = format!("sponsor:booth:{}", booth.id);
    let json = serde_json::to_string(booth).unwrap_or_default();
    conn.set_ex::<_, _, ()>(&key, json, 300).await?;
    Ok(())
}

/// Invalidate a single booth cache entry.
///
/// # Expected Behavior
///
/// Deletes the Redis key `sponsor:booth:{id}`. No-op if the key does not
/// exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `sponsor:booth:{id}` (write).
pub async fn invalidate_booth_cache(
    conn: Option<MultiplexedConnection>,
    booth_id: &Uuid,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let key = format!("sponsor:booth:{booth_id}");
    let _: () = conn.del(&key).await?;
    Ok(())
}

/// Get the cached list of published booths from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `sponsor:booths:published`. Returns
/// `Ok(Some(String))` if the key exists (the value is a JSON string
/// of a `BoothListResponse`). Returns `Ok(None)` if the key does not
/// exist, has expired, or if the Redis connection is not available
/// (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `sponsor:booths:published` (read-only).
pub async fn get_published_booths_from_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let val: Option<String> = conn.get("sponsor:booths:published").await?;
    Ok(val)
}

/// Cache the list of published booths in Redis with a 1-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `sponsor:booths:published` with a 60-second (1-minute) TTL.
/// Overwrites any existing value. If `conn` is `None`, returns
/// `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `sponsor:booths:published` (write).
/// - Sets 60-second TTL on the key (write).
pub async fn set_published_booths_cache(
    conn: Option<MultiplexedConnection>,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    conn.set_ex::<_, _, ()>("sponsor:booths:published", json, 60)
        .await?;
    Ok(())
}

/// Invalidate the published booths list cache.
///
/// # Expected Behavior
///
/// Deletes the Redis key `sponsor:booths:published`. No-op if the key
/// does not exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `sponsor:booths:published` (write).
pub async fn invalidate_published_booths_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let _: () = conn.del("sponsor:booths:published").await?;
    Ok(())
}

/// Get the cached list of prizes for a booth from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `sponsor:booth:{boothId}:prizes`. Returns
/// `Ok(Some(String))` if the key exists (the value is a JSON string
/// of a `PrizeListResponse`). Returns `Ok(None)` if the key does not
/// exist, has expired, or if the Redis connection is not available
/// (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `sponsor:booth:{boothId}:prizes` (read-only).
pub async fn get_prizes_from_cache(
    conn: Option<MultiplexedConnection>,
    booth_id: &Uuid,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let key = format!("sponsor:booth:{booth_id}:prizes");
    let val: Option<String> = conn.get(&key).await?;
    Ok(val)
}

/// Cache the list of prizes for a booth in Redis with a 5-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `sponsor:booth:{boothId}:prizes` with a 300-second (5-minute) TTL.
/// Overwrites any existing value. If `conn` is `None`, returns
/// `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `sponsor:booth:{boothId}:prizes` (write).
/// - Sets 300-second TTL on the key (write).
pub async fn set_prizes_cache(
    conn: Option<MultiplexedConnection>,
    booth_id: &Uuid,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let key = format!("sponsor:booth:{booth_id}:prizes");
    conn.set_ex::<_, _, ()>(&key, json, 300).await?;
    Ok(())
}

/// Invalidate the prizes cache for a specific booth.
///
/// # Expected Behavior
///
/// Deletes the Redis key `sponsor:booth:{boothId}:prizes`. No-op if the
/// key does not exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `sponsor:booth:{boothId}:prizes` (write).
pub async fn invalidate_prizes_cache(
    conn: Option<MultiplexedConnection>,
    booth_id: &Uuid,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let key = format!("sponsor:booth:{booth_id}:prizes");
    let _: () = conn.del(&key).await?;
    Ok(())
}

/// Invalidate all cache entries related to a booth.
///
/// # Expected Behavior
///
/// Invalidates the single booth cache, the published booths list cache,
/// and the prizes cache for the given booth ID. Catches and logs Redis
/// errors without propagating them, so cache invalidation failure does
/// not block the primary operation. If `conn` is `None`, returns
/// immediately (all invalidation skipped).
///
/// # Errors
///
/// None. Redis errors are logged and swallowed.
///
/// # Side Effects
///
/// - Deletes Redis keys `sponsor:booth:{id}`, `sponsor:booths:published`,
///   and `sponsor:booth:{id}:prizes` (writes).
/// - Logs at WARN level on Redis errors.
pub async fn invalidate_booth_caches(conn: Option<MultiplexedConnection>, booth_id: &Uuid) {
    let Some(conn) = conn else { return };
    if let Err(e) = invalidate_booth_cache(Some(conn.clone()), booth_id).await {
        log::warn!("Failed to invalidate booth cache: {e}");
    }
    if let Err(e) = invalidate_published_booths_cache(Some(conn.clone())).await {
        log::warn!("Failed to invalidate published booths cache: {e}");
    }
    if let Err(e) = invalidate_prizes_cache(Some(conn.clone()), booth_id).await {
        log::warn!("Failed to invalidate prizes cache: {e}");
    }
}

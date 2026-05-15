use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use uuid::Uuid;

/// Get the cached leaderboard from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `leaderboard:current`. Returns
/// `Ok(Some(String))` if the key exists (the value is a JSON string
/// of a `LeaderboardResponse`). Returns `Ok(None)` if the key does
/// not exist, has expired, or if the Redis connection is not available
/// (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `leaderboard:current` (read-only).
pub async fn get_leaderboard_from_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let val: Option<String> = conn.get("leaderboard:current").await?;
    Ok(val)
}

/// Cache the leaderboard in Redis with a 1-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `leaderboard:current` with a 60-second (1-minute) TTL.
/// Overwrites any existing value. If `conn` is `None`, returns
/// `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `leaderboard:current` (write).
/// - Sets 60-second TTL on the key (write).
pub async fn set_leaderboard_cache(
    conn: Option<MultiplexedConnection>,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    conn.set_ex::<_, _, ()>("leaderboard:current", json, 60)
        .await?;
    Ok(())
}

/// Invalidate the leaderboard cache.
///
/// # Expected Behavior
///
/// Deletes the Redis key `leaderboard:current`. No-op if the key
/// does not exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `leaderboard:current` (write).
pub async fn invalidate_leaderboard_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let _: () = conn.del("leaderboard:current").await?;
    Ok(())
}

/// Get the cached score history for a team from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `leaderboard:history:{team_id}`. Returns
/// `Ok(Some(String))` if the key exists. Returns `Ok(None)` if the
/// key does not exist, has expired, or if the Redis connection is not
/// available (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `leaderboard:history:{team_id}` (read-only).
pub async fn get_score_history_from_cache(
    conn: Option<MultiplexedConnection>,
    team_id: &Uuid,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let key = format!("leaderboard:history:{team_id}");
    let val: Option<String> = conn.get(&key).await?;
    Ok(val)
}

/// Cache score history for a team in Redis with a 2-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `leaderboard:history:{team_id}` with a 120-second (2-minute) TTL.
/// If `conn` is `None`, returns `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `leaderboard:history:{team_id}` (write).
/// - Sets 120-second TTL on the key (write).
pub async fn set_score_history_cache(
    conn: Option<MultiplexedConnection>,
    team_id: &Uuid,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let key = format!("leaderboard:history:{team_id}");
    conn.set_ex::<_, _, ()>(&key, json, 120).await?;
    Ok(())
}

/// Invalidate score history cache for a team.
///
/// # Expected Behavior
///
/// Deletes the Redis key `leaderboard:history:{team_id}`. No-op if
/// the key does not exist. If `conn` is `None`, returns `Ok(())`
/// immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `leaderboard:history:{team_id}` (write).
pub async fn _invalidate_score_history_cache(
    conn: Option<MultiplexedConnection>,
    team_id: &Uuid,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let key = format!("leaderboard:history:{team_id}");
    let _: () = conn.del(&key).await?;
    Ok(())
}

/// Get the cached vote count for a project from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `leaderboard:votes:{project_id}`. Returns
/// `Ok(Some(String))` if the key exists. Returns `Ok(None)` if the
/// key does not exist, has expired, or if the Redis connection is not
/// available (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `leaderboard:votes:{project_id}` (read-only).
pub async fn get_vote_count_from_cache(
    conn: Option<MultiplexedConnection>,
    project_id: &Uuid,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let key = format!("leaderboard:votes:{project_id}");
    let val: Option<String> = conn.get(&key).await?;
    Ok(val)
}

/// Cache vote count for a project in Redis with a 30-second TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `leaderboard:votes:{project_id}` with a 30-second TTL.
/// If `conn` is `None`, returns `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `leaderboard:votes:{project_id}` (write).
/// - Sets 30-second TTL on the key (write).
pub async fn set_vote_count_cache(
    conn: Option<MultiplexedConnection>,
    project_id: &Uuid,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let key = format!("leaderboard:votes:{project_id}");
    conn.set_ex::<_, _, ()>(&key, json, 30).await?;
    Ok(())
}

/// Get the cached formulas list from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `leaderboard:formulas`. Returns
/// `Ok(Some(String))` if the key exists. Returns `Ok(None)` if
/// the key does not exist, has expired, or if the Redis connection
/// is not available (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `leaderboard:formulas` (read-only).
pub async fn get_formulas_from_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let val: Option<String> = conn.get("leaderboard:formulas").await?;
    Ok(val)
}

/// Cache formulas list in Redis with a 5-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `leaderboard:formulas` with a 300-second (5-minute) TTL.
/// If `conn` is `None`, returns `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `leaderboard:formulas` (write).
/// - Sets 300-second TTL on the key (write).
pub async fn set_formulas_cache(
    conn: Option<MultiplexedConnection>,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    conn.set_ex::<_, _, ()>("leaderboard:formulas", json, 300)
        .await?;
    Ok(())
}

/// Invalidate the formulas list cache.
///
/// # Expected Behavior
///
/// Deletes the Redis key `leaderboard:formulas`. No-op if the key
/// does not exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `leaderboard:formulas` (write).
pub async fn invalidate_formulas_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let _: () = conn.del("leaderboard:formulas").await?;
    Ok(())
}

/// Get the cached active formula from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `leaderboard:formulas:active`. Returns
/// `Ok(Some(String))` if the key exists. Returns `Ok(None)` if the
/// key does not exist, has expired, or if the Redis connection is not
/// available (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `leaderboard:formulas:active` (read-only).
pub async fn get_active_formula_from_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let val: Option<String> = conn.get("leaderboard:formulas:active").await?;
    Ok(val)
}

/// Cache the active formula in Redis with a 5-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `leaderboard:formulas:active` with a 300-second (5-minute) TTL.
/// If `conn` is `None`, returns `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `leaderboard:formulas:active` (write).
/// - Sets 300-second TTL on the key (write).
pub async fn set_active_formula_cache(
    conn: Option<MultiplexedConnection>,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    conn.set_ex::<_, _, ()>("leaderboard:formulas:active", json, 300)
        .await?;
    Ok(())
}

/// Invalidate the active formula cache.
///
/// # Expected Behavior
///
/// Deletes the Redis key `leaderboard:formulas:active`. No-op if the
/// key does not exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `leaderboard:formulas:active` (write).
pub async fn invalidate_active_formula_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let _: () = conn.del("leaderboard:formulas:active").await?;
    Ok(())
}

/// Get the cached voting config from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `leaderboard:voting_config`. Returns
/// `Ok(Some(String))` if the key exists. Returns `Ok(None)` if the
/// key does not exist, has expired, or if the Redis connection is not
/// available (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `leaderboard:voting_config` (read-only).
pub async fn get_voting_config_from_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let val: Option<String> = conn.get("leaderboard:voting_config").await?;
    Ok(val)
}

/// Cache voting config in Redis with a 5-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `leaderboard:voting_config` with a 300-second (5-minute) TTL.
/// If `conn` is `None`, returns `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `leaderboard:voting_config` (write).
/// - Sets 300-second TTL on the key (write).
pub async fn set_voting_config_cache(
    conn: Option<MultiplexedConnection>,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    conn.set_ex::<_, _, ()>("leaderboard:voting_config", json, 300)
        .await?;
    Ok(())
}

/// Invalidate the voting config cache.
///
/// # Expected Behavior
///
/// Deletes the Redis key `leaderboard:voting_config`. No-op if the
/// key does not exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `leaderboard:voting_config` (write).
pub async fn invalidate_voting_config_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let _: () = conn.del("leaderboard:voting_config").await?;
    Ok(())
}

/// Get the cached snapshots list from Redis.
///
/// # Expected Behavior
///
/// Reads the Redis STRING key `leaderboard:snapshots`. Returns
/// `Ok(Some(String))` if the key exists. Returns `Ok(None)` if the
/// key does not exist, has expired, or if the Redis connection is not
/// available (`conn` is `None`).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Reads from Redis key `leaderboard:snapshots` (read-only).
pub async fn get_snapshots_from_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<Option<String>, redis::RedisError> {
    let Some(mut conn) = conn else {
        return Ok(None);
    };
    let val: Option<String> = conn.get("leaderboard:snapshots").await?;
    Ok(val)
}

/// Cache snapshots list in Redis with a 5-minute TTL.
///
/// # Expected Behavior
///
/// Writes the given JSON string to the Redis STRING key
/// `leaderboard:snapshots` with a 300-second (5-minute) TTL.
/// If `conn` is `None`, returns `Ok(())` immediately (cache write skipped).
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Writes to Redis key `leaderboard:snapshots` (write).
/// - Sets 300-second TTL on the key (write).
pub async fn set_snapshots_cache(
    conn: Option<MultiplexedConnection>,
    json: &str,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    conn.set_ex::<_, _, ()>("leaderboard:snapshots", json, 300)
        .await?;
    Ok(())
}

/// Invalidate the snapshots cache.
///
/// # Expected Behavior
///
/// Deletes the Redis key `leaderboard:snapshots`. No-op if the key
/// does not exist. If `conn` is `None`, returns `Ok(())` immediately.
///
/// # Errors
///
/// Returns `redis::RedisError` on Redis command failure.
///
/// # Side Effects
///
/// - Deletes Redis key `leaderboard:snapshots` (write).
pub async fn invalidate_snapshots_cache(
    conn: Option<MultiplexedConnection>,
) -> Result<(), redis::RedisError> {
    let Some(mut conn) = conn else { return Ok(()) };
    let _: () = conn.del("leaderboard:snapshots").await?;
    Ok(())
}

/// Invalidate all cache entries related to the leaderboard.
///
/// # Expected Behavior
///
/// Invalidates the leaderboard, formulas, active formula, voting config,
/// and snapshots caches. Catches and logs Redis errors without propagating
/// them, so cache invalidation failure does not block the primary operation.
/// If `conn` is `None`, returns immediately (all invalidation skipped).
///
/// # Errors
///
/// None. Redis errors are logged and swallowed.
///
/// # Side Effects
///
/// - Deletes Redis keys `leaderboard:current`, `leaderboard:formulas`,
///   `leaderboard:formulas:active`, `leaderboard:voting_config`, and
///   `leaderboard:snapshots` (writes).
/// - Logs at WARN level on Redis errors.
pub async fn invalidate_all_leaderboard_caches(conn: Option<MultiplexedConnection>) {
    let Some(conn) = conn else { return };
    if let Err(e) = invalidate_leaderboard_cache(Some(conn.clone())).await {
        log::warn!("Failed to invalidate leaderboard cache: {e}");
    }
    if let Err(e) = invalidate_formulas_cache(Some(conn.clone())).await {
        log::warn!("Failed to invalidate formulas cache: {e}");
    }
    if let Err(e) = invalidate_active_formula_cache(Some(conn.clone())).await {
        log::warn!("Failed to invalidate active formula cache: {e}");
    }
    if let Err(e) = invalidate_voting_config_cache(Some(conn.clone())).await {
        log::warn!("Failed to invalidate voting config cache: {e}");
    }
    if let Err(e) = invalidate_snapshots_cache(Some(conn.clone())).await {
        log::warn!("Failed to invalidate snapshots cache: {e}");
    }
}

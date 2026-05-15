use chrono::Utc;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AuthError;
use crate::models::session::Session;

/// Compute the SHA-256 hex digest of a string.
///
/// # Expected Behavior
///
/// Takes a string input, computes its SHA-256 hash, and returns
/// the hex-encoded digest string. Used for hashing refresh tokens
/// before storage in the database.
///
/// # Errors
///
/// None. SHA-256 computation is infallible.
///
/// # Side Effects
///
/// None. Pure computation.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Create a new session for a user.
///
/// # Expected Behavior
///
/// Generates a new UUID refresh token, hashes it with SHA-256, and inserts
/// a new row into `auth.sessions`. If a Redis connection is provided, also caches
/// the session in Redis with key `session:{id}` containing JSON `{userId, createdAt}`
/// with a TTL matching `refresh_token_expiry_secs`. If Redis is None, the cache
/// write is skipped — the DB is the source of truth. Returns the plaintext refresh
/// token (UUID string) for the client.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database insert failures.
/// Redis cache write failures are silently ignored.
///
/// # Side Effects
///
/// - Writes to `auth.sessions` table (insert new session).
/// - Writes to Redis key `session:{id}` (set with TTL), if Redis is available.
pub async fn create_session(
    pool: &PgPool,
    conn: Option<&MultiplexedConnection>,
    user_id: Uuid,
    device_info: Option<serde_json::Value>,
    ip_address: Option<String>,
    config: &Config,
) -> Result<String, AuthError> {
    let session_id = Uuid::new_v4();
    let refresh_token = Uuid::new_v4().to_string();
    let refresh_token_hash = sha256_hex(&refresh_token);

    let expires_at =
        Utc::now().naive_utc() + chrono::Duration::seconds(config.refresh_token_expiry_secs);

    sqlx::query(
        "INSERT INTO auth.sessions (id, user_id, refresh_token_hash, device_info, ip_address, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW())",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(&refresh_token_hash)
    .bind(&device_info)
    .bind(&ip_address)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(AuthError::DatabaseError)?;

    if let Some(conn) = conn {
        let cache_data = serde_json::json!({
            "userId": user_id.to_string(),
            "createdAt": Utc::now().to_rfc3339(),
        });

        let mut c = conn.clone();
        let cache_key = format!("session:{session_id}");
        let _: Result<(), _> = c
            .set_ex(
                &cache_key,
                cache_data.to_string(),
                config.refresh_token_expiry_secs.cast_unsigned(),
            )
            .await;
    }

    Ok(refresh_token)
}

/// Validate a refresh token and return the associated session.
///
/// # Expected Behavior
///
/// Hashes the provided refresh token with SHA-256, looks up the session
/// in `auth.sessions` by `refresh_token_hash`. Checks that the session
/// is not revoked (`revoked_at` IS NULL) and not expired (`expires_at` > `NOW()`).
/// Returns the Session record if valid.
///
/// # Errors
///
/// Returns `AuthError::Unauthorized` if the token is invalid, revoked,
/// or expired. Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Reads from `auth.sessions` table.
pub async fn validate_refresh_token(
    pool: &PgPool,
    refresh_token: &str,
) -> Result<Session, AuthError> {
    let token_hash = sha256_hex(refresh_token);

    let session = sqlx::query_as::<_, Session>(
        "SELECT id, user_id, refresh_token_hash, device_info, ip_address, expires_at, revoked_at, created_at FROM auth.sessions WHERE refresh_token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(AuthError::DatabaseError)?
    .ok_or_else(|| AuthError::Unauthorized("Invalid or revoked refresh token".to_string()))?;

    if let Some(expires_at) = session.expires_at {
        if expires_at < Utc::now().naive_utc() {
            return Err(AuthError::Unauthorized("Refresh token expired".to_string()));
        }
    }

    Ok(session)
}

/// Rotate a refresh token: revoke the old session and create a new one.
///
/// # Expected Behavior
///
/// Validates the old refresh token, revokes the old session (sets `revoked_at`
/// to `NOW()`), then creates a new session with a new refresh token. The new
/// session inherits the same `user_id`, `device_info`, and `ip_address`. If
/// Redis is available, deletes the old cache entry and creates a new one. If
/// Redis is None, cache operations are skipped — the DB is the source of truth.
/// Returns the new plaintext refresh token.
///
/// # Errors
///
/// Returns `AuthError::Unauthorized` if the old token is invalid.
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Writes to `auth.sessions` table (update `revoked_at`, insert new row).
/// - Deletes and writes to Redis key `session:{id}`, if Redis is available.
pub async fn rotate_refresh_token(
    pool: &PgPool,
    conn: Option<&MultiplexedConnection>,
    old_refresh_token: &str,
    config: &Config,
) -> Result<(String, Uuid), AuthError> {
    let old_session = validate_refresh_token(pool, old_refresh_token).await?;

    sqlx::query("UPDATE auth.sessions SET revoked_at = NOW() WHERE id = $1")
        .bind(old_session.id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

    if let Some(c) = conn {
        let mut redis = c.clone();
        let old_cache_key = format!("session:{}", old_session.id);
        let _: Result<(), _> = redis.del(&old_cache_key).await;
    }

    let new_refresh_token = create_session(
        pool,
        conn,
        old_session.user_id,
        old_session.device_info.clone(),
        old_session.ip_address.clone(),
        config,
    )
    .await?;

    Ok((new_refresh_token, old_session.user_id))
}

/// Revoke a specific session by ID.
///
/// # Expected Behavior
///
/// Sets `revoked_at` to `NOW()` for the session with the given ID.
/// If Redis is available, also deletes the Redis cache entry for the session.
/// If Redis is None, the cache delete is skipped — the DB is the source of truth.
/// No-op if the session does not exist or is already revoked.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Writes to `auth.sessions` table (set `revoked_at`).
/// - Deletes Redis key `session:{id}`, if Redis is available.
#[allow(dead_code)]
pub async fn revoke_session(
    pool: &PgPool,
    conn: Option<&MultiplexedConnection>,
    session_id: Uuid,
) -> Result<(), AuthError> {
    sqlx::query("UPDATE auth.sessions SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

    if let Some(c) = conn {
        let mut redis = c.clone();
        let cache_key = format!("session:{session_id}");
        let _: Result<(), _> = redis.del(&cache_key).await;
    }

    Ok(())
}

/// Revoke all sessions for a user.
///
/// # Expected Behavior
///
/// Sets `revoked_at` to `NOW()` for all non-revoked sessions belonging to
/// the given `user_id`. Returns the number of sessions revoked. If Redis is
/// available, deletes all corresponding Redis cache entries. If Redis is None,
/// cache deletes are skipped — the DB is the source of truth.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
/// Redis errors are logged but not propagated.
///
/// # Side Effects
///
/// - Writes to `auth.sessions` table (bulk update `revoked_at`).
/// - Deletes multiple Redis keys `session:*` (best-effort), if Redis is available.
pub async fn revoke_all_user_sessions(
    pool: &PgPool,
    conn: Option<&MultiplexedConnection>,
    user_id: Uuid,
) -> Result<i64, AuthError> {
    let result = sqlx::query(
        "UPDATE auth.sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AuthError::DatabaseError)?;

    let sessions: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM auth.sessions WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    if let Some(c) = conn {
        let mut redis = c.clone();
        for sid in sessions {
            let cache_key = format!("session:{sid}");
            let _: Result<(), _> = redis.del(&cache_key).await;
        }
    }

    Ok(result.rows_affected().cast_signed())
}

/// Store an MFA challenge in Redis for the login flow.
///
/// # Expected Behavior
///
/// Stores a JSON object `{userId, email, provider, type: "mfa_challenge"}`
/// in Redis with key `mfa:challenge:{token}` and a 5-minute TTL (300s).
/// This allows the subsequent login request with MFA code to look up
/// the pending challenge without re-verifying the password. If Redis is
/// None, returns `AuthError::BadRequest` since MFA challenges require Redis.
///
/// # Errors
///
/// Returns `AuthError::BadRequest` if Redis is unavailable.
/// Returns `AuthError::RedisError` on Redis write failures.
///
/// # Side Effects
///
/// - Writes to Redis key `mfa:challenge:{token}` with 300s TTL.
pub async fn store_mfa_challenge(
    conn: Option<&MultiplexedConnection>,
    mfa_token: &str,
    user_id: Uuid,
    email: &str,
) -> Result<(), AuthError> {
    let Some(conn) = conn else {
        return Err(AuthError::BadRequest(
            "MFA requires Redis, which is unavailable".to_string(),
        ));
    };
    let mut c = conn.clone();

    let data = serde_json::json!({
        "userId": user_id.to_string(),
        "email": email,
    });

    let key = format!("mfa:challenge:{mfa_token}");
    c.set_ex::<_, _, ()>(&key, data.to_string(), 300)
        .await
        .map_err(AuthError::RedisError)?;

    Ok(())
}

/// Retrieve and delete an MFA challenge from Redis.
///
/// # Expected Behavior
///
/// Looks up the MFA challenge by token in Redis. If found, parses the
/// JSON to extract the userId, then deletes the key (single-use).
/// Returns the `user_id` if the challenge exists and is valid.
/// If Redis is None, returns `AuthError::BadRequest` since MFA challenges
/// require Redis.
///
/// # Errors
///
/// Returns `AuthError::BadRequest` if Redis is unavailable.
/// Returns `AuthError::Unauthorized` if the challenge token is not
/// found or has expired. Returns `AuthError::RedisError` on Redis errors.
///
/// # Side Effects
///
/// - Reads and deletes Redis key `mfa:challenge:{token}`.
pub async fn consume_mfa_challenge(
    conn: Option<&MultiplexedConnection>,
    mfa_token: &str,
) -> Result<Uuid, AuthError> {
    let Some(conn) = conn else {
        return Err(AuthError::BadRequest(
            "MFA requires Redis, which is unavailable".to_string(),
        ));
    };
    let mut c = conn.clone();

    let key = format!("mfa:challenge:{mfa_token}");
    let value: Option<String> = c.get(&key).await.map_err(AuthError::RedisError)?;

    let value = value
        .ok_or_else(|| AuthError::Unauthorized("MFA challenge expired or invalid".to_string()))?;

    let _: () = c.del(&key).await.map_err(AuthError::RedisError)?;

    let data: serde_json::Value =
        serde_json::from_str(&value).map_err(|e| AuthError::InternalError(e.to_string()))?;

    let user_id_str = data["userId"]
        .as_str()
        .ok_or_else(|| AuthError::InternalError("Invalid MFA challenge data".to_string()))?;

    Uuid::parse_str(user_id_str)
        .map_err(|e| AuthError::InternalError(format!("Invalid user ID in challenge: {e}")))
}

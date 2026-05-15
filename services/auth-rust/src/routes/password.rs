use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use chrono::Utc;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AuthError;
use crate::events::publisher;
use crate::middleware::auth::get_client_ip;
use crate::middleware::rate_limit::check_rate_limit;
use crate::models::oauth::ResetPasswordRequest;
use crate::services::auth;

/// POST /api/auth/forgot-password — Request a password reset email.
///
/// # Expected Behavior
///
/// Rate-limited to 100 requests per minute per IP. Always returns the
/// same message to prevent email enumeration: "If an account with that
/// email exists, a reset link has been sent." If the email exists in
/// `auth.users`, generates a random reset token (UUID), inserts it
/// into `auth.password_reset_tokens` with a 1-hour expiration, and
/// publishes a `user.password_reset` event (which an email service
/// can consume to send the actual email). If the email does not exist,
/// no token is created but the response is identical.
///
/// # Errors
///
/// Returns 429 if rate limit is exceeded.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
/// - Writes to `auth.password_reset_tokens` table (insert token).
/// - Publishes `user.password_reset` event to Redis (only if user found).
/// - Increments Redis rate limit counter.
pub async fn forgot_password(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let ip = get_client_ip(&req);

    if let Err(e) = check_rate_limit(
        redis_conn.get_ref().as_ref(),
        "forgot-password",
        &ip,
        100,
        60,
    )
    .await
    {
        return e.error_response();
    }

    let email = body["email"].as_str().unwrap_or("").to_string();

    if let Ok(Some(user)) = auth::find_user_by_email(pool.get_ref(), &email).await {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now().naive_utc() + chrono::Duration::hours(1);

        let result = sqlx::query(
            "INSERT INTO auth.password_reset_tokens (id, user_id, token, expires_at, created_at)
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(user.id)
        .bind(&token)
        .bind(expires_at)
        .execute(pool.get_ref())
        .await;

        if let Err(e) = result {
            log::error!("Failed to create password reset token: {e}");
        } else {
            publisher::user_password_reset(redis_conn.get_ref().as_ref(), &user.id.to_string())
                .await;
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "If an account with that email exists, a reset link has been sent."
    }))
}

/// POST /api/auth/reset-password — Reset password using a valid reset token.
///
/// # Expected Behavior
///
/// Rate-limited to 100 requests per minute per IP. Looks up the reset
/// token in `auth.password_reset_tokens`. Validates that the token has
/// not expired (`expires_at > NOW()`) and has not been used (`used_at IS NULL`).
/// Hashes the new password with Argon2, updates `auth.users.password_hash`,
/// and marks the token as used (`used_at = NOW()`). Revokes all existing
/// sessions for the user as a security measure. Returns a success message.
///
/// # Errors
///
/// Returns 400 if the token is invalid, expired, or already used, or if
/// the new password is too short (< 8 characters).
/// Returns 429 if rate limit is exceeded.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `auth.password_reset_tokens` table.
/// - Writes to `auth.users` table (update `password_hash`).
/// - Writes to `auth.password_reset_tokens` table (mark `used_at`).
/// - Writes to `auth.sessions` table (revoke all sessions).
/// - Deletes Redis session cache entries.
/// - Increments Redis rate limit counter.
pub async fn reset_password(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    let ip = get_client_ip(&req);

    if let Err(e) = check_rate_limit(
        redis_conn.get_ref().as_ref(),
        "reset-password",
        &ip,
        100,
        60,
    )
    .await
    {
        return e.error_response();
    }

    let body = body.into_inner();

    if body.new_password.len() < 8 {
        return AuthError::BadRequest("Password must be at least 8 characters".to_string())
            .error_response();
    }

    let reset_token = match sqlx::query_as::<_, crate::models::oauth::PasswordResetToken>(
        "SELECT id, user_id, token, expires_at, used_at, created_at FROM auth.password_reset_tokens WHERE token = $1 AND used_at IS NULL",
    )
    .bind(&body.token)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(token)) => token,
        Ok(None) => {
            return AuthError::BadRequest(
                "Invalid or expired reset token".to_string(),
            )
            .error_response();
        }
        Err(e) => return AuthError::DatabaseError(e).error_response(),
    };

    if let Some(expires_at) = reset_token.expires_at {
        if expires_at < Utc::now().naive_utc() {
            return AuthError::BadRequest("Reset token has expired".to_string()).error_response();
        }
    }

    let new_hash = match auth::hash_password(&body.new_password) {
        Ok(hash) => hash,
        Err(e) => return e.error_response(),
    };

    let update_result =
        sqlx::query("UPDATE auth.users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_hash)
            .bind(reset_token.user_id)
            .execute(pool.get_ref())
            .await;

    if let Err(e) = update_result {
        return AuthError::DatabaseError(e).error_response();
    }

    let mark_used =
        sqlx::query("UPDATE auth.password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .bind(reset_token.id)
            .execute(pool.get_ref())
            .await;

    if let Err(e) = mark_used {
        log::error!("Failed to mark reset token as used: {e}");
    }

    if let Err(e) = crate::services::session::revoke_all_user_sessions(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        reset_token.user_id,
    )
    .await
    {
        log::warn!("Failed to revoke sessions after password reset: {e}");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Password has been reset"
    }))
}

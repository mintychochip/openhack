use actix_web::{web, HttpResponse, ResponseError};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::config::Config;
use crate::errors::AuthError;
use crate::models::session::{RefreshTokenRequest, TokenResponse};
use crate::services::{auth, session};

/// POST /api/auth/refresh — Rotate refresh token and issue new access token.
///
/// # Expected Behavior
///
/// Accepts a valid refresh token, validates it (checks hash, revocation,
/// and expiration), revokes the old session, creates a new session with
/// a fresh refresh token (rotation), generates a new JWT access token,
/// and returns both. Implements refresh token rotation to prevent token
/// reuse attacks.
///
/// # Errors
///
/// Returns 401 if the refresh token is invalid, revoked, or expired.
/// Returns 500 on database or internal errors.
///
/// # Side Effects
///
/// - Reads and writes to `auth.sessions` table (validate, revoke old, insert new).
/// - Writes to Redis (delete old cache, set new cache).
/// - Publishes no event (token refresh is routine and not audit-worthy).
pub async fn refresh(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    config: web::Data<Config>,
    body: web::Json<RefreshTokenRequest>,
) -> HttpResponse {
    let body = body.into_inner();

    let (new_refresh_token, user_id) = match session::rotate_refresh_token(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &body.refresh_token,
        config.get_ref(),
    )
    .await
    {
        Ok(result) => result,
        Err(e) => return e.error_response(),
    };

    let user = match auth::find_user_by_id(pool.get_ref(), user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return AuthError::NotFound("User not found".to_string()).error_response();
        }
        Err(e) => return e.error_response(),
    };

    let access_token = match auth::create_access_token(&user, config.get_ref()) {
        Ok(token) => token,
        Err(e) => return e.error_response(),
    };

    HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
        expires_in: config.jwt_expiry_secs,
    })
}

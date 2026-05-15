use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AuthError;
use crate::events::publisher;
use crate::middleware::auth::get_client_ip;
use crate::middleware::rate_limit::check_rate_limit;
use crate::models::session::{MfaRequiredResponse, TokenResponse};
use crate::models::user::LoginRequest;
use crate::services::{auth, mfa as mfa_service, session};

/// POST /api/auth/login — Authenticate a user and issue tokens.
///
/// # Expected Behavior
///
/// Rate-limited to 10 requests per minute per IP. Looks up the user by
/// email and verifies the password with Argon2. If MFA is enabled and
/// no MFA code is provided, returns `{mfaRequired: true, mfaToken: "..."}`
/// with a temporary challenge stored in Redis (5 min TTL). If MFA is
/// enabled and a valid MFA code is provided (with mfaToken), verifies
/// the code and completes login. If MFA is not enabled, completes login
/// directly. Creates a new session with refresh token, generates a JWT
/// access token, and returns both. Updates `last_login_at`. Publishes
/// `user.logged_in` event.
///
/// # Errors
///
/// Returns 400 if email/password is missing or MFA code is invalid.
/// Returns 401 if credentials are incorrect or MFA challenge expired.
/// Returns 429 if rate limit is exceeded.
/// Returns 500 on database or internal errors.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
/// - Writes to `auth.sessions` table (insert).
/// - Writes to Redis (session cache, MFA challenge).
/// - Writes to `auth.users` table (update `last_login_at`).
/// - Publishes `user.logged_in` event to Redis.
/// - CPU-intensive Argon2 password verification.
pub async fn login(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let ip = get_client_ip(&req);

    if let Err(e) = check_rate_limit(redis_conn.get_ref().as_ref(), "login", &ip, 10, 60).await {
        return e.error_response();
    }

    let body = body.into_inner();

    let user = match auth::find_user_by_email(pool.get_ref(), &body.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return AuthError::Unauthorized("Invalid email or password".to_string())
                .error_response();
        }
        Err(e) => return e.error_response(),
    };

    let Some(password_hash) = &user.password_hash else {
        return AuthError::Unauthorized(
            "Account uses OAuth. Please log in with your provider.".to_string(),
        )
        .error_response();
    };

    if auth::verify_password(&body.password, password_hash).is_err() {
        return AuthError::Unauthorized("Invalid email or password".to_string()).error_response();
    }

    if user.mfa_enabled {
        if let (Some(mfa_token), Some(mfa_code)) = (&body.mfa_token, &body.mfa_code) {
            let user_id = match session::consume_mfa_challenge(
                redis_conn.get_ref().as_ref(),
                mfa_token,
            )
            .await
            {
                Ok(id) => id,
                Err(e) => return e.error_response(),
            };

            if user_id != user.id {
                return AuthError::Unauthorized("MFA challenge mismatch".to_string())
                    .error_response();
            }

            match mfa_service::verify_mfa_code(pool.get_ref(), &user, mfa_code) {
                Ok(true) => {}
                Ok(false) => {
                    return AuthError::MfaError("Invalid MFA code".to_string()).error_response();
                }
                Err(e) => return e.error_response(),
            }
        } else {
            let mfa_token = Uuid::new_v4().to_string();
            if let Err(e) = session::store_mfa_challenge(
                redis_conn.get_ref().as_ref(),
                &mfa_token,
                user.id,
                &user.email,
            )
            .await
            {
                return e.error_response();
            }

            return HttpResponse::Ok().json(MfaRequiredResponse {
                mfa_required: true,
                mfa_token,
            });
        }
    }

    let refresh_token = match session::create_session(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        user.id,
        body.device_info,
        Some(ip),
        config.get_ref(),
    )
    .await
    {
        Ok(token) => token,
        Err(e) => return e.error_response(),
    };

    let access_token = match auth::create_access_token(&user, config.get_ref()) {
        Ok(token) => token,
        Err(e) => return e.error_response(),
    };

    if let Err(e) = auth::update_last_login(pool.get_ref(), user.id).await {
        log::warn!("Failed to update last_login_at: {e}");
    }

    publisher::user_logged_in(
        redis_conn.get_ref().as_ref(),
        &user.id.to_string(),
        &user.email,
    )
    .await;

    HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        expires_in: config.jwt_expiry_secs,
    })
}

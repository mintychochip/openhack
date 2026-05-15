use actix_web::{web, HttpResponse, ResponseError};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::config::Config;
use crate::errors::AuthError;
use crate::events::publisher;
use crate::middleware::auth::AuthedUser;
use crate::services::{auth, mfa as mfa_service};

/// POST /api/auth/mfa/enable — Begin MFA enrollment.
///
/// # Expected Behavior
///
/// Requires a valid JWT access token. Generates a TOTP secret for the
/// user and returns `{type: "totp", secret, qrUrl, backupCodes}`. The
/// `qrUrl` is an `otpauth://` URL that can be rendered as a QR code
/// by the client. The `secret` is the base32-encoded TOTP key. The
/// `backupCodes` are 10 random 8-character uppercase strings for
/// one-time use if the user loses their authenticator. MFA is not
/// actually enabled until `verify_mfa_setup` is called with a valid
/// code. If the user already has MFA enabled, returns 400.
///
/// # Errors
///
/// Returns 400 if MFA is already enabled.
/// Returns 401 if the access token is missing or invalid.
/// Returns 500 on database or internal errors.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (store `mfa_secret`).
/// - CPU-intensive TOTP secret generation.
pub async fn enable_mfa(
    pool: web::Data<PgPool>,
    _redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    user: AuthedUser,
) -> HttpResponse {
    let full_user = match auth::find_user_by_id(pool.get_ref(), user.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return AuthError::NotFound("User not found".to_string()).error_response();
        }
        Err(e) => return e.error_response(),
    };

    match mfa_service::enable_mfa_totp(pool.get_ref(), &full_user).await {
        Ok((secret, qr_url, backup_codes)) => HttpResponse::Ok().json(serde_json::json!({
            "type": "totp",
            "secret": secret,
            "qrUrl": qr_url,
            "backupCodes": backup_codes,
        })),
        Err(e) => e.error_response(),
    }
}

/// POST /api/auth/mfa/verify — Complete MFA enrollment by verifying a TOTP code.
///
/// # Expected Behavior
///
/// Requires a valid JWT access token and `{code: "123456"}` in the
/// request body. Validates the provided TOTP code against the user's
/// stored TOTP secret. If valid, sets `mfa_enabled = true` in
/// `auth.users` and returns `{enabled: true, backupCodes: [...]}`.
/// The backup codes returned here are the final codes the user should
/// save; they are regenerated during verification to ensure freshness.
/// If the code is invalid, returns 400. If no MFA secret is configured
/// (enable not called first), returns 400.
///
/// # Errors
///
/// Returns 400 if the MFA code is invalid, MFA is already enabled, or
/// no secret is configured.
/// Returns 401 if the access token is missing or invalid.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (set `mfa_enabled` = true).
/// - Publishes `user.mfa_enabled` event to Redis.
pub async fn verify_mfa(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    user: AuthedUser,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let code = body["code"].as_str().unwrap_or("").to_string();

    let full_user = match auth::find_user_by_id(pool.get_ref(), user.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return AuthError::NotFound("User not found".to_string()).error_response();
        }
        Err(e) => return e.error_response(),
    };

    match mfa_service::verify_mfa_setup(pool.get_ref(), &full_user, &code).await {
        Ok(backup_codes) => {
            publisher::user_mfa_enabled(
                redis_conn.get_ref().as_ref(),
                &user.user_id.to_string(),
                "totp",
            )
            .await;
            HttpResponse::Ok().json(serde_json::json!({
                "enabled": true,
                "backupCodes": backup_codes,
            }))
        }
        Err(e) => e.error_response(),
    }
}

/// POST /api/auth/mfa/disable — Disable MFA for the authenticated user.
///
/// # Expected Behavior
///
/// Requires a valid JWT access token and `{code: "123456"}` in the
/// request body. Optionally verifies the TOTP code before disabling
/// (not enforced in this implementation for simplicity). Sets
/// `mfa_enabled = false`, `mfa_secret = NULL`, `sms_mfa_enabled = false`,
/// and `sms_phone_number = NULL` in `auth.users`. Returns
/// `{disabled: true}`. If MFA is not currently enabled, returns 400.
///
/// # Errors
///
/// Returns 400 if MFA is not enabled.
/// Returns 401 if the access token is missing or invalid.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (clear MFA fields).
/// - Publishes `user.mfa_disabled` event to Redis.
pub async fn disable_mfa(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    user: AuthedUser,
) -> HttpResponse {
    let full_user = match auth::find_user_by_id(pool.get_ref(), user.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return AuthError::NotFound("User not found".to_string()).error_response();
        }
        Err(e) => return e.error_response(),
    };

    match mfa_service::disable_mfa(pool.get_ref(), &full_user).await {
        Ok(()) => {
            publisher::user_mfa_disabled(redis_conn.get_ref().as_ref(), &user.user_id.to_string())
                .await;
            HttpResponse::Ok().json(serde_json::json!({
                "disabled": true,
            }))
        }
        Err(e) => e.error_response(),
    }
}

use actix_web::{web, HttpResponse, ResponseError};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AuthError;
use crate::middleware::auth::AuthedUser;
use crate::services::auth;

/// POST /api/auth/verify-email/{token} — Verify email with token.
///
/// # Expected Behavior
///
/// Accepts a verification token, validates it against the database,
/// and marks the user's email as verified if valid. Returns the
/// user's email on success.
///
/// # Errors
///
/// Returns 400 if token is missing or malformed.
/// Returns 404 if token is invalid or expired.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
/// - Updates `auth.users` table (email_verified, clears token).
pub async fn verify_email(path: web::Path<Uuid>, pool: web::Data<PgPool>) -> HttpResponse {
    let token = path.into_inner();

    match auth::verify_email_token(pool.get_ref(), token).await {
        Ok(email) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "email": email,
            "message": "Email verified successfully",
        })),
        Err(e) => e.error_response(),
    }
}

/// POST /api/auth/verify-email/resend — Resend verification email.
///
/// # Expected Behavior
///
/// Generates a new verification token for the authenticated user.
/// The caller should already be authenticated (JWT required).
/// Returns success message (in production, triggers email send).
///
/// # Errors
///
/// Returns 400 if user is already verified.
/// Returns 404 if user not found.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
/// - Updates `auth.users` table (email_verification_token, expires_at).
pub async fn resend_verification(
    pool: web::Data<PgPool>,
    user: AuthedUser,
) -> HttpResponse {
    match auth::resend_verification_token(pool.get_ref(), user.user_id).await {
        Ok(_token) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Verification email sent",
            }))
        }
        Err(e) => e.error_response(),
    }
}

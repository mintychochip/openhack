use actix_web::{web, HttpResponse, ResponseError};
use sqlx::PgPool;

use crate::errors::AuthError;
use crate::middleware::auth::AuthedUser;
use crate::models::user::{UpdateProfileRequest, UserProfile};
use crate::services::auth;

/// GET /api/auth/me — Get the authenticated user's profile.
///
/// # Expected Behavior
///
/// Requires a valid JWT access token. Looks up the user by ID from the
/// JWT subject claim. Returns the user's public profile (`UserProfile`),
/// which excludes the password hash and MFA secret. If the user no
/// longer exists (e.g., deleted after token issuance), returns 404.
///
/// # Errors
///
/// Returns 401 if the access token is missing or invalid.
/// Returns 404 if the user does not exist.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
pub async fn get_me(pool: web::Data<PgPool>, user: AuthedUser) -> HttpResponse {
    match auth::find_user_by_id(pool.get_ref(), user.user_id).await {
        Ok(Some(u)) => {
            let profile: UserProfile = u.into();
            HttpResponse::Ok().json(profile)
        }
        Ok(None) => AuthError::NotFound("User not found".to_string()).error_response(),
        Err(e) => e.error_response(),
    }
}

/// PUT /api/auth/me — Update the authenticated user's profile.
///
/// # Expected Behavior
///
/// Requires a valid JWT access token. Accepts optional fields: name,
/// `avatar_url`, `github_username`, `discord_id`. Only non-None fields are
/// updated. Returns the updated user profile. Email and password changes
/// are not supported through this endpoint.
///
/// # Errors
///
/// Returns 400 if name is empty (when provided).
/// Returns 401 if the access token is missing or invalid.
/// Returns 404 if the user does not exist.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (update fields).
pub async fn update_me(
    pool: web::Data<PgPool>,
    user: AuthedUser,
    body: web::Json<UpdateProfileRequest>,
) -> HttpResponse {
    let body = body.into_inner();

    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return AuthError::BadRequest("Name cannot be empty".to_string()).error_response();
        }
    }

    match auth::update_user_profile(pool.get_ref(), user.user_id, &body).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => e.error_response(),
    }
}

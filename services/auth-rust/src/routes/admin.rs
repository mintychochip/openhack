use actix_web::{web, HttpResponse, ResponseError};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AuthError;
use crate::middleware::auth::{is_admin, AuthedUser};
use crate::models::user::{UpdateRoleRequest, UserListQuery, UserListResponse, UserProfile};
use crate::services::auth;

/// GET /api/auth/users — Paginated list of all users (admin only).
///
/// # Expected Behavior
///
/// Requires a valid JWT with the "admin" role. Returns a paginated list
/// of user profiles. Supports optional `search` query parameter to filter
/// by email or name (case-insensitive ILIKE). `limit` defaults to 20,
/// max 100. `offset` defaults to 0. Returns `{users, total, limit, offset}`.
///
/// # Errors
///
/// Returns 401 if the access token is missing or invalid.
/// Returns 403 if the user is not an admin.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
pub async fn list_users(
    pool: web::Data<PgPool>,
    _redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    user: AuthedUser,
    query: web::Query<UserListQuery>,
) -> HttpResponse {
    if !is_admin(&user) {
        return AuthError::Forbidden("Admin access required".to_string()).error_response();
    }

    let query = query.into_inner();
    let limit = query.effective_limit();
    let offset = query.effective_offset();
    let search = query.search.as_deref();

    match auth::list_users(pool.get_ref(), limit, offset, search).await {
        Ok((users, total)) => {
            let profiles: Vec<UserProfile> =
                users.into_iter().map(std::convert::Into::into).collect();
            HttpResponse::Ok().json(UserListResponse {
                users: profiles,
                total,
                limit,
                offset,
            })
        }
        Err(e) => e.error_response(),
    }
}

/// DELETE /api/auth/users/{id} — Delete a user (admin only).
///
/// # Expected Behavior
///
/// Requires a valid JWT with the "admin" role. Deletes the user with the
/// given ID from `auth.users`. Cascading deletes remove associated sessions,
/// OAuth accounts, and password reset tokens (requires ON DELETE CASCADE
/// in the schema). Admins cannot delete themselves. Returns 200 on success.
///
/// # Errors
///
/// Returns 401 if the access token is missing or invalid.
/// Returns 403 if the user is not an admin.
/// Returns 400 if the admin attempts to delete themselves.
/// Returns 404 if the target user does not exist.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Deletes from `auth.users`, `auth.sessions`, `auth.oauth_accounts`,
///   and `auth.password_reset_tokens` tables (cascade).
pub async fn delete_user(
    pool: web::Data<PgPool>,
    _redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    user: AuthedUser,
    target_id: web::Path<Uuid>,
) -> HttpResponse {
    if !is_admin(&user) {
        return AuthError::Forbidden("Admin access required".to_string()).error_response();
    }

    let target_id = target_id.into_inner();

    if target_id == user.user_id {
        return AuthError::BadRequest("Cannot delete your own account".to_string())
            .error_response();
    }

    match auth::find_user_by_id(pool.get_ref(), target_id).await {
        Ok(Some(_)) => match auth::delete_user(pool.get_ref(), target_id).await {
            Ok(()) => HttpResponse::Ok().json(serde_json::json!({
                "deleted": true,
                "id": target_id.to_string(),
            })),
            Err(e) => e.error_response(),
        },
        Ok(None) => AuthError::NotFound("User not found".to_string()).error_response(),
        Err(e) => e.error_response(),
    }
}

/// POST /api/auth/users/{id}/role — Update a user's roles (admin only).
///
/// # Expected Behavior
///
/// Requires a valid JWT with the "admin" role. Accepts `{roles: ["participant", "judge"]}`
/// in the request body. Replaces the target user's role list entirely.
/// Admins cannot remove their own admin role. Returns the updated user profile.
///
/// # Errors
///
/// Returns 401 if the access token is missing or invalid.
/// Returns 403 if the user is not an admin.
/// Returns 400 if the roles array is empty or the admin attempts to remove
/// their own admin role.
/// Returns 404 if the target user does not exist.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (update roles).
pub async fn update_role(
    pool: web::Data<PgPool>,
    _redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    user: AuthedUser,
    target_id: web::Path<Uuid>,
    body: web::Json<UpdateRoleRequest>,
) -> HttpResponse {
    if !is_admin(&user) {
        return AuthError::Forbidden("Admin access required".to_string()).error_response();
    }

    let target_id = target_id.into_inner();
    let body = body.into_inner();

    if body.roles.is_empty() {
        return AuthError::BadRequest("Roles cannot be empty".to_string()).error_response();
    }

    if target_id == user.user_id && !body.roles.contains(&"admin".to_string()) {
        return AuthError::BadRequest("Cannot remove your own admin role".to_string())
            .error_response();
    }

    match auth::update_user_roles(pool.get_ref(), target_id, &body.roles).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => e.error_response(),
    }
}

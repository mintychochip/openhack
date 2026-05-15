use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::middleware::auth::AuthUser;
use crate::models::announcement::{CreateAnnouncementRequest, ListAnnouncementsQuery};
use crate::services::announcement;

/// Create a new announcement.
///
/// # Expected Behavior
///
/// Accepts a JSON body with title, content, audience, channels, and optional
/// status and `scheduled_at`. Creates the announcement in the database and
/// returns 201 with the created announcement. Requires authentication.
///
/// # Errors
///
/// Returns 400 if the request body is malformed. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Writes to `notify.announcements` table via service layer.
pub async fn create_announcement(
    pool: web::Data<PgPool>,
    req: web::Json<CreateAnnouncementRequest>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    match announcement::create_announcement(pool.get_ref(), &req).await {
        Ok(a) => HttpResponse::Created().json(a),
        Err(e) => e.to_http_response(),
    }
}

/// List announcements with optional status filter and pagination.
///
/// # Expected Behavior
///
/// Returns a paginated list of announcements. Supports `limit`, `offset`,
/// and `status` query parameters. Requires authentication.
///
/// # Errors
///
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `notify.announcements` table.
pub async fn list_announcements(
    pool: web::Data<PgPool>,
    query: web::Query<ListAnnouncementsQuery>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    match announcement::list_announcements(pool.get_ref(), &query).await {
        Ok(announcements) => HttpResponse::Ok().json(announcements),
        Err(e) => e.to_http_response(),
    }
}

/// Get a single announcement by ID.
///
/// # Expected Behavior
///
/// Returns the announcement with the given UUID. Returns 404 if not found.
/// Requires authentication.
///
/// # Errors
///
/// Returns 404 if the announcement does not exist. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `notify.announcements` table.
pub async fn get_announcement(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    let id = id.into_inner();
    match announcement::get_announcement(pool.get_ref(), id).await {
        Ok(a) => HttpResponse::Ok().json(a),
        Err(e) => e.to_http_response(),
    }
}

/// Delete an announcement by ID.
///
/// # Expected Behavior
///
/// Deletes the announcement with the given UUID. Returns 404 if not found.
/// Returns 200 with empty body on success. Requires authentication.
///
/// # Errors
///
/// Returns 404 if the announcement does not exist. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Deletes from `notify.announcements` table.
pub async fn delete_announcement(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    let id = id.into_inner();
    match announcement::delete_announcement(pool.get_ref(), id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Err(e) => e.to_http_response(),
    }
}

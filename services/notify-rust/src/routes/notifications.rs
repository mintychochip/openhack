use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::middleware::auth::AuthUser;
use crate::models::notification::ListNotificationsQuery;
use crate::services::notification;

/// Get user notifications.
///
/// # Expected Behavior
///
/// Returns a paginated list of notifications for the authenticated user,
/// ordered by `created_at` descending. Supports `limit` and `offset` query
/// parameters. Requires authentication.
///
/// # Errors
///
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `notify.user_notifications` table.
pub async fn get_notifications(
    pool: web::Data<PgPool>,
    query: web::Query<ListNotificationsQuery>,
    user: web::ReqData<AuthUser>,
) -> HttpResponse {
    match notification::get_user_notifications(pool.get_ref(), user.user_id, &query).await {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => e.to_http_response(),
    }
}

/// Mark a notification as read.
///
/// # Expected Behavior
///
/// Marks the notification with the given ID as read. The notification must
/// belong to the authenticated user. Returns 404 if not found or not owned.
/// Requires authentication.
///
/// # Errors
///
/// Returns 404 if the notification does not exist for this user.
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Updates `notify.user_notifications` table.
pub async fn mark_notification_read(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    user: web::ReqData<AuthUser>,
) -> HttpResponse {
    let id = id.into_inner();
    match notification::mark_notification_read(pool.get_ref(), id, user.user_id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"read": true})),
        Err(e) => e.to_http_response(),
    }
}

/// Mark all notifications as read for the authenticated user.
///
/// # Expected Behavior
///
/// Marks all unread notifications as read for the authenticated user.
/// Returns the count of updated notifications. Requires authentication.
///
/// # Errors
///
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Updates multiple rows in `notify.user_notifications` table.
pub async fn mark_all_read(pool: web::Data<PgPool>, user: web::ReqData<AuthUser>) -> HttpResponse {
    match notification::mark_all_read(pool.get_ref(), user.user_id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({"updated": count})),
        Err(e) => e.to_http_response(),
    }
}

/// Get the count of unread notifications for the authenticated user.
///
/// # Expected Behavior
///
/// Returns a JSON object with a `count` field containing the number of
/// unread notifications. Requires authentication.
///
/// # Errors
///
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `notify.user_notifications` table.
pub async fn get_unread_count(
    pool: web::Data<PgPool>,
    user: web::ReqData<AuthUser>,
) -> HttpResponse {
    match notification::get_unread_count(pool.get_ref(), user.user_id).await {
        Ok(count) => HttpResponse::Ok().json(count),
        Err(e) => e.to_http_response(),
    }
}

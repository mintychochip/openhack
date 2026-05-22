use crate::middleware::auth::{is_admin, AuthedUser};
use crate::services::gdpr::GdprService;
use actix_web::{web, HttpResponse, ResponseError};
use sqlx::PgPool;
use uuid::Uuid;

/// Export all data for a user (GDPR compliance).
///
/// # Expected Behavior
///
/// Returns a comprehensive JSON export of all data associated with a user:
/// - Profile information
/// - Session history
/// - OAuth account links
/// - Audit log entries
/// - Submissions (from core service)
/// - Scores/judgments (from judging service)
/// - Team memberships
/// - Event participations
///
/// Only the user themselves or an admin can export their data.
///
/// # Errors
///
/// Returns 401 if not authenticated, 403 if not the user or admin,
/// 404 if user not found, 500 on database error.
///
/// # Side Effects
///
/// - Reads from multiple database tables (auth, core, judging schemas)
/// - Logs audit event for data export
pub async fn export_user_data(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    user: AuthedUser,
) -> HttpResponse {
    let user_id = path.into_inner();

    if user.user_id != user_id && !is_admin(&user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "You can only export your own data"
        }));
    }

    match GdprService::export_user_data(pool.get_ref(), user_id).await {
        Ok(export) => HttpResponse::Ok()
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Content-Disposition", format!("attachment; filename=\"user-{user_id}-export.json\"")))
            .json(export),
        Err(e) => e.error_response(),
    }
}

/// Request data deletion (GDPR right to be forgotten).
///
/// # Expected Behavior
///
/// Marks a user account for deletion. The account is anonymized rather than
/// hard-deleted to preserve referential integrity. Scheduled for deletion
/// after 30 days (configurable). User receives confirmation email.
///
/// Only the user themselves can request deletion (not reversible by user).
///
/// # Errors
///
/// Returns 401 if not authenticated, 403 if not the user,
/// 404 if user not found, 500 on database error.
///
/// # Side Effects
///
/// - Updates user record (sets deletion_requested_at)
/// - Sends deletion confirmation email
/// - Logs audit event
pub async fn request_deletion(
    pool: web::Data<PgPool>,
    user: AuthedUser,
) -> HttpResponse {
    let user_id = user.user_id;

    match GdprService::request_deletion(pool.get_ref(), user_id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "deletion_requested",
            "message": "Your account will be deleted in 30 days",
            "deletion_date": chrono::Utc::now() + chrono::Duration::days(30)
        })),
        Err(e) => e.error_response(),
    }
}

use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::{get_auth_user, require_sponsor_role};
use crate::models::submission::SubmissionCreate;
use crate::services::prize::PrizeService;
use crate::services::submission::SubmissionService;

/// Submit a project for a prize.
///
/// # Expected Behavior
///
/// Requires authentication. Creates a submission linking the project to
/// the prize. The `prize_id` is taken from the URL path parameter and the
/// `project_id` from the request body. Returns 201 Created with the
/// submission data on success.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 404 if the prize does
/// not exist. Returns 500 on database errors (including unique constraint
/// violations for duplicate submissions).
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Creates a submission row in the database (write).
pub async fn submit_project(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<SubmissionCreate>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let prize_id = path.into_inner();
    match SubmissionService::submit_project(pool.get_ref(), prize_id, &body.into_inner()).await {
        Ok(submission) => HttpResponse::Created().json(submission),
        Err(e) => e.to_http_response(),
    }
}

/// List all submissions for a prize.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Returns all submissions
/// for the given prize ID, ordered by submission time descending.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.submissions` (database read).
pub async fn list_submissions(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let prize_id = path.into_inner();
    match SubmissionService::list_submissions(pool.get_ref(), prize_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

/// Select a winner for a prize from submissions.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Verifies that a submission
/// exists for the given (`prize_id`, `project_id`) pair, then sets the
/// `winner_project_id` and `announced = true` on the prize. Returns the
/// updated prize data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no submission exists for the given
/// (`prize_id`, `project_id`) pair, or if the prize does not exist. Returns
/// 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.submissions` to verify existence (database read).
/// - Updates the prize row in the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn select_winner(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let (prize_id, project_id) = path.into_inner();
    match PrizeService::select_winner(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        prize_id,
        project_id,
    )
    .await
    {
        Ok(prize) => HttpResponse::Ok().json(prize),
        Err(e) => e.to_http_response(),
    }
}

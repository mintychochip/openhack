use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::{get_auth_user, require_sponsor_role};
use crate::models::booth::{BoothCreate, BoothUpdate};
use crate::models::prize::{PrizeCreate, PrizeUpdate};
use crate::services::booth::BoothService;
use crate::services::prize::PrizeService;
use crate::services::submission::SubmissionService;

/// Request body for selecting a prize winner by team.
///
/// # Expected Behavior
///
/// Contains `team_id` which is the UUID of the team being selected as the
/// winner. The submission for this (`prize_id`, `team_id`) pair must already
/// exist in the database.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WinnerSelect {
    pub team_id: Uuid,
}

/// Get the authenticated sponsor's own booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Looks up the booth where
/// `sponsor_id` matches the authenticated user's ID from the JWT. Returns
/// 200 OK with the booth data, or 404 if no booth exists for this sponsor.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no booth exists for the sponsor.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.booths` (database read).
pub async fn get_sponsor_booth(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    match BoothService::get_sponsor_booth(pool.get_ref(), &auth_user.user_id.to_string()).await {
        Ok(booth) => HttpResponse::Ok().json(booth),
        Err(e) => e.to_http_response(),
    }
}

/// Create a booth for the authenticated sponsor.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Creates a new booth with
/// `sponsor_id` set to the authenticated user's ID from the JWT. Returns
/// 201 Created with the booth data on success.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Creates a booth row in the database with `sponsor_id` (write).
/// - Invalidates the published booths cache (write).
pub async fn create_sponsor_booth(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<BoothCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    match BoothService::create_sponsor_booth(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &auth_user.user_id.to_string(),
        &body.into_inner(),
    )
    .await
    {
        Ok(booth) => HttpResponse::Created().json(booth),
        Err(e) => e.to_http_response(),
    }
}

/// Update the authenticated sponsor's booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Updates the booth with the
/// given ID, but only if the booth's `sponsor_id` matches the authenticated
/// user's ID (ownership check). Returns 200 OK with the updated booth data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks the
/// required role or does not own the booth. Returns 404 if no booth exists
/// with the given ID. Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.booths` to verify ownership (database read).
/// - Updates the booth row in the database (write).
/// - Invalidates booth, published booths, and prizes caches (writes).
pub async fn update_sponsor_booth(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<BoothUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match BoothService::update_booth_owner_check(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &auth_user.user_id.to_string(),
        &body.into_inner(),
    )
    .await
    {
        Ok(booth) => HttpResponse::Ok().json(booth),
        Err(e) => e.to_http_response(),
    }
}

/// Delete the authenticated sponsor's booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Deletes the booth with the
/// given ID, but only if the booth's `sponsor_id` matches the authenticated
/// user's ID (ownership check). Returns 204 No Content on success, 404 if
/// the booth does not exist, 403 if the user does not own the booth.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks the
/// required role or does not own the booth. Returns 404 if no booth exists.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.booths` to verify ownership (database read).
/// - Deletes the booth row from the database (write).
/// - Invalidates booth, published booths, and prizes caches (writes).
pub async fn delete_sponsor_booth(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
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

    let id = path.into_inner();
    match BoothService::delete_booth_owner_check(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &auth_user.user_id.to_string(),
    )
    .await
    {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({"error": "Booth not found"})),
        Err(e) => e.to_http_response(),
    }
}

/// List all prizes created by the authenticated sponsor.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Returns all prizes belonging
/// to booths where `sponsor_id` matches the authenticated user's ID.
/// Includes total count for pagination.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.prizes` and `sponsor.booths` (database reads).
pub async fn list_sponsor_prizes(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    match PrizeService::list_sponsor_prizes(pool.get_ref(), &auth_user.user_id.to_string()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

/// Create a prize for the authenticated sponsor's booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Finds the booth owned by the
/// authenticated user (by `sponsor_id`) and creates a prize for that booth.
/// Returns 201 Created with the prize data on success. Returns 404 if the
/// sponsor has no booth.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no booth exists for the sponsor.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.booths` to find the sponsor's booth (database read).
/// - Creates a prize row in the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn create_sponsor_prize(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<PrizeCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    match PrizeService::create_sponsor_prize(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &auth_user.user_id.to_string(),
        &body.into_inner(),
    )
    .await
    {
        Ok(prize) => HttpResponse::Created().json(prize),
        Err(e) => e.to_http_response(),
    }
}

/// Update a prize owned by the authenticated sponsor.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Updates the prize with the
/// given ID, but only if the prize's booth `sponsor_id` matches the
/// authenticated user's ID (ownership check via booth). Returns 200 OK
/// with the updated prize data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks the
/// required role or does not own the prize. Returns 404 if no prize exists.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.prizes` and `sponsor.booths` to verify ownership
///   (database reads).
/// - Updates the prize row in the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn update_sponsor_prize(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<PrizeUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PrizeService::update_prize_owner_check(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &auth_user.user_id.to_string(),
        &body.into_inner(),
    )
    .await
    {
        Ok(prize) => HttpResponse::Ok().json(prize),
        Err(e) => e.to_http_response(),
    }
}

/// Delete a prize owned by the authenticated sponsor.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Deletes the prize with the
/// given ID, but only if the prize's booth `sponsor_id` matches the
/// authenticated user's ID (ownership check via booth). Returns 204 No
/// Content on success, 404 if the prize does not exist, 403 if the user
/// does not own the prize.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks the
/// required role or does not own the prize. Returns 404 if no prize exists.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.prizes` and `sponsor.booths` to verify ownership
///   (database reads).
/// - Deletes the prize row from the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn delete_sponsor_prize(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
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

    let id = path.into_inner();
    match PrizeService::delete_prize_owner_check(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &auth_user.user_id.to_string(),
    )
    .await
    {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({"error": "Prize not found"})),
        Err(e) => e.to_http_response(),
    }
}

/// Select a winner for a prize owned by the authenticated sponsor.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Verifies ownership of the
/// prize (via booth's `sponsor_id`). Verifies a submission exists for the
/// given (`prize_id`, `team_id`) pair. Sets the submission's status to
/// 'awarded' and updates the prize's `winner_project_id` and `announced =
/// true`. Returns 200 OK with the updated prize data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks the
/// required role or does not own the prize. Returns 404 if no prize or
/// submission exists. Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.prizes` and `sponsor.booths` to verify ownership
///   (database reads).
/// - Reads from `sponsor.submissions` to verify existence (database read).
/// - Updates submission status to 'awarded' (database write).
/// - Updates prize `winner_project_id` and `announced` (database write).
/// - Invalidates the booth's prizes cache (write).
pub async fn select_sponsor_winner(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<WinnerSelect>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let prize_id = path.into_inner();
    match PrizeService::select_sponsor_winner(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        prize_id,
        &auth_user.user_id.to_string(),
        body.team_id,
    )
    .await
    {
        Ok(prize) => HttpResponse::Ok().json(prize),
        Err(e) => e.to_http_response(),
    }
}

/// List all submissions for the authenticated sponsor's prizes.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Returns all submissions for
/// prizes that belong to booths where `sponsor_id` matches the authenticated
/// user's ID. Includes total count for pagination.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.submissions`, `sponsor.prizes`, and
///   `sponsor.booths` (database reads).
pub async fn list_sponsor_submissions(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    match SubmissionService::list_sponsor_submissions(
        pool.get_ref(),
        &auth_user.user_id.to_string(),
    )
    .await
    {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

/// Approve a submission owned by the authenticated sponsor.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Verifies that the
/// submission's prize's booth is owned by the authenticated user
/// (ownership check via booth's `sponsor_id`). If the check passes, sets
/// the submission's status to 'approved'. Returns 200 OK with the updated
/// submission data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks the
/// required role or does not own the submission. Returns 404 if no
/// submission exists with the given ID. Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.submissions`, `sponsor.prizes`, and
///   `sponsor.booths` to verify ownership (database reads).
/// - Updates `status` column in `sponsor.submissions` to 'approved'
///   (database write).
pub async fn approve_submission(
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

    let id = path.into_inner();
    match SubmissionService::approve_submission(pool.get_ref(), id, &auth_user.user_id.to_string())
        .await
    {
        Ok(submission) => HttpResponse::Ok().json(submission),
        Err(e) => e.to_http_response(),
    }
}

/// Reject a submission owned by the authenticated sponsor.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Verifies that the
/// submission's prize's booth is owned by the authenticated user
/// (ownership check via booth's `sponsor_id`). If the check passes, sets
/// the submission's status to 'rejected'. Returns 200 OK with the updated
/// submission data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks the
/// required role or does not own the submission. Returns 404 if no
/// submission exists with the given ID. Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.submissions`, `sponsor.prizes`, and
///   `sponsor.booths` to verify ownership (database reads).
/// - Updates `status` column in `sponsor.submissions` to 'rejected'
///   (database write).
pub async fn reject_submission(
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

    let id = path.into_inner();
    match SubmissionService::reject_submission(pool.get_ref(), id, &auth_user.user_id.to_string())
        .await
    {
        Ok(submission) => HttpResponse::Ok().json(submission),
        Err(e) => e.to_http_response(),
    }
}

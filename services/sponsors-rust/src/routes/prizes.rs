use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::{get_auth_user, require_sponsor_role};
use crate::models::prize::{PrizeAnnounce, PrizeCreate, PrizeUpdate};
use crate::services::prize::PrizeService;

/// Create a new prize for a booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. The `booth_id` is taken
/// from the URL path parameter. Returns 201 Created with the prize data
/// on success.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if the booth does not exist. Returns
/// 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Creates a prize row in the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn create_prize(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<PrizeCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let booth_id = path.into_inner();
    match PrizeService::create_prize(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        booth_id,
        &body.into_inner(),
    )
    .await
    {
        Ok(prize) => HttpResponse::Created().json(prize),
        Err(e) => e.to_http_response(),
    }
}

/// List all prizes for a booth.
///
/// # Expected Behavior
///
/// Requires authentication. Returns all prizes for the given booth ID.
/// Checks Redis cache first.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 500 on database/Redis
/// errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn list_prizes(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let booth_id = path.into_inner();
    match PrizeService::list_prizes(pool.get_ref(), redis_conn.get_ref().as_ref(), booth_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

/// Update an existing prize.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Updates only the fields
/// provided in the request body. Returns the updated prize data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no prize exists with the given ID.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates the prize row in the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn update_prize(
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
    match PrizeService::update_prize(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &body.into_inner(),
    )
    .await
    {
        Ok(prize) => HttpResponse::Ok().json(prize),
        Err(e) => e.to_http_response(),
    }
}

/// Delete a prize by ID.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Deletes the prize. Returns
/// 204 No Content on success, 404 if the prize does not exist.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no prize exists. Returns 500 on
/// database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Deletes the prize row from the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn delete_prize(
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
    match PrizeService::delete_prize(pool.get_ref(), redis_conn.get_ref().as_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({"error": "Prize not found"})),
        Err(e) => e.to_http_response(),
    }
}

/// Announce a winner for a prize.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Sets `winner_project_id`
/// and `announced = true` on the prize. Returns the updated prize data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no prize exists with the given ID.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates the prize row in the database (write).
/// - Invalidates the booth's prizes cache (write).
pub async fn announce_winner(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<PrizeAnnounce>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PrizeService::announce_winner(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &body.into_inner(),
    )
    .await
    {
        Ok(prize) => HttpResponse::Ok().json(prize),
        Err(e) => e.to_http_response(),
    }
}

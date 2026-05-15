use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::{get_auth_user, require_sponsor_role};
use crate::models::booth::{BoothCreate, BoothPublish, BoothUpdate};
use crate::services::booth::BoothService;

/// Create a new sponsor booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Validates the JWT from
/// the Authorization header, then delegates to `BoothService::create_booth`.
/// Returns 201 Created with the booth data on success.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Creates a booth row in the database (write).
/// - Invalidates the published booths cache (write).
pub async fn create_booth(
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

    match BoothService::create_booth(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &body.into_inner(),
    )
    .await
    {
        Ok(booth) => HttpResponse::Created().json(booth),
        Err(e) => e.to_http_response(),
    }
}

/// List all published booths (public endpoint).
///
/// # Expected Behavior
///
/// Returns a paginated list of booths where `published = true`. No
/// authentication required. Supports `limit` and `offset` query parameters
/// (defaults: 100 and 0). Checks Redis cache first.
///
/// # Errors
///
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn list_published_booths(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    match BoothService::list_published_booths(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        limit,
        offset,
    )
    .await
    {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

/// Get a single booth by ID, incrementing the view count.
///
/// # Expected Behavior
///
/// Requires authentication. Returns the booth data and increments the
/// view count by 1. Checks Redis cache first for the booth data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 404 if no booth exists
/// with the given ID. Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Increments `view_count` in the database (write).
/// - Reads from Redis cache (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn get_booth(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let id = path.into_inner();
    match BoothService::get_booth(pool.get_ref(), redis_conn.get_ref().as_ref(), id).await {
        Ok(booth) => HttpResponse::Ok().json(booth),
        Err(e) => e.to_http_response(),
    }
}

/// Update an existing booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Updates only the fields
/// provided in the request body. Returns the updated booth data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no booth exists with the given ID.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates the booth row in the database (write).
/// - Invalidates booth, published booths, and prizes caches (writes).
pub async fn update_booth(
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
    match BoothService::update_booth(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &body.into_inner(),
    )
    .await
    {
        Ok(booth) => HttpResponse::Ok().json(booth),
        Err(e) => e.to_http_response(),
    }
}

/// Delete a booth by ID.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Deletes the booth and all
/// associated data (depending on cascade constraints). Returns 204 No
/// Content on success, 404 if the booth does not exist.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no booth exists with the given ID.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Deletes the booth row from the database (write).
/// - Invalidates booth, published booths, and prizes caches (writes).
pub async fn delete_booth(
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
    match BoothService::delete_booth(pool.get_ref(), redis_conn.get_ref().as_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({"error": "Booth not found"})),
        Err(e) => e.to_http_response(),
    }
}

/// Publish or unpublish a booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. If the request body
/// contains `published: true/false`, sets that value. If `published`
/// is not provided, toggles the current value. Returns the updated
/// booth data.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no booth exists with the given ID.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates the `published` column in the database (write).
/// - Invalidates booth, published booths, and prizes caches (writes).
pub async fn publish_booth(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<BoothPublish>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_sponsor_role(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match BoothService::publish_booth(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        id,
        &body.into_inner(),
    )
    .await
    {
        Ok(booth) => HttpResponse::Ok().json(booth),
        Err(e) => e.to_http_response(),
    }
}

/// Get analytics for a booth.
///
/// # Expected Behavior
///
/// Requires sponsor, organizer, or admin role. Returns the booth's
/// view count, prize count, and total submission count.
///
/// # Errors
///
/// Returns 401 if authentication fails. Returns 403 if the user lacks
/// the required role. Returns 404 if no booth exists with the given ID.
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `sponsor.booths`, `sponsor.prizes`, and
///   `sponsor.submissions` (database reads).
pub async fn get_analytics(
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
    match BoothService::get_booth_analytics(pool.get_ref(), id).await {
        Ok(analytics) => HttpResponse::Ok().json(analytics),
        Err(e) => e.to_http_response(),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

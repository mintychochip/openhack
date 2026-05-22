use crate::middleware::auth::{get_auth_user, require_admin_or_organizer};
use crate::models::assignment::{
    AssignmentBulkRequest, AssignmentCreate, AssignmentQuery, AssignmentUpdate,
};
use crate::services::assignment::AssignmentService;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_assignment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<AssignmentCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    match AssignmentService::create(pool.get_ref(), &body.into_inner()).await {
        Ok(a) => HttpResponse::Created().json(a),
        Err(e) => e.to_http_response(),
    }
}

pub async fn bulk_assign(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<AssignmentBulkRequest>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    match AssignmentService::bulk_assign(pool.get_ref(), &body.into_inner()).await {
        Ok(list) => HttpResponse::Created().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_assignments(
    pool: web::Data<PgPool>,
    query: web::Query<AssignmentQuery>,
) -> HttpResponse {
    match AssignmentService::list(pool.get_ref(), &query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_assignment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AssignmentUpdate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match AssignmentService::update(pool.get_ref(), id, &body.into_inner()).await {
        Ok(a) => HttpResponse::Ok().json(a),
        Err(e) => e.to_http_response(),
    }
}

pub async fn delete_assignment(req: HttpRequest, pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match AssignmentService::delete(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Assignment not found"}))
        }
        Err(e) => e.to_http_response(),
    }
}

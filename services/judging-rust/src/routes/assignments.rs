use crate::models::assignment::{
    AssignmentBulkRequest, AssignmentCreate, AssignmentQuery, AssignmentUpdate,
};
use crate::services::assignment::AssignmentService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_assignment(
    pool: web::Data<PgPool>,
    body: web::Json<AssignmentCreate>,
) -> HttpResponse {
    match AssignmentService::create(pool.get_ref(), &body.into_inner()).await {
        Ok(a) => HttpResponse::Created().json(a),
        Err(e) => e.to_http_response(),
    }
}

pub async fn bulk_assign(
    pool: web::Data<PgPool>,
    body: web::Json<AssignmentBulkRequest>,
) -> HttpResponse {
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
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AssignmentUpdate>,
) -> HttpResponse {
    let id = path.into_inner();
    match AssignmentService::update(pool.get_ref(), id, &body.into_inner()).await {
        Ok(a) => HttpResponse::Ok().json(a),
        Err(e) => e.to_http_response(),
    }
}

pub async fn delete_assignment(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match AssignmentService::delete(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Assignment not found"}))
        }
        Err(e) => e.to_http_response(),
    }
}

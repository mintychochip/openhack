use crate::models::rubric::{ListQuery, RubricCreate, RubricUpdate, ScoreValidationRequest};
use crate::services::judging::JudgingService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_rubric(pool: web::Data<PgPool>, body: web::Json<RubricCreate>) -> HttpResponse {
    match JudgingService::create_rubric(pool.get_ref(), &body.into_inner()).await {
        Ok(r) => HttpResponse::Created().json(r),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_rubrics(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> HttpResponse {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    match JudgingService::list_rubrics(pool.get_ref(), limit, offset).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_rubric(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match JudgingService::get_rubric(pool.get_ref(), id).await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_rubric(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<RubricUpdate>,
) -> HttpResponse {
    let id = path.into_inner();
    match JudgingService::update_rubric(pool.get_ref(), id, &body.into_inner()).await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => e.to_http_response(),
    }
}

pub async fn delete_rubric(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match JudgingService::delete_rubric(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Rubric not found"}))
        }
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_versions(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let rubric_id = path.into_inner();
    match JudgingService::list_rubric_versions(pool.get_ref(), rubric_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_version(pool: web::Data<PgPool>, path: web::Path<(Uuid, i32)>) -> HttpResponse {
    let (rubric_id, version) = path.into_inner();
    match JudgingService::get_rubric_version(pool.get_ref(), rubric_id, Some(version)).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => e.to_http_response(),
    }
}

pub async fn create_version(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let rubric_id = path.into_inner();
    match JudgingService::create_rubric_version(pool.get_ref(), rubric_id).await {
        Ok(v) => HttpResponse::Created().json(v),
        Err(e) => e.to_http_response(),
    }
}

pub async fn validate_score(
    pool: web::Data<PgPool>,
    body: web::Json<ScoreValidationRequest>,
) -> HttpResponse {
    match JudgingService::validate_score(pool.get_ref(), &body.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => e.to_http_response(),
    }
}

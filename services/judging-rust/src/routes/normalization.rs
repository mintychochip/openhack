use crate::models::normalization::{
    NormalizationRequest, NormalizationResponse, NormalizationStatusQuery,
};
use crate::services::normalization;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn normalize(
    pool: web::Data<PgPool>,
    body: web::Json<NormalizationRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    match normalization::apply_normalization(pool.get_ref(), req.phase_id, &req.method).await {
        Ok(_count) => HttpResponse::Ok().json(NormalizationResponse {
            success: true,
            result: None,
            preview: None,
            errors: vec![],
        }),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_status(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let phase_id = path.into_inner();
    match normalization::get_normalization_status(pool.get_ref(), phase_id).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_stats(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let phase_id = path.into_inner();
    match normalization::get_normalization_stats(pool.get_ref(), phase_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.to_http_response(),
    }
}

pub async fn apply(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<NormalizationStatusQuery>,
) -> HttpResponse {
    let phase_id = path.into_inner();
    let method = query.method.as_deref().unwrap_or("zscore");
    match normalization::apply_normalization(pool.get_ref(), phase_id, method).await {
        Ok(_count) => HttpResponse::Ok().json(NormalizationResponse {
            success: true,
            result: None,
            preview: None,
            errors: vec![],
        }),
        Err(e) => e.to_http_response(),
    }
}

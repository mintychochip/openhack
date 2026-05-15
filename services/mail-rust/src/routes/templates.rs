use crate::models::template::{
    TemplateCreate, TemplateListQuery, TemplatePreviewRequest, TemplateUpdate,
};
use crate::services::template::TemplateService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_template(
    pool: web::Data<PgPool>,
    body: web::Json<TemplateCreate>,
) -> HttpResponse {
    match TemplateService::create_template(pool.get_ref(), &body.into_inner()).await {
        Ok(tmpl) => HttpResponse::Created().json(tmpl),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_templates(
    pool: web::Data<PgPool>,
    query: web::Query<TemplateListQuery>,
) -> HttpResponse {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    match TemplateService::list_templates(pool.get_ref(), limit, offset).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_template(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match TemplateService::get_template(pool.get_ref(), id).await {
        Ok(tmpl) => HttpResponse::Ok().json(tmpl),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_template(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<TemplateUpdate>,
) -> HttpResponse {
    let id = path.into_inner();
    match TemplateService::update_template(pool.get_ref(), id, &body.into_inner()).await {
        Ok(tmpl) => HttpResponse::Ok().json(tmpl),
        Err(e) => e.to_http_response(),
    }
}

pub async fn delete_template(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match TemplateService::delete_template(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Template not found"}))
        }
        Err(e) => e.to_http_response(),
    }
}

pub async fn preview_template(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<TemplatePreviewRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    match TemplateService::preview_template(pool.get_ref(), id, &body.into_inner()).await {
        Ok(preview) => HttpResponse::Ok().json(preview),
        Err(e) => e.to_http_response(),
    }
}

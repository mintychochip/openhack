use crate::services::judging::JudgingService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

pub async fn get_dashboard(pool: web::Data<PgPool>) -> HttpResponse {
    match JudgingService::get_dashboard(pool.get_ref()).await {
        Ok(d) => HttpResponse::Ok().json(d),
        Err(e) => e.to_http_response(),
    }
}

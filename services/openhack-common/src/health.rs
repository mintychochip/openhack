use actix_web::{web, HttpResponse};
use sqlx::postgres::PgPool;

#[must_use]
pub fn health_check(service_name: &str) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": service_name
    }))
}

pub async fn readiness_check(pool: web::Data<PgPool>, service_name: &str) -> HttpResponse {
    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": service_name
        })),
        Err(e) => {
            log::error!("Readiness check failed: {e}");
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "not_ready",
                "service": service_name,
                "error": "Database unavailable"
            }))
        }
    }
}

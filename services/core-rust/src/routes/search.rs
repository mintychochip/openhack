use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::models::search::SearchQuery;
use crate::services::search::SearchService;

/// GET /api/core/search — Global search across projects, teams, users.
pub async fn search(
    pool: web::Data<PgPool>,
    query: web::Query<SearchQuery>,
) -> HttpResponse {
    match SearchService::search(pool.get_ref(), &query).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "search_error"})),
    }
}

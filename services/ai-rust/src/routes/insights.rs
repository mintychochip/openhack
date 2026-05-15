use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::config::Config;
use crate::models::insights::InsightGenerateRequest;
use crate::services::insights;

/// Get cached insights for a given period.
///
/// # Expected Behavior
///
/// Accepts an optional `period` query parameter (defaults to "daily").
/// Reads cached insights from Redis if available; returns an empty list
/// if Redis is unavailable or no insights are cached. Returns 200 with
/// an `InsightResponse`.
///
/// # Errors
///
/// Returns 500 on Redis error.
///
/// # Side Effects
///
/// - Reads from Redis via service layer (read-only; skipped if Redis is unavailable).
pub async fn get_insights(
    redis_conn: web::Data<Option<redis::aio::MultiplexedConnection>>,
    query: web::Query<InsightQueryParams>,
) -> HttpResponse {
    let period = query.period.clone().unwrap_or_else(|| "daily".to_string());
    let redis = redis_conn.get_ref().clone();

    match insights::get_insights(redis, &period).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Get insights error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get insights"
            }))
        }
    }
}

/// Generate new insights using the LLM.
///
/// # Expected Behavior
///
/// Accepts a JSON `InsightGenerateRequest` body. Delegates to
/// `insights::generate_insights` which queries the knowledge base,
/// calls the LLM, caches the result in Redis if available, and returns
/// an `InsightResponse`.
///
/// # Errors
///
/// Returns 500 on database, LLM, or Redis errors.
///
/// # Side Effects
///
/// - Reads from `ai.knowledge` via service layer (database I/O).
/// - Calls the configured LLM provider API via service layer (network I/O).
/// - Writes to Redis via service layer (cache write; skipped if Redis is unavailable).
pub async fn generate_insights(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<redis::aio::MultiplexedConnection>>,
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    req: web::Json<InsightGenerateRequest>,
) -> HttpResponse {
    let redis = redis_conn.get_ref().clone();
    match insights::generate_insights(
        pool.get_ref(),
        redis,
        http_client.get_ref(),
        config.get_ref(),
        &req.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Generate insights error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate insights"
            }))
        }
    }
}

/// Query parameters for the insights GET endpoint.
///
/// # Expected Behavior
///
/// Period is optional and defaults to "daily". Valid values include
/// "daily", "weekly", "monthly".
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct InsightQueryParams {
    pub period: Option<String>,
}

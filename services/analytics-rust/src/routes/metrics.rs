use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::models::metric::{MetricsByCategoryParams, MetricsOverTimeParams};
use crate::services::aggregator;

/// Get summary metrics.
///
/// # Expected Behavior
///
/// Returns aggregate counts: `total_registrations`, `total_teams`, `total_projects`,
/// `total_submissions`, and `active_users` (distinct users with events in the
/// last 30 days).
///
/// # Errors
///
/// Returns 500 if any database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn get_metrics(pool: web::Data<PgPool>) -> HttpResponse {
    match aggregator::get_summary(pool.get_ref()).await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(e) => {
            log::error!("Get metrics error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get metrics"
            }))
        }
    }
}

/// Get time-bucketed metrics over a given period.
///
/// # Expected Behavior
///
/// Accepts query params: `period` ("hourly"|"daily", default "daily"),
/// `days` (default 7), `metric_type` ("registrations"|"teams"|"projects"|
/// "submissions", default "registrations"). Returns time-bucketed counts.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn metrics_over_time(
    pool: web::Data<PgPool>,
    query: web::Query<MetricsOverTimeParams>,
) -> HttpResponse {
    match aggregator::get_metrics_over_time(pool.get_ref(), &query.into_inner()).await {
        Ok(data) => HttpResponse::Ok().json(serde_json::json!({ "data": data })),
        Err(e) => {
            log::error!("Metrics over time error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get metrics over time"
            }))
        }
    }
}

/// Get category breakdown for a given metric.
///
/// # Expected Behavior
///
/// Accepts query param: `metric_name` (default "projects"). Returns a list
/// of {category, count} objects ordered by count descending.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn metrics_by_category(
    pool: web::Data<PgPool>,
    query: web::Query<MetricsByCategoryParams>,
) -> HttpResponse {
    match aggregator::get_metrics_by_category(pool.get_ref(), &query.into_inner()).await {
        Ok(data) => HttpResponse::Ok().json(serde_json::json!({ "data": data })),
        Err(e) => {
            log::error!("Metrics by category error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get metrics by category"
            }))
        }
    }
}

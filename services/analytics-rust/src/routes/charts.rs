use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::models::chart::{CategoryDistributionParams, HeatmapQueryParams, TimelineQueryParams};
use crate::services::charts;

/// Get the 4-stage registration funnel chart data.
///
/// # Expected Behavior
///
/// Returns a `FunnelResponse` with four stages: Registered → Joined Team →
/// Created Project → Submitted. Each stage includes the count and conversion
/// rate from the previous stage. The first stage always has a 100% conversion rate.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn registration_funnel(pool: web::Data<PgPool>) -> HttpResponse {
    match charts::get_registration_funnel(pool.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Registration funnel error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate registration funnel"
            }))
        }
    }
}

/// Get the submission timeline chart data.
///
/// # Expected Behavior
///
/// Accepts optional query params: `days` (default 7), `period` ("hourly"|"daily",
/// default "daily"). Returns time-bucketed submission counts with peak detection
/// (the bucket with the highest count is marked `is_peak = true`).
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn submission_timeline(
    pool: web::Data<PgPool>,
    query: web::Query<TimelineQueryParams>,
) -> HttpResponse {
    match charts::get_submission_timeline(pool.get_ref(), &query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Submission timeline error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate submission timeline"
            }))
        }
    }
}

/// Get the category distribution pie chart data.
///
/// # Expected Behavior
///
/// Accepts optional query param: `category_field` (default "category").
/// Returns pie chart data with labels, values, and colors from the
/// `CHART_COLORS` palette.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn category_distribution(
    pool: web::Data<PgPool>,
    query: web::Query<CategoryDistributionParams>,
) -> HttpResponse {
    match charts::get_category_distribution(pool.get_ref(), &query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Category distribution error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate category distribution"
            }))
        }
    }
}

/// Get the engagement heatmap data.
///
/// # Expected Behavior
///
/// Accepts optional query param: `days` (default 30). Returns a day×hour
/// heatmap of user activity as a list of cells with day (0–6), hour (0–23),
/// and count.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn engagement_heatmap(
    pool: web::Data<PgPool>,
    query: web::Query<HeatmapQueryParams>,
) -> HttpResponse {
    match charts::get_engagement_heatmap(pool.get_ref(), &query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Engagement heatmap error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate engagement heatmap"
            }))
        }
    }
}

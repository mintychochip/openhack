use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::models::event::EventExportParams;
use crate::models::metric::MetricExportParams;
use crate::models::report::ReportExportParams;
use crate::services::export;

/// Export events as CSV.
///
/// # Expected Behavior
///
/// Accepts query params: `limit` (default 10000), `event_type` (optional filter).
/// Returns a CSV file with Content-Type text/csv and Content-Disposition header.
///
/// # Errors
///
/// Returns 500 if the database query or CSV generation fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
/// - Returns CSV content in the response body.
pub async fn export_events_csv(
    pool: web::Data<PgPool>,
    query: web::Query<EventExportParams>,
) -> HttpResponse {
    match export::export_events_csv(pool.get_ref(), &query.into_inner()).await {
        Ok(csv_content) => HttpResponse::Ok()
            .content_type("text/csv")
            .insert_header(("Content-Disposition", "attachment; filename=\"events.csv\""))
            .body(csv_content),
        Err(e) => {
            log::error!("Export events CSV error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to export events as CSV"
            }))
        }
    }
}

/// Export events as JSON.
///
/// # Expected Behavior
///
/// Accepts query params: `limit` (default 10000), `event_type` (optional filter).
/// Returns a JSON array of event objects.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.events` via service layer (database I/O, read-only).
pub async fn export_events_json(
    pool: web::Data<PgPool>,
    query: web::Query<EventExportParams>,
) -> HttpResponse {
    match export::export_events_json(pool.get_ref(), &query.into_inner()).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            log::error!("Export events JSON error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to export events as JSON"
            }))
        }
    }
}

/// Export metrics as CSV.
///
/// # Expected Behavior
///
/// Accepts query params: `limit` (default 10000), `metric_name` (optional filter).
/// Returns a CSV file with Content-Type text/csv and Content-Disposition header.
///
/// # Errors
///
/// Returns 500 if the database query or CSV generation fails.
///
/// # Side Effects
///
/// - Reads from `analytics.metrics` via service layer (database I/O, read-only).
/// - Returns CSV content in the response body.
pub async fn export_metrics_csv(
    pool: web::Data<PgPool>,
    query: web::Query<MetricExportParams>,
) -> HttpResponse {
    match export::export_metrics_csv(pool.get_ref(), &query.into_inner()).await {
        Ok(csv_content) => HttpResponse::Ok()
            .content_type("text/csv")
            .insert_header((
                "Content-Disposition",
                "attachment; filename=\"metrics.csv\"",
            ))
            .body(csv_content),
        Err(e) => {
            log::error!("Export metrics CSV error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to export metrics as CSV"
            }))
        }
    }
}

/// Export metrics as JSON.
///
/// # Expected Behavior
///
/// Accepts query params: `limit` (default 10000), `metric_name` (optional filter).
/// Returns a JSON array of metric objects.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.metrics` via service layer (database I/O, read-only).
pub async fn export_metrics_json(
    pool: web::Data<PgPool>,
    query: web::Query<MetricExportParams>,
) -> HttpResponse {
    match export::export_metrics_json(pool.get_ref(), &query.into_inner()).await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => {
            log::error!("Export metrics JSON error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to export metrics as JSON"
            }))
        }
    }
}

/// Export reports as JSON.
///
/// # Expected Behavior
///
/// Accepts query params: `limit` (default 100), `report_type` (optional filter).
/// Returns a JSON array of report objects.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.reports` via service layer (database I/O, read-only).
pub async fn export_reports_json(
    pool: web::Data<PgPool>,
    query: web::Query<ReportExportParams>,
) -> HttpResponse {
    match export::export_reports_json(pool.get_ref(), &query.into_inner()).await {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(e) => {
            log::error!("Export reports JSON error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to export reports as JSON"
            }))
        }
    }
}

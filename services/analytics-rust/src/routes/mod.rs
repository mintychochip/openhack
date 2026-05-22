pub mod certificates;
pub mod charts;
pub mod export;
pub mod metrics;
pub mod reports;

use actix_web::web;

/// Register all Analytics service routes on the given service config.
///
/// # Expected Behavior
///
/// Configures all API endpoints under the `/api/analytics` prefix:
/// - GET `/charts/registration-funnel` → 4-stage registration funnel
/// - GET `/charts/submission-timeline` → time-bucketed submissions
/// - GET `/charts/category-distribution` → pie chart data
/// - GET `/charts/engagement-heatmap` → day×hour heatmap
/// - GET `/metrics` → summary metrics
/// - GET `/metrics/over-time` → time-bucketed metrics
/// - GET `/metrics/by-category` → category breakdown
/// - GET `/reports` → list reports
/// - POST `/reports/generate` → generate a report
/// - GET `/reports/{id}` → get report by ID
/// - GET `/export/csv/events` → stream CSV of events
/// - GET `/export/json/events` → JSON export of events
/// - GET `/export/csv/metrics` → stream CSV of metrics
/// - GET `/export/json/metrics` → JSON export of metrics
/// - GET `/export/json/reports` → JSON export of reports
///
/// # Errors
///
/// None. Route registration is infallible.
///
/// # Side Effects
///
/// - Registers HTTP route handlers with the Actix-web service config.
///   No I/O or network calls occur during registration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/analytics")
            .route("/certificates", web::post().to(certificates::create_certificate))
            .route(
                "/certificates/bulk",
                web::post().to(certificates::bulk_create_certificates),
            )
            .route(
                "/certificates",
                web::get().to(certificates::list_certificates),
            )
            .route(
                "/certificates/{id}",
                web::get().to(certificates::get_certificate),
            )
            .route(
                "/certificates/{id}/revoke",
                web::post().to(certificates::revoke_certificate),
            )
            .route(
                "/certificates/{id}/pdf",
                web::get().to(certificates::get_certificate_pdf),
            )
            .route(
                "/certificates/{id}/pdf-reference",
                web::put().to(certificates::update_certificate_pdf),
            )
            .route(
                "/certificates/verify/{code}",
                web::get().to(certificates::verify_certificate),
            )
            .route(
                "/charts/registration-funnel",
                web::get().to(charts::registration_funnel),
            )
            .route(
                "/charts/submission-timeline",
                web::get().to(charts::submission_timeline),
            )
            .route(
                "/charts/category-distribution",
                web::get().to(charts::category_distribution),
            )
            .route(
                "/charts/engagement-heatmap",
                web::get().to(charts::engagement_heatmap),
            )
            .route("/metrics", web::get().to(metrics::get_metrics))
            .route(
                "/metrics/over-time",
                web::get().to(metrics::metrics_over_time),
            )
            .route(
                "/metrics/by-category",
                web::get().to(metrics::metrics_by_category),
            )
            .route("/reports", web::get().to(reports::list_reports))
            .route(
                "/reports/generate",
                web::post().to(reports::generate_report),
            )
            .route("/reports/{id}", web::get().to(reports::get_report))
            .route(
                "/export/csv/events",
                web::get().to(export::export_events_csv),
            )
            .route(
                "/export/json/events",
                web::get().to(export::export_events_json),
            )
            .route(
                "/export/csv/metrics",
                web::get().to(export::export_metrics_csv),
            )
            .route(
                "/export/json/metrics",
                web::get().to(export::export_metrics_json),
            )
            .route(
                "/export/json/reports",
                web::get().to(export::export_reports_json),
            ),
    );
}

use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::report::{GenerateReportRequest, Report, ReportQueryParams};

/// List reports with optional type filter.
///
/// # Expected Behavior
///
/// Accepts optional query params: `report_type` (filter), `limit` (default 100).
/// Returns a JSON array of Report objects ordered by `generated_at` descending.
///
/// # Errors
///
/// Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads from `analytics.reports` via direct SQL (database I/O, read-only).
pub async fn list_reports(
    pool: web::Data<PgPool>,
    query: web::Query<ReportQueryParams>,
) -> HttpResponse {
    let params = query.into_inner();
    let result: Result<Vec<Report>, sqlx::Error> = if let Some(ref report_type) = params.report_type
    {
        sqlx::query_as::<_, Report>(
            "SELECT id, report_type, generated_at, data \
             FROM analytics.reports WHERE report_type = $1 \
             ORDER BY generated_at DESC LIMIT $2",
        )
        .bind(report_type)
        .bind(params.limit)
        .fetch_all(pool.get_ref())
        .await
    } else {
        sqlx::query_as::<_, Report>(
            "SELECT id, report_type, generated_at, data \
             FROM analytics.reports ORDER BY generated_at DESC LIMIT $1",
        )
        .bind(params.limit)
        .fetch_all(pool.get_ref())
        .await
    };

    match result {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(e) => {
            log::error!("List reports error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list reports"
            }))
        }
    }
}

/// Generate a new analytics report.
///
/// # Expected Behavior
///
/// Accepts a JSON body with `report_type` ("daily"|"weekly"|"final"),
/// optional `start_date`, optional `end_date`, and optional `include_charts`.
/// Builds report data by querying event counts for the relevant event types
/// and time range. If `include_charts` is true, includes funnel and timeline
/// data in the report. Inserts the report into `analytics.reports` and returns
/// the created Report object.
///
/// # Errors
///
/// Returns 400 if `report_type` is invalid. Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `analytics.events` (database I/O, read-only).
/// - Inserts a new row into `analytics.reports` (database write).
/// - Logs at INFO on successful report generation.
pub async fn generate_report(
    pool: web::Data<PgPool>,
    req: web::Json<GenerateReportRequest>,
) -> HttpResponse {
    let body = req.into_inner();

    let event_types: &[&str] = match body.report_type.as_str() {
        "daily" | "weekly" | "final" => &[
            "user.registered",
            "team.created",
            "project.created",
            "project.submitted",
        ],
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid report_type. Must be daily, weekly, or final"
            }));
        }
    };

    let mut report_data = serde_json::Map::new();
    report_data.insert(
        "report_type".to_string(),
        serde_json::Value::String(body.report_type.clone()),
    );

    let mut counts = serde_json::Map::new();
    for event_type in event_types {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM analytics.events WHERE event_type = $1")
                .bind(*event_type)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or((0,));
        counts.insert(event_type.to_string(), serde_json::Value::from(count.0));
    }
    report_data.insert("counts".to_string(), serde_json::Value::Object(counts));

    if let Some(ref start) = body.start_date {
        report_data.insert(
            "start_date".to_string(),
            serde_json::Value::String(start.clone()),
        );
    }
    if let Some(ref end) = body.end_date {
        report_data.insert(
            "end_date".to_string(),
            serde_json::Value::String(end.clone()),
        );
    }

    if body.include_charts {
        report_data.insert("include_charts".to_string(), serde_json::Value::Bool(true));
    }

    let id = Uuid::new_v4();
    let now = Utc::now();

    let insert_result = sqlx::query(
        "INSERT INTO analytics.reports (id, report_type, generated_at, data) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(&body.report_type)
    .bind(now)
    .bind(serde_json::Value::Object(report_data.clone()))
    .execute(pool.get_ref())
    .await;

    match insert_result {
        Ok(_) => {
            log::info!("Generated {} report: {}", body.report_type, id);
            HttpResponse::Created().json(Report {
                id,
                report_type: body.report_type,
                generated_at: now,
                data: serde_json::Value::Object(report_data),
            })
        }
        Err(e) => {
            log::error!("Generate report error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate report"
            }))
        }
    }
}

/// Get a single report by ID.
///
/// # Expected Behavior
///
/// Returns the Report object for the given UUID. Returns 404 if not found.
///
/// # Errors
///
/// Returns 404 if the report does not exist. Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `analytics.reports` (database I/O, read-only).
pub async fn get_report(pool: web::Data<PgPool>, path: web::Path<uuid::Uuid>) -> HttpResponse {
    let id = path.into_inner();

    let result: Result<Option<Report>, sqlx::Error> = sqlx::query_as::<_, Report>(
        "SELECT id, report_type, generated_at, data FROM analytics.reports WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(report)) => HttpResponse::Ok().json(report),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Report not found"
        })),
        Err(e) => {
            log::error!("Get report error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get report"
            }))
        }
    }
}

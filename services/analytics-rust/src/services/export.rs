use sqlx::PgPool;

use crate::errors::AnalyticsError;
use crate::models::event::{Event, EventExportParams};
use crate::models::metric::{Metric, MetricExportParams};
use crate::models::report::{Report, ReportExportParams};

/// Export events as CSV string.
///
/// # Expected Behavior
///
/// Queries the `analytics.events` table with optional `event_type` filter
/// and `limit`. Builds a CSV string with columns:
/// id, `event_type`, `event_data`, `user_id`, `team_id`, `project_id`, `occurred_at`.
/// Returns the CSV content as a String.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
/// - `AnalyticsError::CsvExport` if CSV serialization fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
/// - Allocates memory for CSV content proportional to result size.
pub async fn export_events_csv(
    pool: &PgPool,
    params: &EventExportParams,
) -> Result<String, AnalyticsError> {
    let events: Vec<Event> = if let Some(ref event_type) = params.event_type {
        sqlx::query_as::<_, Event>(
            "SELECT id, event_type, event_data, user_id, team_id, project_id, occurred_at \
             FROM analytics.events WHERE event_type = $1 ORDER BY occurred_at DESC LIMIT $2",
        )
        .bind(event_type)
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Event>(
            "SELECT id, event_type, event_data, user_id, team_id, project_id, occurred_at \
             FROM analytics.events ORDER BY occurred_at DESC LIMIT $1",
        )
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for event in &events {
        wtr.write_record(&[
            event.id.to_string(),
            event.event_type.clone(),
            serde_json::to_string(&event.event_data).unwrap_or_default(),
            event.user_id.map(|u| u.to_string()).unwrap_or_default(),
            event.team_id.map(|t| t.to_string()).unwrap_or_default(),
            event.project_id.map(|p| p.to_string()).unwrap_or_default(),
            event.occurred_at.to_rfc3339(),
        ])
        .map_err(|e| AnalyticsError::CsvExport(e.to_string()))?;
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| AnalyticsError::CsvExport(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| AnalyticsError::CsvExport(e.to_string()))
}

/// Export events as JSON.
///
/// # Expected Behavior
///
/// Queries the `analytics.events` table with optional `event_type` filter
/// and `limit`. Returns a serialized JSON array of Event objects.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn export_events_json(
    pool: &PgPool,
    params: &EventExportParams,
) -> Result<Vec<Event>, AnalyticsError> {
    let events: Vec<Event> = if let Some(ref event_type) = params.event_type {
        sqlx::query_as::<_, Event>(
            "SELECT id, event_type, event_data, user_id, team_id, project_id, occurred_at \
             FROM analytics.events WHERE event_type = $1 ORDER BY occurred_at DESC LIMIT $2",
        )
        .bind(event_type)
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Event>(
            "SELECT id, event_type, event_data, user_id, team_id, project_id, occurred_at \
             FROM analytics.events ORDER BY occurred_at DESC LIMIT $1",
        )
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    };

    Ok(events)
}

/// Export metrics as CSV string.
///
/// # Expected Behavior
///
/// Queries the `analytics.metrics` table with optional `metric_name` filter
/// and `limit`. Builds a CSV string with columns:
/// id, `metric_name`, `metric_value`, dimensions, `recorded_at`.
/// Returns the CSV content as a String.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
/// - `AnalyticsError::CsvExport` if CSV serialization fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
/// - Allocates memory for CSV content proportional to result size.
pub async fn export_metrics_csv(
    pool: &PgPool,
    params: &MetricExportParams,
) -> Result<String, AnalyticsError> {
    let metrics: Vec<Metric> = if let Some(ref metric_name) = params.metric_name {
        sqlx::query_as::<_, Metric>(
            "SELECT id, metric_name, metric_value, dimensions, recorded_at \
             FROM analytics.metrics WHERE metric_name = $1 ORDER BY recorded_at DESC LIMIT $2",
        )
        .bind(metric_name)
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Metric>(
            "SELECT id, metric_name, metric_value, dimensions, recorded_at \
             FROM analytics.metrics ORDER BY recorded_at DESC LIMIT $1",
        )
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for metric in &metrics {
        wtr.write_record(&[
            metric.id.to_string(),
            metric.metric_name.clone(),
            metric
                .metric_value
                .map(|v| v.to_string())
                .unwrap_or_default(),
            serde_json::to_string(&metric.dimensions).unwrap_or_default(),
            metric.recorded_at.to_rfc3339(),
        ])
        .map_err(|e| AnalyticsError::CsvExport(e.to_string()))?;
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| AnalyticsError::CsvExport(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| AnalyticsError::CsvExport(e.to_string()))
}

/// Export metrics as JSON.
///
/// # Expected Behavior
///
/// Queries the `analytics.metrics` table with optional `metric_name` filter
/// and `limit`. Returns a serialized JSON array of Metric objects.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn export_metrics_json(
    pool: &PgPool,
    params: &MetricExportParams,
) -> Result<Vec<Metric>, AnalyticsError> {
    let metrics: Vec<Metric> = if let Some(ref metric_name) = params.metric_name {
        sqlx::query_as::<_, Metric>(
            "SELECT id, metric_name, metric_value, dimensions, recorded_at \
             FROM analytics.metrics WHERE metric_name = $1 ORDER BY recorded_at DESC LIMIT $2",
        )
        .bind(metric_name)
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Metric>(
            "SELECT id, metric_name, metric_value, dimensions, recorded_at \
             FROM analytics.metrics ORDER BY recorded_at DESC LIMIT $1",
        )
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    };

    Ok(metrics)
}

/// Export reports as JSON.
///
/// # Expected Behavior
///
/// Queries the `analytics.reports` table with optional `report_type` filter
/// and `limit`. Returns a serialized JSON array of Report objects.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn export_reports_json(
    pool: &PgPool,
    params: &ReportExportParams,
) -> Result<Vec<Report>, AnalyticsError> {
    let reports: Vec<Report> = if let Some(ref report_type) = params.report_type {
        sqlx::query_as::<_, Report>(
            "SELECT id, report_type, generated_at, data \
             FROM analytics.reports WHERE report_type = $1 ORDER BY generated_at DESC LIMIT $2",
        )
        .bind(report_type)
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Report>(
            "SELECT id, report_type, generated_at, data \
             FROM analytics.reports ORDER BY generated_at DESC LIMIT $1",
        )
        .bind(params.limit)
        .fetch_all(pool)
        .await?
    };

    Ok(reports)
}

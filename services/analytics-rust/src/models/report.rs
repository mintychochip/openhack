use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A generated analytics report from the `analytics.reports` table.
///
/// # Expected Behavior
///
/// Represents a single report with an ID, type, generation timestamp,
/// and structured data payload as JSON.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[allow(clippy::struct_field_names)]
pub struct Report {
    pub id: Uuid,
    pub report_type: String,
    pub generated_at: DateTime<Utc>,
    pub data: Value,
}

/// Query parameters for the report listing endpoint.
///
/// # Expected Behavior
///
/// `report_type` is optional and filters reports by type when provided.
/// `limit` defaults to 100.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct ReportQueryParams {
    pub report_type: Option<String>,
    #[serde(default = "default_report_limit")]
    pub limit: i64,
}

fn default_report_limit() -> i64 {
    100
}

/// Request body for the report generation endpoint.
///
/// # Expected Behavior
///
/// `report_type` must be "daily", "weekly", or "final". `start_date` and
/// `end_date` are optional ISO date strings. `include_charts` is optional
/// and defaults to false.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub include_charts: bool,
}

/// Query parameters for the report export endpoint.
///
/// # Expected Behavior
///
/// `limit` defaults to 100. `report_type` is optional.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct ReportExportParams {
    #[serde(default = "default_report_export_limit")]
    pub limit: i64,
    pub report_type: Option<String>,
}

fn default_report_export_limit() -> i64 {
    100
}

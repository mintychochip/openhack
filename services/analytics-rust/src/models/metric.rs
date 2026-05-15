use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A recorded analytics metric from the `analytics.metrics` table.
///
/// # Expected Behavior
///
/// Represents a single metric with an ID, name, optional numeric value,
/// optional dimensions as JSON, and the recording timestamp.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[allow(clippy::struct_field_names)]
pub struct Metric {
    pub id: Uuid,
    pub metric_name: String,
    pub metric_value: Option<f64>,
    pub dimensions: Value,
    pub recorded_at: DateTime<Utc>,
}

/// Query parameters for metric export endpoints.
///
/// # Expected Behavior
///
/// `limit` defaults to 10000. `metric_name` is optional and filters
/// metrics by name when provided.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricExportParams {
    #[serde(default = "default_metric_limit")]
    pub limit: i64,
    pub metric_name: Option<String>,
}

fn default_metric_limit() -> i64 {
    10000
}

/// Response for the summary metrics endpoint.
///
/// # Expected Behavior
///
/// Contains aggregate counts of registrations, teams, projects,
/// submissions, and active users.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummary {
    pub total_registrations: i64,
    pub total_teams: i64,
    pub total_projects: i64,
    pub total_submissions: i64,
    pub active_users: i64,
}

/// Query parameters for the over-time metrics endpoint.
///
/// # Expected Behavior
///
/// `period` must be "hourly" or "daily" (defaults to "daily").
/// `days` defaults to 7. `metric_type` must be one of
/// "registrations", "teams", "projects", "submissions"
/// (defaults to "registrations").
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsOverTimeParams {
    #[serde(default = "default_period")]
    pub period: String,
    #[serde(default = "default_days")]
    pub days: i64,
    #[serde(default = "default_metric_type")]
    pub metric_type: String,
}

fn default_period() -> String {
    "daily".to_string()
}

fn default_days() -> i64 {
    7
}

fn default_metric_type() -> String {
    "registrations".to_string()
}

/// A single time bucket with a label and count.
///
/// # Expected Behavior
///
/// Used in over-time and timeline responses. `bucket` is the ISO-formatted
/// timestamp of the bucket start. `count` is the number of events in that bucket.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct TimeBucket {
    pub bucket: String,
    pub count: i64,
}

/// Query parameters for the by-category metrics endpoint.
///
/// # Expected Behavior
///
/// `metric_name` defaults to "projects". Valid values include "projects",
/// "submissions", "registrations", "teams".
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsByCategoryParams {
    #[serde(default = "default_metric_name")]
    pub metric_name: String,
}

fn default_metric_name() -> String {
    "projects".to_string()
}

/// A category with a name and count.
///
/// # Expected Behavior
///
/// Used in category breakdown responses.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

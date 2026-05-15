use serde::{Deserialize, Serialize};

/// Chart color palette used across all chart endpoints.
///
/// # Expected Behavior
///
/// Returns a static array of 8 hex color strings used for chart rendering.
///
/// # Side Effects
///
/// None. Pure computation.
pub const CHART_COLORS: [&str; 8] = [
    "#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16",
];

/// A single funnel stage with name, count, and conversion rate.
///
/// # Expected Behavior
///
/// `name` is the stage label. `count` is the absolute number of items
/// at this stage. `conversion_rate` is the percentage from the previous
/// stage (0.0–100.0), or 100.0 for the first stage.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct FunnelStage {
    pub name: String,
    pub count: i64,
    pub conversion_rate: f64,
}

/// Response for the registration funnel chart endpoint.
///
/// # Expected Behavior
///
/// Contains an ordered list of funnel stages with counts and conversion rates.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct FunnelResponse {
    pub stages: Vec<FunnelStage>,
}

/// A single time-bucketed data point with optional peak flag.
///
/// # Expected Behavior
///
/// `bucket` is the ISO-formatted timestamp of the bucket start.
/// `count` is the number of events. `is_peak` is true if this bucket
/// has the highest count in the dataset.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct TimelinePoint {
    pub bucket: String,
    pub count: i64,
    pub is_peak: bool,
}

/// Response for the submission timeline chart endpoint.
///
/// # Expected Behavior
///
/// Contains an ordered list of timeline data points with peak detection.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct TimelineResponse {
    pub data: Vec<TimelinePoint>,
}

/// Query parameters for the submission timeline endpoint.
///
/// # Expected Behavior
///
/// `days` defaults to 7. `period` must be "hourly" or "daily"
/// (defaults to "daily").
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct TimelineQueryParams {
    #[serde(default = "default_timeline_days")]
    pub days: i64,
    #[serde(default = "default_timeline_period")]
    pub period: String,
}

fn default_timeline_days() -> i64 {
    7
}

fn default_timeline_period() -> String {
    "daily".to_string()
}

/// A single slice for a pie/donut chart.
///
/// # Expected Behavior
///
/// `label` is the category name. `value` is the count. `color` is assigned
/// from the chart palette cycling through `CHART_COLORS`.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct PieSlice {
    pub label: String,
    pub value: i64,
    pub color: String,
}

/// Response for the category distribution chart endpoint.
///
/// # Expected Behavior
///
/// Contains an ordered list of pie chart slices.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryDistributionResponse {
    pub data: Vec<PieSlice>,
}

/// Query parameters for the category distribution endpoint.
///
/// # Expected Behavior
///
/// `category_field` defaults to "category". Determines which JSONB key
/// in `event_data` is used for grouping.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryDistributionParams {
    #[serde(default = "default_category_field")]
    pub category_field: String,
}

fn default_category_field() -> String {
    "category".to_string()
}

/// A single cell in the engagement heatmap.
///
/// # Expected Behavior
///
/// `day` is 0–6 (Sunday–Saturday). `hour` is 0–23. `count` is the
/// number of events at that day×hour intersection.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapCell {
    pub day: i32,
    pub hour: i32,
    pub count: i64,
}

/// Response for the engagement heatmap endpoint.
///
/// # Expected Behavior
///
/// Contains an ordered list of heatmap cells.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapResponse {
    pub data: Vec<HeatmapCell>,
}

/// Query parameters for the engagement heatmap endpoint.
///
/// # Expected Behavior
///
/// `days` defaults to 30. Determines how many days back to look
/// for user activity.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct HeatmapQueryParams {
    #[serde(default = "default_heatmap_days")]
    pub days: i64,
}

fn default_heatmap_days() -> i64 {
    30
}

/// Maps a metric type string to its corresponding event type.
///
/// # Expected Behavior
///
/// Returns the event type string for the given metric type.
/// - "registrations" → "user.registered"
/// - "teams" → "team.created"
/// - "projects" → "project.created"
/// - "submissions" → "project.submitted"
///   Returns "user.registered" for any unrecognized metric type.
///
/// # Side Effects
///
/// None. Pure computation.
pub fn metric_type_to_event_type(metric_type: &str) -> &'static str {
    match metric_type {
        "teams" => "team.created",
        "projects" => "project.created",
        "submissions" => "project.submitted",
        _ => "user.registered",
    }
}

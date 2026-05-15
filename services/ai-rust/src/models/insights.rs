use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request body for the insight generation endpoint.
///
/// # Expected Behavior
///
/// Period determines the time window for analysis (e.g., "daily", "weekly").
/// Topic is an optional focus area. Period defaults to "daily".
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct InsightGenerateRequest {
    #[serde(default = "default_period")]
    pub period: String,
    pub topic: Option<String>,
}

fn default_period() -> String {
    "daily".to_string()
}

/// A single AI-generated insight.
///
/// # Expected Behavior
///
/// Contains a unique identifier, the period it covers, a summary string,
/// structured details as JSON, and the generation timestamp.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: String,
    pub period: String,
    pub summary: String,
    pub details: Value,
    pub generated_at: String,
}

/// Response body for the insights endpoint.
///
/// # Expected Behavior
///
/// Contains a list of insights, which may be empty if no insights have
/// been generated for the requested period.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct InsightResponse {
    pub insights: Vec<Insight>,
}

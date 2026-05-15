use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A recorded analytics event from the `analytics.events` table.
///
/// # Expected Behavior
///
/// Represents a single event with an ID, type, optional JSON data payload,
/// optional user/team/project references, and the timestamp of occurrence.
/// Used for both reading from and writing to the database.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[allow(clippy::struct_field_names)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub event_data: Value,
    pub user_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
}

/// Query parameters for CSV/JSON event export.
///
/// # Expected Behavior
///
/// `limit` defaults to 10000. `event_type` is optional.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct EventExportParams {
    #[serde(default = "default_export_limit")]
    pub limit: i64,
    pub event_type: Option<String>,
}

fn default_export_limit() -> i64 {
    10000
}

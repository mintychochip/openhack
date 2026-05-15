use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a leaderboard snapshot.
///
/// # Expected Behavior
///
/// Maps directly to the `leaderboard.snapshots` table. `snapshot_data`
/// is a JSONB column containing the full leaderboard state at the time
/// the snapshot was taken. `formula_id` references the ranking formula
/// that was active when the snapshot was created.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Snapshot {
    pub id: Uuid,
    pub name: String,
    pub phase_id: Option<Uuid>,
    pub formula_id: Option<Uuid>,
    #[sqlx(rename = "snapshot_data")]
    pub data: serde_json::Value,
    pub team_count: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Request body for creating a snapshot.
///
/// # Expected Behavior
///
/// `name` is required. `phase_id` and `formula_id` are optional and
/// associate the snapshot with a specific phase and formula. The
/// `snapshot_data` is generated server-side from the current leaderboard
/// state.
#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotCreate {
    pub name: String,
    pub phase_id: Option<Uuid>,
    pub formula_id: Option<Uuid>,
}

/// Response body for a single snapshot.
///
/// # Expected Behavior
///
/// Contains all snapshot fields. `team_count` defaults to 0 if null.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub id: Uuid,
    pub name: String,
    pub phase_id: Option<Uuid>,
    pub formula_id: Option<Uuid>,
    pub snapshot_data: serde_json::Value,
    pub team_count: i32,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<Snapshot> for SnapshotResponse {
    fn from(s: Snapshot) -> Self {
        Self {
            id: s.id,
            name: s.name,
            phase_id: s.phase_id,
            formula_id: s.formula_id,
            snapshot_data: s.data,
            team_count: s.team_count.unwrap_or(0),
            created_at: s.created_at,
        }
    }
}

/// Response body for a list of snapshots.
///
/// # Expected Behavior
///
/// Contains the list of snapshot responses and a total count for pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotListResponse {
    pub snapshots: Vec<SnapshotResponse>,
    pub total: i64,
}

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a score history entry.
///
/// # Expected Behavior
///
/// Maps directly to the `leaderboard.score_history` table. Each row
/// records a team's score and rank at a specific point in time, typically
/// created during ranking recalculation.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScoreHistory {
    pub id: i64,
    pub team_id: Uuid,
    pub score: Option<Decimal>,
    pub rank: Option<i32>,
    pub recorded_at: Option<DateTime<Utc>>,
}

/// Response body for a single score history entry.
///
/// # Expected Behavior
///
/// Decimal `score` is serialized as a string to preserve precision.
/// Defaults to "0" if null in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreHistoryResponse {
    pub id: i64,
    pub team_id: Uuid,
    pub score: String,
    pub rank: Option<i32>,
    pub recorded_at: Option<DateTime<Utc>>,
}

impl From<ScoreHistory> for ScoreHistoryResponse {
    fn from(h: ScoreHistory) -> Self {
        Self {
            id: h.id,
            team_id: h.team_id,
            score: h.score.unwrap_or(Decimal::ZERO).to_string(),
            rank: h.rank,
            recorded_at: h.recorded_at,
        }
    }
}

/// Response body for a list of score history entries.
///
/// # Expected Behavior
///
/// Contains the list of history responses and a total count for pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreHistoryListResponse {
    pub history: Vec<ScoreHistoryResponse>,
    pub total: i64,
}

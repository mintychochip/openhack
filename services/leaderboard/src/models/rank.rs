use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a leaderboard rank entry.
///
/// # Expected Behavior
///
/// Maps directly to the `leaderboard.ranks` table. `total_score` represents
/// the judge/expert score. `combined_score` is the result of the active
/// ranking formula. `weighted_votes` aggregates vote weights. `rank` and
/// `previous_rank` track ranking movement. `is_frozen` locks a team's
/// ranking when the phase ends.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Rank {
    pub team_id: Uuid,
    pub total_score: Option<Decimal>,
    pub public_votes: Option<i32>,
    #[sqlx(rename = "rank")]
    pub position: Option<i32>,
    #[sqlx(rename = "previous_rank")]
    pub previous: Option<i32>,
    pub last_updated: Option<DateTime<Utc>>,
    pub is_frozen: Option<bool>,
    pub frozen_at: Option<DateTime<Utc>>,
    pub phase_id: Option<Uuid>,
    pub combined_score: Option<Decimal>,
    pub formula_id: Option<Uuid>,
    pub weighted_votes: Option<Decimal>,
    pub tiebreaker_score: Option<Decimal>,
}

/// Response body for a single rank entry.
///
/// # Expected Behavior
///
/// Decimal fields are serialized as strings to preserve precision.
/// Boolean and integer defaults are applied for nullable columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankResponse {
    pub team_id: Uuid,
    pub total_score: String,
    pub public_votes: i32,
    pub rank: Option<i32>,
    pub previous_rank: Option<i32>,
    pub last_updated: Option<DateTime<Utc>>,
    pub is_frozen: bool,
    pub frozen_at: Option<DateTime<Utc>>,
    pub phase_id: Option<Uuid>,
    pub combined_score: Option<String>,
    pub formula_id: Option<Uuid>,
    pub weighted_votes: String,
    pub tiebreaker_score: Option<String>,
}

impl From<Rank> for RankResponse {
    fn from(r: Rank) -> Self {
        Self {
            team_id: r.team_id,
            total_score: r.total_score.unwrap_or(Decimal::ZERO).to_string(),
            public_votes: r.public_votes.unwrap_or(0),
            rank: r.position,
            previous_rank: r.previous,
            last_updated: r.last_updated,
            is_frozen: r.is_frozen.unwrap_or(false),
            frozen_at: r.frozen_at,
            phase_id: r.phase_id,
            combined_score: r.combined_score.map(|d| d.to_string()),
            formula_id: r.formula_id,
            weighted_votes: r.weighted_votes.unwrap_or(Decimal::ZERO).to_string(),
            tiebreaker_score: r.tiebreaker_score.map(|d| d.to_string()),
        }
    }
}

/// Response body for the leaderboard list.
///
/// # Expected Behavior
///
/// Contains the list of rank entries sorted by rank and a total count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub leaderboard: Vec<RankResponse>,
    pub total: i64,
}

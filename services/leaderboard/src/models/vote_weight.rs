use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a vote weight by role.
///
/// # Expected Behavior
///
/// Maps directly to the `leaderboard.vote_weights` table. Each row defines
/// a vote weight multiplier for a specific voter role. The `voter_role`
/// column has a UNIQUE constraint ensuring one weight per role.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct VoteWeight {
    pub id: Uuid,
    pub voter_role: String,
    pub weight: Decimal,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Response body for a vote weight entry.
///
/// # Expected Behavior
///
/// Decimal `weight` is serialized as a string to preserve precision.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct VoteWeightResponse {
    pub id: Uuid,
    pub voter_role: String,
    pub weight: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<VoteWeight> for VoteWeightResponse {
    fn from(w: VoteWeight) -> Self {
        Self {
            id: w.id,
            voter_role: w.voter_role,
            weight: w.weight.to_string(),
            description: w.description,
            created_at: w.created_at,
        }
    }
}

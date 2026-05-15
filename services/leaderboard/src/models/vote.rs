use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a vote.
///
/// # Expected Behavior
///
/// Maps directly to the `leaderboard.votes` table. Each row represents
/// a unique (`voter_token_hash`, `project_id`) pair enforced by a UNIQUE
/// constraint. `weight_applied` stores the actual vote weight applied
/// based on the voter's role at the time of casting. `is_valid` tracks
/// moderation status.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: Uuid,
    pub project_id: Uuid,
    pub voter_token_hash: String,
    pub voter_ip: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub is_valid: Option<bool>,
    pub moderated_at: Option<DateTime<Utc>>,
    pub moderated_by: Option<Uuid>,
    pub moderation_reason: Option<String>,
    pub weight_applied: Option<Decimal>,
}

/// Request body for casting a vote.
///
/// # Expected Behavior
///
/// `project_id` and `voter_token_hash` are required. `voter_ip` is optional
/// and used for fraud detection. Duplicate votes for the same
/// (`voter_token_hash`, `project_id`) pair are rejected by the database
/// UNIQUE constraint.
#[derive(Debug, Clone, Deserialize)]
pub struct VoteCreate {
    pub project_id: Uuid,
    pub voter_token_hash: String,
    pub voter_ip: Option<String>,
}

/// Response body for a single vote.
///
/// # Expected Behavior
///
/// Decimal fields are serialized as strings to preserve precision.
/// `is_valid` defaults to `true` if null in the database.
#[derive(Debug, Clone, Serialize)]
pub struct VoteResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub voter_token_hash: String,
    pub voter_ip: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub is_valid: bool,
    pub weight_applied: String,
}

impl From<Vote> for VoteResponse {
    fn from(v: Vote) -> Self {
        Self {
            id: v.id,
            project_id: v.project_id,
            voter_token_hash: v.voter_token_hash,
            voter_ip: v.voter_ip,
            created_at: v.created_at,
            is_valid: v.is_valid.unwrap_or(true),
            weight_applied: v.weight_applied.unwrap_or(Decimal::ONE).to_string(),
        }
    }
}

/// Request body for moderating a vote.
///
/// # Expected Behavior
///
/// `is_valid` is required to set the moderation status. `moderation_reason`
/// is optional and documents why the vote was moderated.
#[derive(Debug, Clone, Deserialize)]
pub struct VoteModerate {
    pub is_valid: bool,
    pub moderation_reason: Option<String>,
}

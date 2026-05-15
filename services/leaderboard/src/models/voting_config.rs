use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of voting configuration.
///
/// # Expected Behavior
///
/// Maps directly to the `leaderboard.voting_config` table. Controls
/// voting windows, rate limits, and default vote weight. Only one config
/// should be `is_active = true` at a time. `phase_id` is NULL for global
/// config. `opens_at` and `closes_at` are NULL for always-open or
/// never-close voting.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VotingConfig {
    pub id: Uuid,
    pub phase_id: Option<Uuid>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub max_votes_per_user: Option<i32>,
    pub max_votes_per_minute: Option<i32>,
    pub rate_limit_window_seconds: Option<i32>,
    pub vote_weight_default: Option<Decimal>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request body for updating voting configuration.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated. If no
/// fields are provided, the existing config is returned unchanged.
#[derive(Debug, Clone, Deserialize)]
pub struct VotingConfigUpdate {
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub max_votes_per_user: Option<i32>,
    pub max_votes_per_minute: Option<i32>,
    pub rate_limit_window_seconds: Option<i32>,
    pub vote_weight_default: Option<Decimal>,
    pub is_active: Option<bool>,
}

/// Response body for voting configuration.
///
/// # Expected Behavior
///
/// Decimal fields are serialized as strings to preserve precision.
/// Integer and boolean defaults are applied for nullable columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingConfigResponse {
    pub id: Uuid,
    pub phase_id: Option<Uuid>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub max_votes_per_user: i32,
    pub max_votes_per_minute: i32,
    pub rate_limit_window_seconds: i32,
    pub vote_weight_default: String,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<VotingConfig> for VotingConfigResponse {
    fn from(c: VotingConfig) -> Self {
        Self {
            id: c.id,
            phase_id: c.phase_id,
            opens_at: c.opens_at,
            closes_at: c.closes_at,
            max_votes_per_user: c.max_votes_per_user.unwrap_or(10),
            max_votes_per_minute: c.max_votes_per_minute.unwrap_or(5),
            rate_limit_window_seconds: c.rate_limit_window_seconds.unwrap_or(60),
            vote_weight_default: c.vote_weight_default.unwrap_or(Decimal::ONE).to_string(),
            is_active: c.is_active.unwrap_or(true),
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a hackathon phase.
///
/// # Expected Behavior
///
/// Maps directly to the `core.hackathon_phases` table. `type` is one of:
/// "registration", "hacking", "submission", "judging", "awards". `is_active`
/// is managed by the scheduler. `config` is JSONB for phase-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HackathonPhase {
    pub id: Uuid,
    pub hackathon_id: Uuid,
    pub name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub config: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request body for creating a new phase.
///
/// # Expected Behavior
///
/// `hackathon_id`, `name`, `type`, `opens_at`, and `closes_at` are required.
/// `type` must be one of: "registration", "hacking", "submission", "judging",
/// "awards".
#[derive(Debug, Clone, Deserialize)]
pub struct PhaseCreate {
    pub hackathon_id: Uuid,
    pub name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub opens_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
    pub config: Option<serde_json::Value>,
}

/// Request body for updating a phase.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated.
#[derive(Debug, Clone, Deserialize)]
pub struct PhaseUpdate {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub description: Option<String>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub config: Option<serde_json::Value>,
}

/// Response body for a single phase.
///
/// # Expected Behavior
///
/// Contains all phase fields with `is_active` defaulting to false.
#[derive(Debug, Clone, Serialize)]
pub struct PhaseResponse {
    pub id: Uuid,
    pub hackathon_id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub phase_type: String,
    pub description: Option<String>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub config: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<HackathonPhase> for PhaseResponse {
    fn from(p: HackathonPhase) -> Self {
        Self {
            id: p.id,
            hackathon_id: p.hackathon_id,
            name: p.name,
            phase_type: p.r#type,
            description: p.description,
            opens_at: p.opens_at,
            closes_at: p.closes_at,
            is_active: p.is_active.unwrap_or(false),
            config: p.config.unwrap_or(serde_json::json!({})),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

/// Response body for the next phase transition.
///
/// # Expected Behavior
///
/// Contains the next phase that will transition and when it will happen.
#[derive(Debug, Clone, Serialize)]
pub struct NextTransitionResponse {
    pub phase: PhaseResponse,
    pub transition_at: DateTime<Utc>,
    pub transition_type: String,
}

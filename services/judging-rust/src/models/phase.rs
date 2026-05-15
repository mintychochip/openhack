use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Phase {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    #[sqlx(rename = "phase_type")]
    pub kind: String,
    pub rubric_id: Option<Uuid>,
    pub opens_at: Option<NaiveDateTime>,
    pub closes_at: Option<NaiveDateTime>,
    pub advancement_count: Option<i32>,
    pub parent_phase_id: Option<Uuid>,
    pub status: String,
    pub scoring_method: String,
    pub normalization_config: Option<serde_json::Value>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhaseCreate {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub phase_type: String,
    pub rubric_id: Option<Uuid>,
    pub opens_at: Option<String>,
    pub closes_at: Option<String>,
    pub advancement_count: Option<i32>,
    pub parent_phase_id: Option<Uuid>,
    #[serde(default = "default_scoring_method")]
    pub scoring_method: String,
    pub normalization_config: Option<serde_json::Value>,
}

fn default_scoring_method() -> String {
    "raw".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhaseUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub scoring_method: Option<String>,
    pub rubric_id: Option<Uuid>,
    pub _opens_at: Option<String>,
    pub _closes_at: Option<String>,
    pub advancement_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub phase_type: String,
    pub rubric_id: Option<Uuid>,
    pub opens_at: Option<NaiveDateTime>,
    pub closes_at: Option<NaiveDateTime>,
    pub advancement_count: Option<i32>,
    pub parent_phase_id: Option<Uuid>,
    pub status: String,
    pub scoring_method: String,
    pub normalization_config: Option<serde_json::Value>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Phase> for PhaseResponse {
    fn from(p: Phase) -> Self {
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            phase_type: p.kind,
            rubric_id: p.rubric_id,
            opens_at: p.opens_at,
            closes_at: p.closes_at,
            advancement_count: p.advancement_count,
            parent_phase_id: p.parent_phase_id,
            status: p.status,
            scoring_method: p.scoring_method,
            normalization_config: p.normalization_config,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseListResponse {
    pub phases: Vec<PhaseResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhaseQuery {
    pub status_filter: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseLeaderboardEntry {
    pub rank: i32,
    pub team_id: Uuid,
    pub project_id: Uuid,
    pub total_score: f64,
    pub score_count: i32,
    pub normalized_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseLeaderboardResponse {
    pub phase_id: Uuid,
    pub phase_name: String,
    pub scoring_method: String,
    pub entries: Vec<PhaseLeaderboardEntry>,
    pub total: i64,
}

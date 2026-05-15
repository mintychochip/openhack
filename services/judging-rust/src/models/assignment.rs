use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub judge_id: Uuid,
    pub project_id: Uuid,
    pub phase_id: Option<Uuid>,
    pub rubric_id: Option<Uuid>,
    pub rubric_version: Option<i32>,
    pub status: String,
    pub priority: Option<i32>,
    pub assigned_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignmentCreate {
    pub judge_id: Uuid,
    pub project_id: Uuid,
    pub rubric_id: Option<Uuid>,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignmentUpdate {
    pub status: Option<String>,
    pub rubric_id: Option<Uuid>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentResponse {
    pub id: Uuid,
    pub judge_id: Uuid,
    pub project_id: Uuid,
    pub rubric_id: Option<Uuid>,
    pub status: String,
    pub priority: i32,
    pub assigned_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

impl From<Assignment> for AssignmentResponse {
    fn from(a: Assignment) -> Self {
        Self {
            id: a.id,
            judge_id: a.judge_id,
            project_id: a.project_id,
            rubric_id: a.rubric_id,
            status: a.status,
            priority: a.priority.unwrap_or(0),
            assigned_at: a.assigned_at,
            completed_at: a.completed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentListResponse {
    pub assignments: Vec<AssignmentResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManualAssignment {
    pub judge_id: Uuid,
    pub project_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignmentBulkRequest {
    pub distribution: String,
    #[serde(default)]
    pub judge_ids: Vec<Uuid>,
    #[serde(default)]
    pub project_ids: Vec<Uuid>,
    #[serde(default)]
    pub assignments: Vec<ManualAssignment>,
    pub rubric_id: Option<Uuid>,
    #[serde(default = "default_assignments_per_project")]
    pub assignments_per_project: i32,
}

fn default_assignments_per_project() -> i32 {
    3
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignmentQuery {
    pub judge_id: Option<Uuid>,
    pub status_filter: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

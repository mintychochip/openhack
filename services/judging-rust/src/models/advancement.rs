use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Advancement {
    pub id: Uuid,
    pub from_phase_id: Option<Uuid>,
    pub to_phase_id: Option<Uuid>,
    pub team_id: Uuid,
    pub project_id: Uuid,
    pub advanced_at: Option<NaiveDateTime>,
    pub rank_in_phase: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvancementListResponse {
    pub advancements: Vec<Advancement>,
    pub total: i64,
}

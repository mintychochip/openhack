use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Score {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub scores: serde_json::Value,
    #[sqlx(rename = "total_score")]
    pub total: Option<rust_decimal::Decimal>,
    pub comment: Option<String>,
    pub feedback: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoreCreate {
    pub assignment_id: Uuid,
    pub scores: std::collections::HashMap<String, f64>,
    pub comment: Option<String>,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoreUpdate {
    pub scores: Option<std::collections::HashMap<String, f64>>,
    pub comment: Option<String>,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreResponse {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub scores: serde_json::Value,
    pub total_score: Option<String>,
    pub comment: Option<String>,
    pub feedback: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    #[serde(default)]
    pub is_normalized: bool,
    pub judge_stats: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreListResponse {
    pub scores: Vec<ScoreResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoreQuery {
    pub judge_id: Option<Uuid>,
}

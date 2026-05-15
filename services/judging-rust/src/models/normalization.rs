use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JudgeStats {
    pub id: Uuid,
    pub judge_id: Uuid,
    pub phase_id: Option<Uuid>,
    pub mean_score: rust_decimal::Decimal,
    pub std_dev: rust_decimal::Decimal,
    pub min_score: rust_decimal::Decimal,
    pub max_score: rust_decimal::Decimal,
    pub total_scores_count: i32,
    pub computed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizationResult {
    pub phase_id: Uuid,
    pub method: String,
    pub judge_count: i32,
    pub scores_normalized: i32,
    pub mean_of_means: f64,
    pub avg_std_dev: f64,
    pub computed_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NormalizationRequest {
    pub phase_id: Uuid,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub _dry_run: bool,
}

fn default_method() -> String {
    "zscore".into()
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizationPreviewEntry {
    pub score_id: Uuid,
    pub judge_id: Uuid,
    pub raw_score: f64,
    pub normalized_score: f64,
    pub method: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizationResponse {
    pub success: bool,
    pub result: Option<NormalizationResult>,
    pub preview: Option<Vec<NormalizationPreviewEntry>>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NormalizationStatusQuery {
    pub method: Option<String>,
}

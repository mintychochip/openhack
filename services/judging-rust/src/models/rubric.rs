use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Rubric {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub criteria: serde_json::Value,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RubricVersion {
    pub id: Uuid,
    pub rubric_id: Uuid,
    pub version: i32,
    pub criteria: serde_json::Value,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionInput {
    pub name: String,
    pub description: Option<String>,
    pub max_score: f64,
    #[serde(default = "default_weight")]
    pub weight: f64,
    pub order: Option<i32>,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_weight() -> f64 {
    1.0
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct RubricCreate {
    pub name: String,
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_criteria")]
    pub criteria: Vec<CriterionInput>,
}

fn deserialize_criteria<'de, D>(deserializer: D) -> Result<Vec<CriterionInput>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Vec<CriterionInput> = Vec::deserialize(deserializer)?;
    if v.is_empty() {
        return Err(serde::de::Error::custom(
            "criteria must have at least one entry",
        ));
    }
    Ok(v)
}

#[derive(Debug, Clone, Deserialize)]
pub struct RubricUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub criteria: Option<Vec<CriterionInput>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RubricResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub criteria: serde_json::Value,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Rubric> for RubricResponse {
    fn from(r: Rubric) -> Self {
        Self {
            id: r.id,
            name: r.name,
            description: r.description,
            criteria: r.criteria,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RubricVersionResponse {
    pub id: Uuid,
    pub rubric_id: Uuid,
    pub version: i32,
    pub criteria: serde_json::Value,
    pub created_at: Option<NaiveDateTime>,
}

impl From<RubricVersion> for RubricVersionResponse {
    fn from(r: RubricVersion) -> Self {
        Self {
            id: r.id,
            rubric_id: r.rubric_id,
            version: r.version,
            criteria: r.criteria,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RubricListResponse {
    pub rubrics: Vec<RubricResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RubricVersionListResponse {
    pub versions: Vec<RubricVersionResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoreValidationRequest {
    pub assignment_id: Uuid,
    pub scores: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub normalized_total: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

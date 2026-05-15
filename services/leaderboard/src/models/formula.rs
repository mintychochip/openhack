use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a ranking formula.
///
/// # Expected Behavior
///
/// Maps directly to the `leaderboard.ranking_formulas` table. The `formula`
/// field contains a mathematical expression using variable names like
/// `judge_score` and `public_votes`. `variables` lists the variable names
/// used in the formula. Only one formula can be `is_active = true` at a
/// time. `is_preset` marks system-provided formulas that cannot be deleted.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RankingFormula {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub formula: String,
    pub variables: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub is_preset: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request body for creating a new ranking formula.
///
/// # Expected Behavior
///
/// `name` and `formula` are required. `description` and `variables` are
/// optional. `is_active` and `is_preset` default to `false` on the server
/// side regardless of what is sent by the client.
#[derive(Debug, Clone, Deserialize)]
pub struct FormulaCreate {
    pub name: String,
    pub description: Option<String>,
    pub formula: String,
    pub variables: Option<Vec<String>>,
}

/// Response body for a single ranking formula.
///
/// # Expected Behavior
///
/// Boolean defaults are applied for nullable columns. All fields from the
/// database row are included.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub formula: String,
    pub variables: Option<Vec<String>>,
    pub is_active: bool,
    pub is_preset: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<RankingFormula> for FormulaResponse {
    fn from(f: RankingFormula) -> Self {
        Self {
            id: f.id,
            name: f.name,
            description: f.description,
            formula: f.formula,
            variables: f.variables,
            is_active: f.is_active.unwrap_or(false),
            is_preset: f.is_preset.unwrap_or(false),
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

/// Response body for a list of ranking formulas.
///
/// # Expected Behavior
///
/// Contains the list of formula responses and a total count for pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaListResponse {
    pub formulas: Vec<FormulaResponse>,
    pub total: i64,
}

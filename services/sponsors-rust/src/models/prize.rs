use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a sponsor prize.
///
/// # Expected Behavior
///
/// Maps directly to the `sponsor.prizes` table. `value_usd` is a
/// `DECIMAL(10,2)` mapped to `rust_decimal::Decimal`. `winner_project_id`
/// is nullable and only set when a winner is announced.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prize {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub value_usd: Option<Decimal>,
    pub criteria: Option<String>,
    pub winner_project_id: Option<Uuid>,
    pub announced: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
}

/// Request body for creating a new prize.
///
/// # Expected Behavior
///
/// `booth_id` and `title` are required. All other fields are optional.
/// The `announced` field defaults to `false` on the server side.
#[derive(Debug, Clone, Deserialize)]
pub struct PrizeCreate {
    pub title: String,
    pub description: Option<String>,
    pub value_usd: Option<Decimal>,
    pub criteria: Option<String>,
}

/// Request body for updating an existing prize.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated. If no
/// fields are provided, no database update occurs (the existing row is
/// returned unchanged).
#[derive(Debug, Clone, Deserialize)]
pub struct PrizeUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub value_usd: Option<Decimal>,
    pub criteria: Option<String>,
}

/// Request body for announcing a prize winner.
///
/// # Expected Behavior
///
/// `winner_project_id` is required and identifies the winning project.
/// Sets `announced` to `true` and stores the winner reference.
#[derive(Debug, Clone, Deserialize)]
pub struct PrizeAnnounce {
    pub winner_project_id: Uuid,
}

/// Response body for a single prize.
///
/// # Expected Behavior
///
/// Contains all prize fields. `announced` defaults to `false` if null
/// in the database. `value_usd` is serialized as a string to preserve
/// decimal precision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrizeResponse {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub value_usd: Option<String>,
    pub criteria: Option<String>,
    pub winner_project_id: Option<Uuid>,
    pub announced: bool,
    pub created_at: Option<NaiveDateTime>,
}

impl From<Prize> for PrizeResponse {
    fn from(p: Prize) -> Self {
        Self {
            id: p.id,
            booth_id: p.booth_id,
            title: p.title,
            description: p.description,
            value_usd: p.value_usd.map(|d| d.to_string()),
            criteria: p.criteria,
            winner_project_id: p.winner_project_id,
            announced: p.announced.unwrap_or(false),
            created_at: p.created_at,
        }
    }
}

/// Response body for a list of prizes.
///
/// # Expected Behavior
///
/// Contains the list of prize responses and a total count for pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrizeListResponse {
    pub prizes: Vec<PrizeResponse>,
    pub total: i64,
}

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a sponsor booth.
///
/// # Expected Behavior
///
/// Maps directly to the `sponsor.booths` table. All fields correspond to
/// column values. `theme_colors` is stored as JSONB and represented as
/// `serde_json::Value`. `technologies` is a `PostgreSQL` text[] array
/// represented as `Vec<String>`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Booth {
    pub id: Uuid,
    pub sponsor_id: Option<String>,
    pub sponsor_name: String,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub banner_url: Option<String>,
    pub website_url: Option<String>,
    pub careers_url: Option<String>,
    pub api_docs_url: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub contact_email: Option<String>,
    pub discord_channel: Option<String>,
    pub theme_colors: Option<serde_json::Value>,
    pub published: Option<bool>,
    pub view_count: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// Request body for creating a new booth.
///
/// # Expected Behavior
///
/// Only `sponsor_name` is required. All other fields are optional. The
/// `published` field defaults to `false` on the server side regardless
/// of what is sent by the client.
#[derive(Debug, Clone, Deserialize)]
pub struct BoothCreate {
    pub sponsor_name: String,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub banner_url: Option<String>,
    pub website_url: Option<String>,
    pub careers_url: Option<String>,
    pub api_docs_url: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub contact_email: Option<String>,
    pub discord_channel: Option<String>,
    pub theme_colors: Option<serde_json::Value>,
}

/// Request body for updating an existing booth.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated. If no
/// fields are provided, only `updated_at` is refreshed.
#[derive(Debug, Clone, Deserialize)]
pub struct BoothUpdate {
    pub sponsor_name: Option<String>,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub banner_url: Option<String>,
    pub website_url: Option<String>,
    pub careers_url: Option<String>,
    pub api_docs_url: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub contact_email: Option<String>,
    pub discord_channel: Option<String>,
    pub theme_colors: Option<serde_json::Value>,
}

/// Request body for toggling booth published state.
///
/// # Expected Behavior
///
/// If `published` is provided, sets the booth's published flag to that
/// value. If not provided, toggles the current value.
#[derive(Debug, Clone, Deserialize)]
pub struct BoothPublish {
    pub published: Option<bool>,
}

/// Response body for a single booth.
///
/// # Expected Behavior
///
/// Contains all booth fields. `published`, `view_count`, `created_at`,
/// and `updated_at` are always populated from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoothResponse {
    pub id: Uuid,
    pub sponsor_id: Option<String>,
    pub sponsor_name: String,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub banner_url: Option<String>,
    pub website_url: Option<String>,
    pub careers_url: Option<String>,
    pub api_docs_url: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub contact_email: Option<String>,
    pub discord_channel: Option<String>,
    pub theme_colors: Option<serde_json::Value>,
    pub published: bool,
    pub view_count: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Booth> for BoothResponse {
    fn from(b: Booth) -> Self {
        Self {
            id: b.id,
            sponsor_id: b.sponsor_id,
            sponsor_name: b.sponsor_name,
            tagline: b.tagline,
            description: b.description,
            logo_url: b.logo_url,
            banner_url: b.banner_url,
            website_url: b.website_url,
            careers_url: b.careers_url,
            api_docs_url: b.api_docs_url,
            technologies: b.technologies,
            contact_email: b.contact_email,
            discord_channel: b.discord_channel,
            theme_colors: b.theme_colors,
            published: b.published.unwrap_or(false),
            view_count: b.view_count.unwrap_or(0),
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}

/// Response body for a list of booths.
///
/// # Expected Behavior
///
/// Contains the list of booth responses and a total count for pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoothListResponse {
    pub booths: Vec<BoothResponse>,
    pub total: i64,
}

/// Response body for booth analytics.
///
/// # Expected Behavior
///
/// Contains the booth ID, view count, prize count, and submission count.
/// All counts are non-negative integers.
#[derive(Debug, Clone, Serialize)]
pub struct BoothAnalyticsResponse {
    pub booth_id: Uuid,
    pub view_count: i32,
    pub prize_count: i64,
    pub submission_count: i64,
}

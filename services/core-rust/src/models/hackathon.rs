use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of hackathon configuration.
///
/// # Expected Behavior
///
/// Maps directly to the `core.hackathon_config` table. This is a singleton
/// table (one row). All fields correspond to column values. `theme_colors`
/// and `social_links` are JSONB columns represented as `serde_json::Value`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HackathonConfig {
    pub id: Uuid,
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub logo_url: Option<String>,
    pub theme_colors: Option<serde_json::Value>,
    pub social_links: Option<serde_json::Value>,
    pub registration_open: Option<bool>,
    pub custom_css: Option<String>,
    pub daisyui_theme_preset: Option<String>,
    pub daisyui_custom_theme: Option<serde_json::Value>,
    pub font_config: Option<serde_json::Value>,
}

/// Request body for updating hackathon configuration.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated.
#[derive(Debug, Clone, Deserialize)]
pub struct HackathonConfigUpdate {
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub logo_url: Option<String>,
    pub theme_colors: Option<serde_json::Value>,
    pub social_links: Option<serde_json::Value>,
    pub registration_open: Option<bool>,
    pub custom_css: Option<String>,
    pub daisyui_theme_preset: Option<String>,
    pub daisyui_custom_theme: Option<serde_json::Value>,
    pub font_config: Option<serde_json::Value>,
}

/// Response body for hackathon configuration.
///
/// # Expected Behavior
///
/// Contains all hackathon config fields with sensible defaults for
/// optional columns.
#[derive(Debug, Clone, Serialize)]
pub struct HackathonConfigResponse {
    pub id: Uuid,
    pub name: String,
    pub tagline: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub timezone: String,
    pub logo_url: Option<String>,
    pub theme_colors: serde_json::Value,
    pub social_links: serde_json::Value,
    pub registration_open: bool,
    pub custom_css: String,
    pub daisyui_theme_preset: String,
    pub daisyui_custom_theme: serde_json::Value,
    pub font_config: serde_json::Value,
}

impl From<HackathonConfig> for HackathonConfigResponse {
    fn from(c: HackathonConfig) -> Self {
        Self {
            id: c.id,
            name: c.name.unwrap_or_default(),
            tagline: c.tagline.unwrap_or_default(),
            start_time: c.start_time,
            end_time: c.end_time,
            timezone: c.timezone.unwrap_or_else(|| "UTC".to_string()),
            logo_url: c.logo_url,
            theme_colors: c.theme_colors.unwrap_or(serde_json::json!({})),
            social_links: c.social_links.unwrap_or(serde_json::json!({})),
            registration_open: c.registration_open.unwrap_or(true),
            custom_css: c.custom_css.unwrap_or_default(),
            daisyui_theme_preset: c.daisyui_theme_preset.unwrap_or_else(|| "light".to_string()),
            daisyui_custom_theme: c.daisyui_custom_theme.unwrap_or(serde_json::json!({})),
            font_config: c.font_config.unwrap_or(serde_json::json!({})),
        }
    }
}

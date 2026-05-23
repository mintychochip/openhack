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
    pub ai_provider: Option<String>,
    pub ai_enabled: Option<bool>,
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub openai_model: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub anthropic_model: Option<String>,
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
    pub ai_provider: Option<String>,
    pub ai_enabled: Option<bool>,
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub openai_model: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub anthropic_model: Option<String>,
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
    pub ai_provider: String,
    pub ai_enabled: bool,
    pub openai_api_key: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub anthropic_api_key: String,
    pub anthropic_model: String,
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
            ai_provider: c.ai_provider.unwrap_or_else(|| "openai".to_string()),
            ai_enabled: c.ai_enabled.unwrap_or(false),
            openai_api_key: mask_api_key(&c.openai_api_key),
            openai_base_url: c.openai_base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            openai_model: c.openai_model.unwrap_or_else(|| "gpt-4-turbo".to_string()),
            anthropic_api_key: mask_api_key(&c.anthropic_api_key),
            anthropic_model: c.anthropic_model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
        }
    }
}

/// Mask an API key for safe display in API responses.
///
/// # Expected Behavior
///
/// Shows only the last 4 characters prefixed with `***`. Returns an empty
/// string if the input is None or shorter than 5 characters.
fn mask_api_key(key: &Option<String>) -> String {
    match key {
        Some(k) if k.len() > 4 => format!("***{}", &k[k.len()-4..]),
        Some(_) => "***".to_string(),
        None => String::new(),
    }
}

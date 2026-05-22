use sqlx::PgPool;

use crate::errors::CoreError;
use crate::models::hackathon::{HackathonConfig, HackathonConfigResponse, HackathonConfigUpdate};

pub struct HackathonService;

impl HackathonService {
    /// Get the hackathon configuration (singleton).
    ///
    /// # Expected Behavior
    ///
    /// Fetches the first row from `core.hackathon_config`. Returns
    /// `CoreError::NotFound` if no configuration exists.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no hackathon config row exists.
    /// Returns `CoreError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.hackathon_config` (database read).
    pub async fn get_config(pool: &PgPool) -> Result<HackathonConfigResponse, CoreError> {
        let row = sqlx::query_as::<_, HackathonConfig>(
            "SELECT id, name, tagline, start_time, end_time, timezone, logo_url, theme_colors, social_links, registration_open, custom_css, daisyui_theme_preset, daisyui_custom_theme, font_config FROM core.hackathon_config LIMIT 1",
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("HackathonConfig".into(), "singleton".into()))?;

        Ok(row.into())
    }

    /// Update the hackathon configuration (singleton).
    ///
    /// # Expected Behavior
    ///
    /// Updates the first row in `core.hackathon_config` with the provided
    /// fields. Only non-None fields are updated. Returns the updated config.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no hackathon config row exists.
    /// Returns `CoreError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates row in `core.hackathon_config` (database write).
    /// - Logs at INFO on success.
    pub async fn update_config(
        pool: &PgPool,
        data: &HackathonConfigUpdate,
    ) -> Result<HackathonConfigResponse, CoreError> {
        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.name.is_some() {
            updates.push(format!("name = ${param_idx}"));
            param_idx += 1;
        }
        if data.tagline.is_some() {
            updates.push(format!("tagline = ${param_idx}"));
            param_idx += 1;
        }
        if data.start_time.is_some() {
            updates.push(format!("start_time = ${param_idx}"));
            param_idx += 1;
        }
        if data.end_time.is_some() {
            updates.push(format!("end_time = ${param_idx}"));
            param_idx += 1;
        }
        if data.timezone.is_some() {
            updates.push(format!("timezone = ${param_idx}"));
            param_idx += 1;
        }
        if data.logo_url.is_some() {
            updates.push(format!("logo_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.theme_colors.is_some() {
            updates.push(format!("theme_colors = ${param_idx}"));
            param_idx += 1;
        }
        if data.social_links.is_some() {
            updates.push(format!("social_links = ${param_idx}"));
            param_idx += 1;
        }
        if data.registration_open.is_some() {
            updates.push(format!("registration_open = ${param_idx}"));
            param_idx += 1;
        }
        if data.custom_css.is_some() {
            updates.push(format!("custom_css = ${param_idx}"));
            param_idx += 1;
        }
        if data.daisyui_theme_preset.is_some() {
            updates.push(format!("daisyui_theme_preset = ${param_idx}"));
            param_idx += 1;
        }
        if data.daisyui_custom_theme.is_some() {
            updates.push(format!("daisyui_custom_theme = ${param_idx}"));
            param_idx += 1;
        }
        if data.font_config.is_some() {
            updates.push(format!("font_config = ${param_idx}"));
        }

        if updates.is_empty() {
            return Self::get_config(pool).await;
        }

        let sql = format!(
            "UPDATE core.hackathon_config SET {} WHERE id = (SELECT id FROM core.hackathon_config LIMIT 1) RETURNING id, name, tagline, start_time, end_time, timezone, logo_url, theme_colors, social_links, registration_open, custom_css, daisyui_theme_preset, daisyui_custom_theme, font_config",
            updates.join(", ")
        );

        let mut query = sqlx::query_as::<_, HackathonConfig>(&sql);

        if let Some(ref v) = data.name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.tagline {
            query = query.bind(v);
        }
        if let Some(ref v) = data.start_time {
            query = query.bind(v);
        }
        if let Some(ref v) = data.end_time {
            query = query.bind(v);
        }
        if let Some(ref v) = data.timezone {
            query = query.bind(v);
        }
        if let Some(ref v) = data.logo_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.theme_colors {
            query = query.bind(v);
        }
        if let Some(ref v) = data.social_links {
            query = query.bind(v);
        }
        if let Some(ref v) = data.registration_open {
            query = query.bind(v);
        }
        if let Some(ref v) = data.custom_css {
            query = query.bind(v);
        }
        if let Some(ref v) = data.daisyui_theme_preset {
            query = query.bind(v);
        }
        if let Some(ref v) = data.daisyui_custom_theme {
            query = query.bind(v);
        }
        if let Some(ref v) = data.font_config {
            query = query.bind(v);
        }

        let row = query
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| CoreError::NotFound("HackathonConfig".into(), "singleton".into()))?;

        log::info!("Updated hackathon config");
        Ok(row.into())
    }
}

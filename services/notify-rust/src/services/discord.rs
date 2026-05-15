use sqlx::PgPool;
use std::time::Duration;

use crate::errors::NotifyError;

/// Send a message to a Discord channel via webhook URL.
///
/// # Expected Behavior
///
/// Posts a JSON payload to the Discord webhook URL. The payload includes
/// `content`, `username` (defaults to "`OpenHack` Bot"), and optional `embeds`.
/// The webhook URL is taken from the `discord_webhook_url` parameter or
/// from the `url` field in the request body. Uses a 15-second timeout.
/// Returns the Discord response body on success.
///
/// # Errors
///
/// Returns `NotifyError::DiscordError` if the HTTP request fails or
/// Discord returns a non-2xx status code.
///
/// # Side Effects
///
/// - Makes an HTTP POST request to the Discord webhook URL (network I/O).
/// - Logs at INFO on success, ERROR on failure.
pub async fn send_discord_webhook(
    _pool: &PgPool,
    discord_webhook_url: &str,
    content: &str,
    username: Option<&str>,
    embeds: Option<&serde_json::Value>,
) -> Result<serde_json::Value, NotifyError> {
    let url = if discord_webhook_url.is_empty() {
        return Err(NotifyError::DiscordError(
            "DISCORD_WEBHOOK_URL not configured".into(),
        ));
    } else {
        discord_webhook_url
    };

    let mut body = serde_json::json!({
        "content": content,
        "username": username.unwrap_or("OpenHack Bot"),
    });

    if let Some(e) = embeds {
        body["embeds"] = e.clone();
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| NotifyError::DiscordError(format!("Request failed: {e}")))?;

    let status = response.status();
    let resp_body = response.text().await.unwrap_or_default();

    if status.is_success() {
        log::info!("Discord webhook sent successfully");
        Ok(serde_json::json!({
            "status": "sent",
            "response": resp_body
        }))
    } else {
        log::error!("Discord webhook failed with status {status}: {resp_body}");
        Err(NotifyError::DiscordError(format!(
            "Discord returned {status}: {resp_body}"
        )))
    }
}

/// Assign a role to a Discord user via the Discord API.
///
/// # Expected Behavior
///
/// Uses the Discord Bot API to assign a role to a user in a guild.
/// Makes a PUT request to `/guilds/{guild_id}/members/{user_id}/roles/{role_id}`
/// with the bot token in the Authorization header. Returns success on 204 No Content.
///
/// # Errors
///
/// Returns `NotifyError::DiscordError` if the bot token or guild ID is not
/// configured, or if the API request fails with a non-2xx status.
///
/// # Side Effects
///
/// - Makes an HTTP PUT request to the Discord API (network I/O).
/// - Assigns a role to a Discord user (external state change).
/// - Logs at INFO on success, ERROR on failure.
pub async fn assign_discord_role(
    _pool: &PgPool,
    bot_token: &str,
    guild_id: &str,
    user_id: &str,
    role_id: &str,
) -> Result<serde_json::Value, NotifyError> {
    if bot_token.is_empty() {
        return Err(NotifyError::DiscordError(
            "DISCORD_BOT_TOKEN not configured".into(),
        ));
    }
    if guild_id.is_empty() {
        return Err(NotifyError::DiscordError(
            "DISCORD_GUILD_ID not configured".into(),
        ));
    }

    let url =
        format!("https://discord.com/api/v10/guilds/{guild_id}/members/{user_id}/roles/{role_id}");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let response = client
        .put(&url)
        .header("Authorization", format!("Bot {bot_token}"))
        .send()
        .await
        .map_err(|e| NotifyError::DiscordError(format!("Request failed: {e}")))?;

    let status = response.status();

    if status.is_success() {
        log::info!("Discord role {role_id} assigned to user {user_id}");
        Ok(serde_json::json!({
            "status": "assigned",
            "user_id": user_id,
            "role_id": role_id
        }))
    } else {
        let body = response.text().await.unwrap_or_default();
        log::error!("Discord role assignment failed: {status} - {body}");
        Err(NotifyError::DiscordError(format!(
            "Discord API returned {status}: {body}"
        )))
    }
}

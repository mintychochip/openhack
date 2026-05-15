use crate::config::Config;
use crate::errors::BotError;

/// Register slash commands with Discord via the REST API.
///
/// # Expected Behavior
///
/// Sends a PUT request to
/// `https://discord.com/api/v10/applications/{app_id}/guilds/{guild_id}/commands`
/// with the full list of command definitions. This bulk-overwrites all guild
/// commands. On success, logs the number of registered commands. On failure,
/// returns `BotError::DiscordError`.
///
/// # Errors
///
/// Returns `BotError::DiscordError` if the HTTP request fails or Discord
/// returns a non-success status code.
///
/// # Side Effects
///
/// - Makes an HTTP PUT request to the Discord API.
/// - Replaces all existing guild slash commands with the new set.
/// - Logs at INFO on success, or includes status in the error on failure.
pub async fn register_commands(config: &Config) -> Result<(), BotError> {
    let commands = crate::commands::get_command_definitions();
    let url = format!(
        "https://discord.com/api/v10/applications/{}/guilds/{}/commands",
        config.discord_application_id, config.discord_guild_id
    );

    let http = reqwest::Client::new();
    let resp = http
        .put(&url)
        .header("Authorization", format!("Bot {}", config.discord_bot_token))
        .json(&commands)
        .send()
        .await
        .map_err(|e| BotError::DiscordError(format!("Command registration request failed: {e}")))?;

    if resp.status().is_success() {
        log::info!(
            "Registered {} slash commands on guild {}",
            commands.len(),
            config.discord_guild_id
        );
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(BotError::DiscordError(format!(
            "Command registration returned {status}: {body}"
        )))
    }
}

mod help;
pub mod leaderboard;
mod register;
mod schedule;
mod status;
pub mod submit;
mod team;

use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Get all slash command definitions for registration with Discord.
///
/// # Expected Behavior
///
/// Returns a vector of JSON command definition objects matching the Discord
/// Application Command structure. Each definition includes the command name,
/// description, and options (subcommands and their parameters). These are
/// sent to Discord via `PUT /applications/{app_id}/guilds/{guild_id}/commands`
/// on startup.
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None. Pure function.
pub fn get_command_definitions() -> Vec<serde_json::Value> {
    vec![serde_json::json!({
        "name": "openhack",
        "description": "OpenHack hackathon management commands",
        "options": [
            {
                "name": "register",
                "description": "Link your Discord account to OpenHack",
                "type": 1
            },
            {
                "name": "status",
                "description": "Show current hackathon status",
                "type": 1
            },
            {
                "name": "team-info",
                "description": "Show your team details",
                "type": 1
            },
            {
                "name": "team-create",
                "description": "Create a new team",
                "type": 1,
                "options": [
                    {
                        "name": "name",
                        "description": "Team name",
                        "type": 3,
                        "required": true
                    }
                ]
            },
            {
                "name": "team-join",
                "description": "Join a team by invite code",
                "type": 1,
                "options": [
                    {
                        "name": "code",
                        "description": "Team invite code",
                        "type": 3,
                        "required": true
                    }
                ]
            },
            {
                "name": "submit",
                "description": "Submit a project (opens a form)",
                "type": 1
            },
            {
                "name": "leaderboard",
                "description": "Show the leaderboard",
                "type": 1,
                "options": [
                    {
                        "name": "top",
                        "description": "Number of entries to show (default 10)",
                        "type": 4,
                        "required": false,
                        "min_value": 1,
                        "max_value": 25
                    }
                ]
            },
            {
                "name": "schedule",
                "description": "Show upcoming events and phases",
                "type": 1
            },
            {
                "name": "help",
                "description": "List all available commands",
                "type": 1
            }
        ]
    })]
}

/// Build a Discord embed JSON object with title, description, and color.
///
/// # Expected Behavior
///
/// Creates a JSON object with `title`, `description`, `color`, and
/// `timestamp` fields. The timestamp is set to the current UTC time in
/// ISO 8601 format.
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None.
pub fn make_embed(title: &str, description: &str, color: u32) -> serde_json::Value {
    serde_json::json!({
        "title": title,
        "description": description,
        "color": color,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })
}

/// Build an interaction response that sends an ephemeral (visible only to the
/// invoking user) message.
///
/// # Expected Behavior
///
/// Returns a type-4 (CHANNEL_MESSAGE_WITH_SOURCE) interaction response with
/// `flags: 64` (EPHEMERAL) and the given content.
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None.
pub fn ephemeral_response(content: &str) -> serde_json::Value {
    serde_json::json!({
        "type": 4,
        "data": {
            "content": content,
            "flags": 64
        }
    })
}

/// Build an interaction response that sends a public embed message.
///
/// # Expected Behavior
///
/// Returns a type-4 (CHANNEL_MESSAGE_WITH_SOURCE) interaction response with
/// the given embed in the `embeds` array.
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None.
pub fn embed_response(embed: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "type": 4,
        "data": {
            "embeds": [embed]
        }
    })
}

/// Require a linked OpenHack account for a command.
///
/// # Expected Behavior
///
/// Looks up the Discord user ID in the `user_links` table. If a link
/// exists, returns the OpenHack user ID as `Ok(String)`. If no link exists,
/// returns `Err` containing an ephemeral interaction response telling the
/// user to register. On database error, returns an ephemeral error message.
///
/// # Errors
///
/// Returns `Err(serde_json::Value)` if the user is not linked or on
/// database error. The `Value` is a ready-to-send ephemeral interaction
/// response.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn require_linked_user(
    pool: &PgPool,
    ctx: &InteractionContext,
) -> Result<String, serde_json::Value> {
    let discord_id = &ctx.discord_user_id;
    match crate::models::user_link::find_by_discord_id(pool, discord_id).await {
        Ok(Some(link)) => Ok(link.openhack_user_id.to_string()),
        Ok(None) => Err(ephemeral_response(
            "You need to link your OpenHack account first! Use `/openhack register`.",
        )),
        Err(e) => {
            log::error!("Failed to look up user link: {e}");
            Err(ephemeral_response(
                "An error occurred while checking your account link.",
            ))
        }
    }
}

/// Extract a string option value from the subcommand's options by name.
///
/// # Expected Behavior
///
/// Navigates the interaction data structure: `data.options[0].options`,
/// searches for an option with the matching `name`, and returns its
/// `value` as a string. Returns `None` if the option is not found or
/// the data structure doesn't match.
///
/// # Errors
///
/// None. Returns `None` on failure.
///
/// # Side Effects
///
/// None.
pub fn get_string_option(ctx: &InteractionContext, name: &str) -> Option<String> {
    let data = ctx.data.as_ref()?;
    let subcmd = data.get("options")?.as_array()?.first()?;
    let sub_options = subcmd.get("options")?.as_array()?;
    for opt in sub_options {
        if opt.get("name").and_then(|n| n.as_str()) == Some(name) {
            return opt.get("value").and_then(|v| v.as_str()).map(String::from);
        }
    }
    None
}

/// Extract an integer option value from the subcommand's options by name.
///
/// # Expected Behavior
///
/// Navigates the interaction data structure: `data.options[0].options`,
/// searches for an option with the matching `name`, and returns its
/// `value` as an i64. Returns `None` if the option is not found or
/// the value is not an integer.
///
/// # Errors
///
/// None. Returns `None` on failure.
///
/// # Side Effects
///
/// None.
pub fn get_int_option(ctx: &InteractionContext, name: &str) -> Option<i64> {
    let data = ctx.data.as_ref()?;
    let subcmd = data.get("options")?.as_array()?.first()?;
    let sub_options = subcmd.get("options")?.as_array()?;
    for opt in sub_options {
        if opt.get("name").and_then(|n| n.as_str()) == Some(name) {
            return opt.get("value").and_then(|v| v.as_i64());
        }
    }
    None
}

/// Extract the `custom_id` from a component or modal interaction's data.
///
/// # Expected Behavior
///
/// Returns `data.custom_id` as a String, or empty string if not present.
///
/// # Errors
///
/// None. Returns empty string on failure.
///
/// # Side Effects
///
/// None.
pub fn get_custom_id(ctx: &InteractionContext) -> String {
    ctx.data
        .as_ref()
        .and_then(|d| d.get("custom_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Extract a text input value from a modal submission by `custom_id`.
///
/// # Expected Behavior
///
/// Navigates the modal submission data: `data.components[*].components[*]`,
/// searching for a component with the matching `custom_id`, and returns
/// its `value` as a String. Returns `None` if not found.
///
/// # Errors
///
/// None. Returns `None` on failure.
///
/// # Side Effects
///
/// None.
pub fn get_modal_value(ctx: &InteractionContext, custom_id: &str) -> Option<String> {
    let data = ctx.data.as_ref()?;
    let rows = data.get("components")?.as_array()?;
    for row in rows {
        let components = row.get("components")?.as_array()?;
        for component in components {
            if component.get("custom_id").and_then(|v| v.as_str()) == Some(custom_id) {
                return component
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(String::from);
            }
        }
    }
    None
}

/// Extract the selected values from a select menu component interaction.
///
/// # Expected Behavior
///
/// Returns `data.values` as a `Vec<String>`, or an empty vec if not present.
///
/// # Errors
///
/// None. Returns empty vec on failure.
///
/// # Side Effects
///
/// None.
pub fn get_selected_values(ctx: &InteractionContext) -> Vec<String> {
    ctx.data
        .as_ref()
        .and_then(|d| d.get("values"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract the subcommand name from the interaction data.
///
/// # Expected Behavior
///
/// Navigates `data.options[0].name` to get the subcommand name. Returns
/// an empty string if the structure doesn't match.
///
/// # Errors
///
/// None. Returns empty string on failure.
///
/// # Side Effects
///
/// None.
fn extract_subcommand(data: &serde_json::Value) -> String {
    data.get("options")
        .and_then(|o| o.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string()
}

/// Top-level dispatcher for incoming Discord interaction events.
///
/// # Expected Behavior
///
/// Dispatches the interaction to the appropriate handler based on the
/// subcommand name. For the "openhack" command, extracts the subcommand
/// and its options, then calls the corresponding handler. Logs
/// interactions if `DISCORD_BOT_LOG_INTERACTIONS` is true. Unrecognized
/// subcommands receive an ephemeral "Unknown command" response.
///
/// Each handler returns a `serde_json::Value` representing the interaction
/// response that is returned directly as the HTTP response body to Discord.
///
/// # Errors
///
/// Errors are logged but not propagated. Individual handler failures
/// result in an ephemeral error message to the user.
///
/// # Side Effects
///
/// - May read/write to the database (interaction logs, user links).
/// - May make HTTP calls to OpenHack services.
/// - The returned response is sent to Discord as the interaction response.
pub async fn handle_interaction(
    pool: &PgPool,
    config: &Config,
    client: &OpenHackClient,
    ctx: &InteractionContext,
) -> serde_json::Value {
    let Some(data) = &ctx.data else {
        return ephemeral_response("Missing interaction data.");
    };

    let subcommand = extract_subcommand(data);

    let result = match subcommand.as_str() {
        "register" => register::handle(ctx, pool, config).await,
        "status" => status::handle(ctx, pool, config, client).await,
        "team-info" => team::handle_info(ctx, pool, config, client).await,
        "team-create" => team::handle_create(ctx, pool, config, client).await,
        "team-join" => team::handle_join(ctx, pool, config, client).await,
        "submit" => submit::handle(ctx, pool, config).await,
        "leaderboard" => leaderboard::handle(ctx, pool, config, client).await,
        "schedule" => schedule::handle(ctx, pool, config, client).await,
        "help" => help::handle(ctx).await,
        _ => ephemeral_response("Unknown command. Use `/openhack help` to see available commands."),
    };

    if config.discord_log_interactions {
        let _ = crate::models::interaction::log_interaction(
            pool,
            &ctx.discord_user_id,
            "slash_command",
            Some(&subcommand),
            ctx.channel_id.as_deref(),
            ctx.guild_id.as_deref(),
            None,
            "dispatched",
        )
        .await;
    }

    result
}

use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Handle the `/openhack team-info` command.
///
/// # Expected Behavior
///
/// Requires a linked OpenHack account. Fetches the user's team info
/// from the OpenHack API and displays it as a rich embed with team
/// name, members, invite code, and project details. If the user is
/// not on a team, returns an ephemeral message. If the user is not
/// linked, returns an ephemeral "register first" message.
///
/// # Errors
///
/// Returns an ephemeral error message if the OpenHack API call fails.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
/// - Makes an HTTP GET to the OpenHack gateway.
pub async fn handle_info(
    ctx: &InteractionContext,
    pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
) -> serde_json::Value {
    let openhack_user_id = match super::require_linked_user(pool, ctx).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    match client.get_team(&openhack_user_id).await {
        Ok(team) => {
            let members: Vec<String> = team
                .members
                .iter()
                .map(|m| format!("- {} ({})", m.display_name, m.role))
                .collect();

            let project_info = team
                .project
                .as_ref()
                .map(|p| format!("**Project:** {} - {}", p.name, p.description))
                .unwrap_or_else(|| "No project submitted yet.".to_string());

            let invite_info = team
                .invite_code
                .as_ref()
                .map(|c| format!("**Invite Code:** `{c}`"))
                .unwrap_or_default();

            let embed = super::make_embed(
                &format!("Team: {}", team.name),
                &format!(
                    "{}\n\n**Members:**\n{}\n\n{}",
                    project_info,
                    members.join("\n"),
                    invite_info
                ),
                0x58_65_f2,
            );

            super::embed_response(embed)
        }
        Err(crate::errors::BotError::OpenHackError(msg))
            if msg.contains("404") || msg.contains("not found") =>
        {
            super::ephemeral_response(
                "You're not on a team yet. Use `/openhack team-create` or `/openhack team-join`.",
            )
        }
        Err(e) => {
            log::error!("Failed to get team: {e}");
            super::ephemeral_response(&format!("Failed to get team info: {e}"))
        }
    }
}

/// Handle the `/openhack team-create` command.
///
/// # Expected Behavior
///
/// Requires a linked OpenHack account. Extracts the team name from the
/// command options and creates a new team via the OpenHack API.
/// Displays a rich embed with the team name and invite code on success.
/// If the user is not linked, returns an ephemeral "register first" message.
///
/// # Errors
///
/// Returns an ephemeral error message if the team creation API call fails.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
/// - Makes an HTTP POST to the OpenHack gateway.
/// - Creates a team in the OpenHack database.
pub async fn handle_create(
    ctx: &InteractionContext,
    pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
) -> serde_json::Value {
    let openhack_user_id = match super::require_linked_user(pool, ctx).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    let team_name = super::get_string_option(ctx, "name").unwrap_or_else(|| "New Team".to_string());

    match client.create_team(&openhack_user_id, &team_name).await {
        Ok(team) => {
            let embed = super::make_embed(
                &format!("Team Created: {}", team.name),
                &format!(
                    "Your team has been created!\n\n**Invite Code:** `{}`\n\nShare this code with teammates so they can join with `/openhack team-join`.",
                    team.invite_code.as_deref().unwrap_or("N/A"),
                ),
                0x2e_cc_71,
            );

            super::embed_response(embed)
        }
        Err(e) => {
            log::error!("Failed to create team: {e}");
            super::ephemeral_response(&format!("Failed to create team: {e}"))
        }
    }
}

/// Handle the `/openhack team-join` command.
///
/// # Expected Behavior
///
/// Requires a linked OpenHack account. Extracts the invite code from
/// the command options and joins the team via the OpenHack API.
/// On success, displays a rich embed with team name and members.
/// If the invite code is empty, returns an ephemeral error message.
/// If the user is not linked, returns an ephemeral "register first" message.
///
/// # Errors
///
/// Returns an ephemeral error message if the team join API call fails.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
/// - Makes an HTTP POST to the OpenHack gateway.
/// - Adds the user to a team in the OpenHack database.
pub async fn handle_join(
    ctx: &InteractionContext,
    pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
) -> serde_json::Value {
    let openhack_user_id = match super::require_linked_user(pool, ctx).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    let invite_code = super::get_string_option(ctx, "code").unwrap_or_default();

    if invite_code.is_empty() {
        return super::ephemeral_response("Please provide an invite code.");
    }

    match client.join_team(&openhack_user_id, &invite_code).await {
        Ok(team) => {
            let members: Vec<String> = team
                .members
                .iter()
                .map(|m| format!("- {} ({})", m.display_name, m.role))
                .collect();

            let embed = super::make_embed(
                &format!("Joined Team: {}", team.name),
                &format!(
                    "You've joined the team!\n\n**Members:**\n{}",
                    members.join("\n")
                ),
                0x2e_cc_71,
            );
            super::embed_response(embed)
        }
        Err(e) => {
            log::error!("Failed to join team: {e}");
            super::ephemeral_response(&format!("Failed to join team: {e}"))
        }
    }
}

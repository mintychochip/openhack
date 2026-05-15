use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Handle a button component interaction.
///
/// # Expected Behavior
///
/// Dispatches button interactions based on the `custom_id` field extracted
/// from the interaction data. Returns a `serde_json::Value` interaction
/// response for each button action:
///
/// - `openhack:join-team:{team_id}`: requires linked account, joins team,
///   returns ephemeral success/error.
/// - `openhack:rsvp:{event_id}`: requires linked account, checks in,
///   returns ephemeral success/error.
/// - `openhack:submit`: opens the submit modal (type-9 response).
/// - `openhack:leaderboard`: shows the leaderboard embed.
/// - Unknown: logs a warning and returns an ephemeral error.
///
/// # Errors
///
/// Errors are logged and converted to ephemeral user-facing messages.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
/// - May make HTTP calls to OpenHack services.
pub async fn handle_button(
    pool: &PgPool,
    config: &Config,
    client: &OpenHackClient,
    ctx: &InteractionContext,
) -> serde_json::Value {
    let custom_id = super::super::commands::get_custom_id(ctx);

    if custom_id.starts_with("openhack:join-team:") {
        handle_join_team(pool, config, client, ctx, &custom_id).await
    } else if custom_id.starts_with("openhack:rsvp:") {
        handle_rsvp(pool, config, client, ctx, &custom_id).await
    } else if custom_id == "openhack:submit" {
        crate::commands::submit::handle(ctx, pool, config).await
    } else if custom_id == "openhack:leaderboard" {
        crate::commands::leaderboard::handle(ctx, pool, config, client).await
    } else {
        log::warn!("Unknown button custom_id: {custom_id}");
        crate::commands::ephemeral_response("Unknown button action.")
    }
}

async fn handle_join_team(
    pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
    ctx: &InteractionContext,
    custom_id: &str,
) -> serde_json::Value {
    let invite_code = custom_id.strip_prefix("openhack:join-team:").unwrap_or("");

    let openhack_user_id = match crate::commands::require_linked_user(pool, ctx).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    match client.join_team(&openhack_user_id, invite_code).await {
        Ok(team) => {
            crate::commands::ephemeral_response(&format!("You've joined **{}**!", team.name))
        }
        Err(e) => crate::commands::ephemeral_response(&format!("Failed to join team: {e}")),
    }
}

async fn handle_rsvp(
    pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
    ctx: &InteractionContext,
    custom_id: &str,
) -> serde_json::Value {
    let _event_id = custom_id.strip_prefix("openhack:rsvp:").unwrap_or("");

    let openhack_user_id = match crate::commands::require_linked_user(pool, ctx).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    match client.check_in(&openhack_user_id).await {
        Ok(_) => crate::commands::ephemeral_response("You've RSVP'd! See you there!"),
        Err(e) => crate::commands::ephemeral_response(&format!("RSVP failed: {e}")),
    }
}

/// Top-level component interaction dispatcher.
///
/// # Expected Behavior
///
/// Determines if the component is a button (component_type 2) or a select
/// menu (component_type 3+) by reading `data.component_type`, and
/// dispatches to the appropriate handler. Returns the handler's response
/// as a `serde_json::Value`.
///
/// # Errors
///
/// Errors are logged and converted to ephemeral messages by handlers.
///
/// # Side Effects
///
/// - Delegates to button or select handlers.
pub async fn handle_component(
    pool: &PgPool,
    config: &Config,
    client: &OpenHackClient,
    ctx: &InteractionContext,
) -> serde_json::Value {
    let component_type = ctx
        .data
        .as_ref()
        .and_then(|d| d.get("component_type"))
        .and_then(|v| v.as_u64());

    match component_type {
        Some(2) => handle_button(pool, config, client, ctx).await,
        Some(3) | Some(5) | Some(6) | Some(7) | Some(8) => {
            crate::components::selects::handle_select(pool, config, client, ctx).await
        }
        _ => {
            log::warn!("Unknown component_type: {component_type:?}");
            crate::commands::ephemeral_response("Unknown component type.")
        }
    }
}

/// Top-level modal interaction dispatcher.
///
/// # Expected Behavior
///
/// Dispatches modal submissions based on the `custom_id` field extracted
/// from the interaction data. Currently handles `openhack:submit` by
/// delegating to `modals::handle_submit_modal`. Unknown modals are
/// logged and receive an ephemeral error response.
///
/// # Errors
///
/// Errors are logged and converted to ephemeral messages by handlers.
///
/// # Side Effects
///
/// - May make HTTP calls to OpenHack services.
/// - May create project submissions in the OpenHack database.
pub async fn handle_modal(
    pool: &PgPool,
    config: &Config,
    client: &OpenHackClient,
    ctx: &InteractionContext,
) -> serde_json::Value {
    let custom_id = crate::commands::get_custom_id(ctx);

    match custom_id.as_str() {
        "openhack:submit" => {
            crate::components::modals::handle_submit_modal(pool, config, client, ctx).await
        }
        _ => {
            log::warn!("Unknown modal custom_id: {custom_id}");
            crate::commands::ephemeral_response("Unknown modal.")
        }
    }
}

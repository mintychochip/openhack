use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;

/// Handle the `/openhack register` command.
///
/// # Expected Behavior
///
/// Checks if the Discord user already has a linked OpenHack account.
/// If already linked, returns an ephemeral message confirming the existing
/// link. If not linked, creates a one-time link token (expires in 15 min),
/// spawns a background task to DM the user a link URL, and returns an
/// ephemeral confirmation in the channel.
///
/// # Errors
///
/// Database errors or link token creation failures result in ephemeral
/// error messages to the user.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
/// - Writes to `discord_bot.link_tokens` table.
/// - Spawns a background Tokio task that sends a DM to the Discord user
///   (HTTP POST to Discord API, fire-and-forget with error logging).
pub async fn handle(ctx: &InteractionContext, pool: &PgPool, config: &Config) -> serde_json::Value {
    let discord_user_id = ctx.discord_user_id.clone();

    match crate::models::user_link::find_by_discord_id(pool, &discord_user_id).await {
        Ok(Some(link)) => {
            return super::ephemeral_response(&format!(
                "Your Discord account is already linked to OpenHack user `{}`. \
                 Contact an admin if you need to change this.",
                link.openhack_user_id
            ));
        }
        Ok(None) => {}
        Err(e) => {
            log::error!("Failed to look up user link: {e}");
            return super::ephemeral_response("An error occurred. Please try again.");
        }
    }

    let link_token = match crate::models::link_token::create(pool, &discord_user_id).await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to create link token: {e}");
            return super::ephemeral_response("An error occurred. Please try again.");
        }
    };

    let cfg = config.clone();
    let user_id = discord_user_id.clone();
    let domain = config.openhack_domain.clone();
    tokio::spawn(async move {
        let link_url = format!("{domain}/link-discord?token={}", link_token.token);
        let dm_content = format!(
            "Link your OpenHack account by opening this URL:\n\
             {link_url}\n\n\
             This link expires in 15 minutes. If you didn't request this, ignore this message."
        );
        let client = crate::openhack_client::OpenHackClient::new(
            &cfg.gateway_url,
            &cfg.lambda_internal_token,
        );
        if let Err(e) = client
            .send_dm(&cfg.discord_bot_token, &user_id, &dm_content, None)
            .await
        {
            log::error!("Failed to send DM: {e}");
        }
    });

    super::ephemeral_response(
        "Check your DMs for a link to connect your OpenHack account! \
         The link expires in 15 minutes.",
    )
}

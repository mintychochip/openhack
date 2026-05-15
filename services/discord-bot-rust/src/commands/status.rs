use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Handle the `/openhack status` command.
///
/// # Expected Behavior
///
/// Fetches the current hackathon status from the OpenHack API and
/// displays it as a rich embed with hackathon name, current phase,
/// and participant/team/project counts. If the API is unreachable,
/// returns an ephemeral error message.
///
/// # Errors
///
/// Returns an ephemeral error message if the OpenHack API call fails.
///
/// # Side Effects
///
/// - Makes an HTTP GET to the OpenHack gateway.
pub async fn handle(
    _ctx: &InteractionContext,
    _pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
) -> serde_json::Value {
    match client.get_hackathon_status().await {
        Ok(status) => {
            let embed = super::make_embed(
                &status.name,
                &format!(
                    "**Current Phase:** {}\n**Participants:** {}\n**Teams:** {}\n**Projects:** {}",
                    status.current_phase.as_deref().unwrap_or("Not started"),
                    status.participant_count,
                    status.team_count,
                    status.project_count,
                ),
                0x00_ff_ff,
            );

            super::embed_response(embed)
        }
        Err(e) => {
            log::error!("Failed to get hackathon status: {e}");
            super::ephemeral_response(&format!("Failed to get hackathon status: {e}"))
        }
    }
}

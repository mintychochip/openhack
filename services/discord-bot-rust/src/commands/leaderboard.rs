use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Handle the `/openhack leaderboard` command.
///
/// # Expected Behavior
///
/// Fetches leaderboard entries from the OpenHack API and displays them
/// as a rich embed. The `top` option defaults to 10, clamped to 1–25.
/// If the API call fails, returns an ephemeral error message.
///
/// # Errors
///
/// Returns an ephemeral error message if the OpenHack API call fails.
///
/// # Side Effects
///
/// - Makes an HTTP GET to the OpenHack gateway.
pub async fn handle(
    ctx: &InteractionContext,
    _pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
) -> serde_json::Value {
    let top: u32 = super::get_int_option(ctx, "top")
        .map(|v| v.clamp(1, 25) as u32)
        .unwrap_or(10);

    match client.get_leaderboard(top).await {
        Ok(entries) => {
            let leaderboard_text = if entries.is_empty() {
                "No entries yet.".to_string()
            } else {
                entries
                    .iter()
                    .map(|e| {
                        let project = e
                            .project_name
                            .as_ref()
                            .map(|p| format!(" - {p}"))
                            .unwrap_or_default();
                        format!(
                            "**{}.** {} ({:.1}){}",
                            e.rank, e.team_name, e.score, project
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            let embed = super::make_embed(
                &format!("Leaderboard (Top {top})"),
                &leaderboard_text,
                0xff_d7_00,
            );

            super::embed_response(embed)
        }
        Err(e) => {
            log::error!("Failed to get leaderboard: {e}");
            super::ephemeral_response(&format!("Failed to get leaderboard: {e}"))
        }
    }
}

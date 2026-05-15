use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Handle the `/openhack schedule` command.
///
/// # Expected Behavior
///
/// Fetches upcoming events from the OpenHack API and displays them
/// as a rich embed. Each event shows name, type, and timing. If the
/// API call fails, returns an ephemeral error message.
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
    match client.get_schedule().await {
        Ok(events) => {
            let schedule_text = if events.is_empty() {
                "No upcoming events scheduled.".to_string()
            } else {
                events
                    .iter()
                    .map(|e| {
                        let start = e.starts_at.clone().unwrap_or_else(|| "TBD".to_string());
                        let end = e
                            .ends_at
                            .as_ref()
                            .map(|s| format!(" - {s}"))
                            .unwrap_or_default();
                        format!("**{}** (`{}`)\n{}{}", e.name, e.event_type, start, end)
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            };

            let embed = super::make_embed("Upcoming Schedule", &schedule_text, 0x99_33_ff);

            super::embed_response(embed)
        }
        Err(e) => {
            log::error!("Failed to get schedule: {e}");
            super::ephemeral_response(&format!("Failed to get schedule: {e}"))
        }
    }
}

use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Handle a select menu component interaction.
///
/// # Expected Behavior
///
/// Dispatches select menu interactions based on the `custom_id` field
/// extracted from the interaction data. Currently returns an ephemeral
/// confirmation response. No backend persistence is implemented yet;
/// selections are acknowledged but not saved.
///
/// - `openhack:role-select`: acknowledges role selection (not persisted).
/// - `openhack:event-filter`: acknowledges filter selection (not persisted).
/// - Unknown: logs a warning and returns an ephemeral error.
///
/// # Errors
///
/// Returns ephemeral error messages for unknown custom IDs.
///
/// # Side Effects
///
/// None beyond the response. No database or API calls.
pub async fn handle_select(
    _pool: &PgPool,
    _config: &Config,
    _client: &OpenHackClient,
    ctx: &InteractionContext,
) -> serde_json::Value {
    let custom_id = crate::commands::get_custom_id(ctx);

    match custom_id.as_str() {
        "openhack:role-select" => {
            let selected = crate::commands::get_selected_values(ctx).first().cloned();

            if let Some(role) = selected {
                crate::commands::ephemeral_response(&format!(
                    "You selected role: **{role}**. (Not yet saved — coming soon!)"
                ))
            } else {
                crate::commands::ephemeral_response("No role selected.")
            }
        }
        "openhack:event-filter" => {
            let selected = crate::commands::get_selected_values(ctx).first().cloned();

            if let Some(filter) = selected {
                crate::commands::ephemeral_response(&format!(
                    "Filtering announcements by: **{filter}** (Not yet saved — coming soon!)"
                ))
            } else {
                crate::commands::ephemeral_response("No filter selected.")
            }
        }
        _ => {
            log::warn!("Unknown select menu custom_id: {custom_id}");
            crate::commands::ephemeral_response("Unknown select menu.")
        }
    }
}

use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;

/// Handle the `/openhack submit` command.
///
/// # Expected Behavior
///
/// Requires a linked OpenHack account. Opens a Discord modal with three
/// text inputs: project name (short, required), description (paragraph,
/// required), and optional repo URL (short, optional). The modal
/// submission is handled by `components::handle_modal`.
///
/// Returns a type-9 (MODAL) interaction response with the modal definition.
/// If the user is not linked, returns an ephemeral message instead.
///
/// # Errors
///
/// Returns an ephemeral error message if the user is not linked.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn handle(
    ctx: &InteractionContext,
    pool: &PgPool,
    _config: &Config,
) -> serde_json::Value {
    let _openhack_user_id = match super::require_linked_user(pool, ctx).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    serde_json::json!({
        "type": 9,
        "data": {
            "custom_id": "openhack:submit",
            "title": "Submit Project",
            "components": [
                {
                    "type": 1,
                    "components": [
                        {
                            "type": 4,
                            "custom_id": "project_name",
                            "style": 1,
                            "label": "Project Name",
                            "min_length": 1,
                            "max_length": 100,
                            "placeholder": "My Awesome Project",
                            "required": true
                        }
                    ]
                },
                {
                    "type": 1,
                    "components": [
                        {
                            "type": 4,
                            "custom_id": "project_description",
                            "style": 2,
                            "label": "Description",
                            "min_length": 10,
                            "max_length": 1000,
                            "placeholder": "Describe your project...",
                            "required": true
                        }
                    ]
                },
                {
                    "type": 1,
                    "components": [
                        {
                            "type": 4,
                            "custom_id": "project_repo_url",
                            "style": 1,
                            "label": "Repository URL (optional)",
                            "max_length": 500,
                            "placeholder": "https://github.com/...",
                            "required": false
                        }
                    ]
                }
            ]
        }
    })
}

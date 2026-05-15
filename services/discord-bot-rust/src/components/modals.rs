use sqlx::PgPool;

use crate::config::Config;
use crate::interaction::InteractionContext;
use crate::openhack_client::OpenHackClient;

/// Handle the project submission modal.
///
/// # Expected Behavior
///
/// Extracts the text input values from the modal submission:
/// `project_name`, `project_description`, and `project_repo_url`
/// (optional). Requires a linked OpenHack account. Fetches the user's
/// team, then calls the OpenHack API to submit the project. Returns
/// an ephemeral success or error message.
///
/// # Errors
///
/// Returns ephemeral error messages if: user is not linked, project
/// name/description are empty, team lookup fails, or submission fails.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
/// - Makes HTTP calls to OpenHack services.
/// - Creates a project submission in the OpenHack database.
pub async fn handle_submit_modal(
    pool: &PgPool,
    _config: &Config,
    client: &OpenHackClient,
    ctx: &InteractionContext,
) -> serde_json::Value {
    let openhack_user_id = match crate::commands::require_linked_user(pool, ctx).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    let project_name = crate::commands::get_modal_value(ctx, "project_name").unwrap_or_default();
    let project_description =
        crate::commands::get_modal_value(ctx, "project_description").unwrap_or_default();
    let project_repo_url = crate::commands::get_modal_value(ctx, "project_repo_url");

    if project_name.is_empty() || project_description.is_empty() {
        return crate::commands::ephemeral_response("Project name and description are required.");
    }

    let team = match client.get_team(&openhack_user_id).await {
        Ok(t) => t,
        Err(e) => {
            return crate::commands::ephemeral_response(&format!("Failed to get your team: {e}"));
        }
    };

    match client
        .submit_project(
            &team.id,
            &project_name,
            &project_description,
            project_repo_url.as_deref(),
        )
        .await
    {
        Ok(project) => crate::commands::ephemeral_response(&format!(
            "Project **{}** submitted successfully!\n\n{}",
            project.name, project.description
        )),
        Err(e) => crate::commands::ephemeral_response(&format!("Project submission failed: {e}")),
    }
}

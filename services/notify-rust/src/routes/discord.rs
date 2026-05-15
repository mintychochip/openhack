use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

use crate::config::Config;
use crate::middleware::auth::AuthUser;
use crate::services::discord;

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordWebhookRequest {
    pub url: Option<String>,
    pub content: String,
    pub username: Option<String>,
    pub embeds: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordRoleAssignRequest {
    pub user_id: String,
    pub role_id: String,
}

/// Send a Discord webhook message.
///
/// # Expected Behavior
///
/// Accepts a JSON body with content, optional url (overrides config), optional
/// username, and optional embeds. Sends the webhook via the Discord service.
/// Requires authentication.
///
/// # Errors
///
/// Returns 400 if the request body is malformed. Returns 502 on Discord failure.
///
/// # Side Effects
///
/// - Makes HTTP POST to Discord webhook URL.
pub async fn send_discord_webhook(
    pool: web::Data<PgPool>,
    req: web::Json<DiscordWebhookRequest>,
    _user: web::ReqData<AuthUser>,
    config: web::Data<Config>,
) -> HttpResponse {
    let webhook_url = req.url.as_deref().unwrap_or(&config.discord_webhook_url);

    match discord::send_discord_webhook(
        pool.get_ref(),
        webhook_url,
        &req.content,
        req.username.as_deref(),
        req.embeds.as_ref(),
    )
    .await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => e.to_http_response(),
    }
}

/// Assign a Discord role to a user.
///
/// # Expected Behavior
///
/// Accepts a JSON body with `user_id` and `role_id`. Uses the configured
/// `DISCORD_BOT_TOKEN` and `DISCORD_GUILD_ID` to assign the role via
/// the Discord API. Requires authentication.
///
/// # Errors
///
/// Returns 400 if the request body is malformed. Returns 502 on Discord failure.
///
/// # Side Effects
///
/// - Makes HTTP PUT to Discord API.
pub async fn assign_discord_role(
    pool: web::Data<PgPool>,
    req: web::Json<DiscordRoleAssignRequest>,
    _user: web::ReqData<AuthUser>,
    config: web::Data<Config>,
) -> HttpResponse {
    match discord::assign_discord_role(
        pool.get_ref(),
        &config.discord_bot_token,
        &config.discord_guild_id,
        &req.user_id,
        &req.role_id,
    )
    .await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => e.to_http_response(),
    }
}

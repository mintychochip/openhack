use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::BotError;
use crate::middleware::auth::AuthUser;
use crate::models::channel_config::{self, CreateChannelConfigRequest, UpdateChannelConfigRequest};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn require_admin(user: &AuthUser) -> Option<HttpResponse> {
    if user.is_admin() {
        None
    } else {
        Some(HttpResponse::Forbidden().json(serde_json::json!({"error": "Admin access required"})))
    }
}

/// List channel configurations.
///
/// # Expected Behavior
///
/// Returns all channel configurations for the configured guild,
/// ordered by event_type. Supports pagination via limit/offset
/// query parameters (defaults: limit=50, offset=0).
/// Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.channel_config` table.
pub async fn list_channels(
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<crate::config::Config>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }
    match channel_config::list_by_guild(pool.get_ref(), &config.discord_guild_id).await {
        Ok(channels) => HttpResponse::Ok().json(channels),
        Err(e) => BotError::from(e).to_http_response(),
    }
}

/// Create a channel configuration.
///
/// # Expected Behavior
///
/// Accepts event_type, channel_id, and guild_id in the request body.
/// Creates a new mapping from event type to Discord channel. Returns
/// the created configuration. Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 400 on constraint violation.
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Writes to `discord_bot.channel_config` table.
pub async fn create_channel(
    pool: web::Data<sqlx::PgPool>,
    req: web::Json<CreateChannelConfigRequest>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }
    match channel_config::create(pool.get_ref(), &req).await {
        Ok(cfg) => HttpResponse::Created().json(cfg),
        Err(e) => BotError::from(e).to_http_response(),
    }
}

/// Update a channel configuration.
///
/// # Expected Behavior
///
/// Updates the specified channel config by ID. Only provided fields
/// are updated (partial update). Returns the updated configuration.
/// Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 404 if the channel config ID
/// doesn't exist. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Writes to `discord_bot.channel_config` table.
pub async fn update_channel(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateChannelConfigRequest>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }
    let id = path.into_inner();
    match channel_config::update(pool.get_ref(), id, &req).await {
        Ok(cfg) => HttpResponse::Ok().json(cfg),
        Err(e) => BotError::from(e).to_http_response(),
    }
}

/// Delete a channel configuration.
///
/// # Expected Behavior
///
/// Deletes the channel config with the given ID. Returns 200 on success,
/// 404 if not found. Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 404 if the ID doesn't exist.
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Deletes from `discord_bot.channel_config` table.
pub async fn delete_channel(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }
    let id = path.into_inner();
    match channel_config::delete(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Channel config not found"}))
        }
        Err(e) => BotError::from(e).to_http_response(),
    }
}

/// List user links.
///
/// # Expected Behavior
///
/// Returns paginated Discord↔OpenHack user link records ordered by
/// linked_at descending. Supports limit/offset query parameters.
/// Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn list_users(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ListQuery>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    match crate::models::user_link::list(pool.get_ref(), limit, offset).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => BotError::from(e).to_http_response(),
    }
}

/// Unlink a Discord account.
///
/// # Expected Behavior
///
/// Deletes the user link for the given Discord user ID. Returns 200
/// on success, 404 if no link exists. Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 404 if no link exists for the
/// Discord user. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Deletes from `discord_bot.user_links` table.
pub async fn unlink_user(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<String>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }
    let discord_id = path.into_inner();
    match crate::auth::link::unlink(pool.get_ref(), &discord_id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"unlinked": true})),
        Err(e) => e.to_http_response(),
    }
}

/// List interaction logs.
///
/// # Expected Behavior
///
/// Returns paginated interaction log entries ordered by created_at
/// descending. Supports limit/offset query parameters.
/// Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.interactions` table.
pub async fn list_interactions(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ListQuery>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    match crate::models::interaction::list(pool.get_ref(), limit, offset).await {
        Ok(interactions) => HttpResponse::Ok().json(interactions),
        Err(e) => BotError::from(e).to_http_response(),
    }
}

/// Send an announcement to a configured Discord channel.
///
/// # Expected Behavior
///
/// Accepts event_type and content in the request body. Looks up the
/// channel configuration for the event type and sends the content
/// as a Discord message. Returns 404 if no channel is configured
/// for the event type. Requires admin authentication.
///
/// # Errors
///
/// Returns 403 if not admin. Returns 404 if no channel is configured.
/// Returns 502 if Discord API call fails. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.channel_config` table.
/// - Makes HTTP POST to Discord API.
#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    pub event_type: String,
    pub content: String,
}

pub async fn announce(
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<crate::config::Config>,
    openhack_client: web::Data<crate::openhack_client::OpenHackClient>,
    req: web::Json<AnnounceRequest>,
    user: AuthUser,
) -> HttpResponse {
    if let Some(resp) = require_admin(&user) {
        return resp;
    }

    let channel_config = match crate::models::channel_config::find_by_event_type(
        pool.get_ref(),
        &req.event_type,
        &config.discord_guild_id,
    )
    .await
    {
        Ok(Some(cfg)) => cfg,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("No channel configured for event type '{}'", req.event_type)
            }));
        }
        Err(e) => return BotError::from(e).to_http_response(),
    };

    match openhack_client
        .send_channel_message(
            &config.discord_bot_token,
            &channel_config.channel_id,
            &req.content,
            None,
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "sent",
            "channel_id": channel_config.channel_id
        })),
        Err(e) => e.to_http_response(),
    }
}

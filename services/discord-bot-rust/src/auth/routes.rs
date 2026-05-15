use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::auth::AuthUser;

#[derive(Debug, Deserialize)]
pub struct InitiateLinkRequest {
    pub discord_user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmLinkRequest {
    pub token: String,
    pub openhack_user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct LinkStatusQuery {
    pub discord_id: Option<String>,
    pub openhack_id: Option<String>,
}

/// Initiate a Discord account link.
///
/// # Expected Behavior
///
/// Accepts a `discord_user_id` in the request body. Creates a one-time
/// link token and returns it. The token should be embedded in a URL
/// for the user to open in their browser. Requires authentication.
///
/// # Errors
///
/// Returns 400 if the Discord user is already linked. Returns 502 on
/// internal errors.
///
/// # Side Effects
///
/// - Writes to `discord_bot.link_tokens` table.
/// - Reads from `discord_bot.user_links` table.
pub async fn initiate_link(
    pool: web::Data<sqlx::PgPool>,
    req: web::Json<InitiateLinkRequest>,
    _user: AuthUser,
) -> HttpResponse {
    match crate::auth::link::initiate_link(pool.get_ref(), &req.discord_user_id).await {
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "expires_in": 900
        })),
        Err(e) => e.to_http_response(),
    }
}

/// Confirm a Discord account link.
///
/// # Expected Behavior
///
/// Accepts a `token` and `openhack_user_id` in the request body.
/// Validates the token, claims it, and creates the user link.
/// Called from the frontend after the user opens the link URL.
/// Requires authentication.
///
/// # Errors
///
/// Returns 410 if the token is expired/invalid/claimed. Returns 400
/// if the OpenHack user is already linked.
///
/// # Side Effects
///
/// - Reads and updates `discord_bot.link_tokens` table.
/// - Writes to `discord_bot.user_links` table.
pub async fn confirm_link(
    pool: web::Data<sqlx::PgPool>,
    req: web::Json<ConfirmLinkRequest>,
    _user: AuthUser,
) -> HttpResponse {
    match crate::auth::link::confirm_link(pool.get_ref(), &req.token, req.openhack_user_id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "linked"
        })),
        Err(e) => e.to_http_response(),
    }
}

/// Check the link status for a Discord or OpenHack user.
///
/// # Expected Behavior
///
/// Accepts optional `discord_id` or `openhack_id` query parameters.
/// Returns the link status including the linked user IDs if found.
/// Requires authentication.
///
/// # Errors
///
/// Returns 400 if neither query parameter is provided.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn get_link_status(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<LinkStatusQuery>,
    _user: AuthUser,
) -> HttpResponse {
    if let Some(ref discord_id) = query.discord_id {
        match crate::auth::link::get_link_status(pool.get_ref(), discord_id).await {
            Ok(Some(openhack_id)) => HttpResponse::Ok().json(serde_json::json!({
                "linked": true,
                "discord_user_id": discord_id,
                "openhack_user_id": openhack_id.to_string()
            })),
            Ok(None) => HttpResponse::Ok().json(serde_json::json!({
                "linked": false,
                "discord_user_id": discord_id
            })),
            Err(e) => e.to_http_response(),
        }
    } else if let Some(ref openhack_id_str) = query.openhack_id {
        let openhack_id = match openhack_id_str.parse::<Uuid>() {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Invalid openhack_id"}))
            }
        };
        match crate::models::user_link::find_by_openhack_id(pool.get_ref(), openhack_id).await {
            Ok(Some(link)) => HttpResponse::Ok().json(serde_json::json!({
                "linked": true,
                "discord_user_id": link.discord_user_id,
                "openhack_user_id": openhack_id.to_string()
            })),
            Ok(None) => HttpResponse::Ok().json(serde_json::json!({
                "linked": false,
                "openhack_user_id": openhack_id.to_string()
            })),
            Err(e) => crate::errors::BotError::from(e).to_http_response(),
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Provide either discord_id or openhack_id query parameter"
        }))
    }
}

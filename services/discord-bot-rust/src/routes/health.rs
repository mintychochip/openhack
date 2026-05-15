use actix_web::{web, HttpResponse};

use crate::BotState;

/// Bot health and connection status endpoint.
///
/// # Expected Behavior
///
/// Returns the bot's connection status and guild ID.
/// Provides more detail than the standard /health endpoint.
///
/// # Errors
///
/// None. Always returns 200.
///
/// # Side Effects
///
/// - Reads from `BotState` in-memory state.
pub async fn bot_health(state: web::Data<BotState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "discord-bot",
        "bot_connected": state.connected.load(std::sync::atomic::Ordering::Relaxed),
        "guild_id": state.guild_id,
    }))
}

/// Bot status endpoint with detailed connection info.
///
/// # Expected Behavior
///
/// Same as `bot_health` but with a "status" wrapper for
/// compatibility with monitoring tools.
///
/// # Errors
///
/// None. Always returns 200.
///
/// # Side Effects
///
/// - Reads from `BotState` in-memory state.
pub async fn bot_status(state: web::Data<BotState>) -> HttpResponse {
    let connected = state.connected.load(std::sync::atomic::Ordering::Relaxed);
    HttpResponse::Ok().json(serde_json::json!({
        "status": if connected { "connected" } else { "disconnected" },
        "service": "discord-bot",
        "bot_connected": connected,
        "guild_id": state.guild_id,
    }))
}

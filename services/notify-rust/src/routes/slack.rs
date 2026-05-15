use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

use crate::config::Config;
use crate::middleware::auth::AuthUser;
use crate::services::slack;

#[derive(Debug, Clone, Deserialize)]
pub struct SlackWebhookRequest {
    pub url: String,
    pub text: String,
    pub blocks: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackChannelMessageRequest {
    pub channel: String,
    pub text: String,
    pub blocks: Option<serde_json::Value>,
}

/// Send a Slack incoming webhook message.
///
/// # Expected Behavior
///
/// Accepts a JSON body with url, text, and optional blocks. Sends the
/// message via the Slack webhook. Requires authentication.
///
/// # Errors
///
/// Returns 400 if the request body is malformed. Returns 502 on Slack failure.
///
/// # Side Effects
///
/// - Makes HTTP POST to Slack webhook URL.
pub async fn send_slack_webhook(
    pool: web::Data<PgPool>,
    req: web::Json<SlackWebhookRequest>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    match slack::send_slack_webhook(pool.get_ref(), &req.url, &req.text, req.blocks.as_ref()).await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => e.to_http_response(),
    }
}

/// Send a message to a Slack channel using the Bot API.
///
/// # Expected Behavior
///
/// Accepts a JSON body with channel, text, and optional blocks. Uses
/// the configured `SLACK_BOT_TOKEN` to post via chat.postMessage.
/// Requires authentication.
///
/// # Errors
///
/// Returns 400 if the request body is malformed. Returns 502 on Slack failure.
///
/// # Side Effects
///
/// - Makes HTTP POST to Slack API.
pub async fn send_slack_channel_message(
    pool: web::Data<PgPool>,
    req: web::Json<SlackChannelMessageRequest>,
    _user: web::ReqData<AuthUser>,
    config: web::Data<Config>,
) -> HttpResponse {
    match slack::send_slack_channel_message(
        pool.get_ref(),
        &config.slack_bot_token,
        &req.channel,
        &req.text,
        req.blocks.as_ref(),
    )
    .await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => e.to_http_response(),
    }
}

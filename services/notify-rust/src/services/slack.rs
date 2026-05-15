use sqlx::PgPool;
use std::time::Duration;

use crate::errors::NotifyError;

/// Send a message to a Slack channel via incoming webhook URL.
///
/// # Expected Behavior
///
/// Posts a JSON payload to the Slack webhook URL with `text` and optional
/// `blocks` fields. Uses a 15-second timeout. Returns the Slack response
/// on success.
///
/// # Errors
///
/// Returns `NotifyError::SlackError` if the webhook URL is empty or
/// the HTTP request fails with a non-2xx status.
///
/// # Side Effects
///
/// - Makes an HTTP POST request to the Slack webhook URL (network I/O).
/// - Logs at INFO on success, ERROR on failure.
pub async fn send_slack_webhook(
    _pool: &PgPool,
    webhook_url: &str,
    text: &str,
    blocks: Option<&serde_json::Value>,
) -> Result<serde_json::Value, NotifyError> {
    if webhook_url.is_empty() {
        return Err(NotifyError::SlackError(
            "Slack webhook URL is required".into(),
        ));
    }

    let mut body = serde_json::json!({
        "text": text,
    });

    if let Some(b) = blocks {
        body["blocks"] = b.clone();
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let response = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| NotifyError::SlackError(format!("Request failed: {e}")))?;

    let status = response.status();
    let resp_body = response.text().await.unwrap_or_default();

    if status.is_success() {
        log::info!("Slack webhook sent successfully");
        Ok(serde_json::json!({
            "status": "sent",
            "response": resp_body
        }))
    } else {
        log::error!("Slack webhook failed with status {status}: {resp_body}");
        Err(NotifyError::SlackError(format!(
            "Slack returned {status}: {resp_body}"
        )))
    }
}

/// Send a message to a Slack channel using the Bot API.
///
/// # Expected Behavior
///
/// Uses the Slack Chat.postMessage API to send a message to a specific
/// channel. Requires `SLACK_BOT_TOKEN` to be configured. Makes a POST
/// request to `https://slack.com/api/chat.postMessage` with the channel,
/// text, and optional blocks. Returns the API response JSON.
///
/// # Errors
///
/// Returns `NotifyError::SlackError` if the bot token is not configured,
/// the HTTP request fails, or the Slack API returns `ok: false`.
///
/// # Side Effects
///
/// - Makes an HTTP POST request to the Slack API (network I/O).
/// - Posts a message to a Slack channel (external state change).
/// - Logs at INFO on success, ERROR on failure.
pub async fn send_slack_channel_message(
    _pool: &PgPool,
    bot_token: &str,
    channel: &str,
    text: &str,
    blocks: Option<&serde_json::Value>,
) -> Result<serde_json::Value, NotifyError> {
    if bot_token.is_empty() {
        return Err(NotifyError::SlackError(
            "SLACK_BOT_TOKEN not configured".into(),
        ));
    }

    let mut body = serde_json::json!({
        "channel": channel,
        "text": text,
    });

    if let Some(b) = blocks {
        body["blocks"] = b.clone();
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let response = client
        .post("https://slack.com/api/chat.postMessage")
        .header("Authorization", format!("Bearer {bot_token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| NotifyError::SlackError(format!("Request failed: {e}")))?;

    let status = response.status();
    let resp_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log::error!("Slack API HTTP error: {status} - {resp_text}");
        return Err(NotifyError::SlackError(format!(
            "Slack API HTTP error: {status}"
        )));
    }

    let resp_json: serde_json::Value = serde_json::from_str(&resp_text).unwrap_or_default();

    if resp_json["ok"].as_bool().unwrap_or(false) {
        log::info!("Slack channel message sent to {channel}");
        Ok(resp_json)
    } else {
        let error = resp_json["error"].as_str().unwrap_or("unknown");
        log::error!("Slack API error: {error}");
        Err(NotifyError::SlackError(format!("Slack API error: {error}")))
    }
}

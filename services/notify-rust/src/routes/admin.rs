use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::config::Config;
use crate::events::subscriber::handle_event;
use crate::services::webhook::retry_pending_deliveries;

#[derive(Debug, Deserialize)]
pub struct ProcessEventRequest {
    pub channel: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Process a single event via HTTP, simulating a Redis pub/sub message.
///
/// # Expected Behavior
///
/// Accepts a JSON body containing `channel`, `event_type`, and `payload`.
/// Merges `event_type` into the payload object under the key `event_type`
/// (overwriting if present), serializes the merged object to a JSON string,
/// and delegates to `handle_event` — the same function used by the Redis
/// pub/sub subscriber. Returns 200 with a JSON acknowledgement on success.
/// Returns 400 if the merged payload cannot be serialized to JSON.
///
/// # Errors
///
/// Returns 400 Bad Request if the request body is malformed or if the
/// merged payload cannot be serialized. Individual delivery failures
/// within `handle_event` are logged but do not produce HTTP errors.
///
/// # Side Effects
///
/// - Reads from `notify.webhooks` table (via `handle_event`).
/// - Writes to `notify.webhook_deliveries` table (via `handle_event`).
/// - Makes HTTP POST requests to matching webhook URLs (via `handle_event`).
/// - May send Discord notification for notable events (via `handle_event`).
/// - Logs at INFO on successful dispatch, WARN/ERROR on failures within
///   `handle_event`.
pub async fn process_event(
    pool: web::Data<sqlx::postgres::PgPool>,
    config: web::Data<Config>,
    body: web::Json<ProcessEventRequest>,
) -> HttpResponse {
    let mut merged = body.payload.clone();
    merged["event_type"] = serde_json::Value::String(body.event_type.clone());

    let message = match serde_json::to_string(&merged) {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to serialize event payload: {e}");
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to serialize event payload"
            }));
        }
    };

    handle_event(
        pool.get_ref(),
        &body.channel,
        &message,
        &config.discord_webhook_url,
    )
    .await;

    HttpResponse::Ok().json(serde_json::json!({
        "status": "processed",
        "channel": body.channel,
        "event_type": body.event_type
    }))
}

/// Run the webhook retry logic once via HTTP.
///
/// # Expected Behavior
///
/// Delegates to `retry_pending_deliveries` — the same function used by the
/// 30-second background retry loop. Processes all pending webhook deliveries
/// whose `next_retry_at` is in the past (up to 50 per call). Returns 200
/// with a JSON acknowledgement.
///
/// # Errors
///
/// Individual delivery failures are logged by `retry_pending_deliveries`
/// but do not produce HTTP errors. Always returns 200.
///
/// # Side Effects
///
/// - Reads from `notify.webhook_deliveries` table (via `retry_pending_deliveries`).
/// - Makes HTTP POST requests to webhook URLs (via `retry_pending_deliveries`).
/// - Updates delivery records in the database (via `retry_pending_deliveries`).
/// - Logs at INFO on successful retry, WARN on retryable failure.
pub async fn retry_webhooks(pool: web::Data<sqlx::postgres::PgPool>) -> HttpResponse {
    retry_pending_deliveries(pool.get_ref()).await;

    HttpResponse::Ok().json(serde_json::json!({
        "status": "completed"
    }))
}

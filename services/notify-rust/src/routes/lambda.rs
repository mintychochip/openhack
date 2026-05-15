use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::postgres::PgPool;

use crate::config::Config;
use crate::events::subscriber::handle_event;
use crate::services::webhook::retry_pending_deliveries;

#[derive(Deserialize)]
struct LambdaEventInput {
    path: Option<String>,
    #[allow(dead_code)]
    method: Option<String>,
    channel: Option<String>,
    event_type: Option<String>,
    payload: Option<serde_json::Value>,
}

/// Lambda event dispatcher endpoint for serverless routing.
///
/// # Expected Behavior
///
/// Receives Lambda-routed events from AWS Lambda Web Adapter. Parses the
/// JSON body for a `path` field. Supported paths:
/// - `/api/notify/admin/retry-webhooks` — triggers webhook retry
/// - `/api/notify/admin/process-event` — processes an event (requires
///   `channel`, `event_type`, `payload` in the body alongside `path`)
///
/// # Errors
///
/// Returns 400 if the event body cannot be parsed or required fields are missing.
///
/// # Side Effects
///
/// - May trigger webhook retry (database reads/writes, HTTP POST to webhooks).
/// - May process an event (database reads/writes, HTTP POST, Discord).
/// - Logs at INFO on dispatch, WARN on unknown path.
pub async fn dispatch(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let input: LambdaEventInput = match serde_json::from_value(body.into_inner()) {
        Ok(i) => i,
        Err(e) => {
            log::warn!("Failed to parse Lambda event input: {e}");
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_input",
                "message": format!("Failed to parse Lambda event: {e}")
            }));
        }
    };

    let Some(path) = input.path else {
        log::warn!("Lambda event missing 'path' field");
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "missing_path",
            "message": "Lambda event must include a 'path' field"
        }));
    };

    match path.as_str() {
        "/api/notify/admin/retry-webhooks" => {
            log::info!("Lambda dispatch: retrying webhooks");
            retry_pending_deliveries(pool.get_ref()).await;
            HttpResponse::Ok().json(serde_json::json!({ "status": "completed" }))
        }
        "/api/notify/admin/process-event" => {
            log::info!("Lambda dispatch: processing event");
            let Some(channel) = input.channel else {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "missing_channel",
                    "message": "process-event requires 'channel' field"
                }));
            };
            let Some(event_type) = input.event_type else {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "missing_event_type",
                    "message": "process-event requires 'event_type' field"
                }));
            };
            let payload = input.payload.unwrap_or(serde_json::json!({}));
            let mut merged = payload;
            merged["event_type"] = serde_json::Value::String(event_type.clone());
            let message = match serde_json::to_string(&merged) {
                Ok(m) => m,
                Err(e) => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "serialization_failed",
                        "message": format!("{e}")
                    }));
                }
            };
            handle_event(
                pool.get_ref(),
                &channel,
                &message,
                &config.discord_webhook_url,
            )
            .await;
            HttpResponse::Ok().json(serde_json::json!({
                "status": "processed",
                "channel": channel
            }))
        }
        _ => {
            log::warn!("Lambda event with unknown path: {path}");
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "unknown_path",
                "message": format!("No handler for path: {path}")
            }))
        }
    }
}

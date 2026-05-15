use futures_util::StreamExt;
use redis::Client;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

const CHANNELS: &[&str] = &[
    "auth:events",
    "core:events",
    "judging:events",
    "mail:events",
];
const NOTABLE_EVENTS: &[&str] = &["team.created", "project.submitted", "event.rsvp"];

/// Subscribe to Redis pub/sub channels and handle events.
///
/// # Expected Behavior
///
/// If `redis_client` is `None`, logs a warning and returns immediately
/// without subscribing. If `redis_client` is `Some`, subscribes to
/// `auth:events`, `core:events`, `judging:events`, and `mail:events`
/// Redis channels. On each event, parses the JSON payload, extracts the
/// event type, finds matching webhooks (events array overlap), and
/// delivers the event payload to each matching webhook. For notable
/// events (`team.created`, `project.submitted`, `event.rsvp`), sends a
/// Discord notification via the configured webhook URL.
///
/// Runs in an infinite loop. Reconnects on connection failure with
/// exponential backoff starting at 5 seconds, doubling each attempt,
/// capped at 60 seconds. Backoff resets to 5s after a successful
/// connection + subscription. Messages that cannot be parsed as JSON
/// are logged at WARN level and skipped.
///
/// # Errors
///
/// Never returns under normal operation. Connection errors trigger
/// reconnection with backoff rather than propagating. Returns `Ok(())`
/// immediately if `redis_client` is None.
///
/// # Side Effects
///
/// - If `redis_client` is None: logs at WARN and returns (no I/O).
/// - Subscribes to Redis pub/sub channels (network connection).
/// - Reads from Redis channels (blocking listen).
/// - On matching events: reads from `notify.webhooks` table, writes to
///   `notify.webhook_deliveries` table, makes HTTP POST requests to
///   webhook URLs.
/// - On notable events: makes HTTP POST to Discord webhook URL.
/// - Logs at INFO on successful event handling and subscription,
///   WARN on parse errors, ERROR on connection failures.
pub async fn subscribe_to_events(
    pool: Arc<PgPool>,
    redis_client: Option<Arc<Client>>,
    discord_webhook_url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(redis_client) = redis_client else {
        log::warn!("Notify event subscriber skipped: REDIS_URL not configured");
        return Ok(());
    };
    let mut backoff_secs: u64 = 5;

    loop {
        match run_subscriber(
            pool.clone(),
            redis_client.clone(),
            discord_webhook_url.clone(),
        )
        .await
        {
            Ok(()) => {
                log::info!("Event subscriber ended, reconnecting in {backoff_secs}s...");
            }
            Err(e) => {
                log::error!("Event subscriber error: {e}, reconnecting in {backoff_secs}s...");
            }
        }
        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}

/// Run the Redis subscriber loop.
///
/// # Expected Behavior
///
/// Opens a new Redis async connection, converts it into a `PubSub` connection,
/// subscribes to `auth:events`, `core:events`, `judging:events`, and
/// `mail:events` channels, and processes incoming messages. Each message
/// is parsed as JSON, the event type is extracted (from `event_type` field,
/// falling back to `type`), and matching webhooks are found and delivered to.
/// Notable events (`team.created`, `project.submitted`, `event.rsvp`) also
/// trigger a Discord notification. Continues until the connection is closed
/// or an unrecoverable error occurs.
///
/// # Errors
///
/// Returns an error if the async connection cannot be established, if the
/// `PubSub` conversion fails, or if any channel subscription fails. Also
/// returns an error if the underlying connection is lost.
///
/// # Side Effects
///
/// - Opens a Redis async connection and converts to `PubSub` (network).
/// - Subscribes to four Redis channels (SUBSCRIBE commands).
/// - Processes and dispatches events (may trigger database reads, HTTP posts).
/// - Sends Discord notifications for notable events (HTTP POST).
/// - Logs at INFO for each received message and on successful subscription.
async fn run_subscriber(
    pool: Arc<PgPool>,
    redis_client: Arc<Client>,
    discord_webhook_url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = redis_client.get_async_connection().await?;
    let mut pubsub = conn.into_pubsub();

    for channel in CHANNELS {
        pubsub.subscribe(*channel).await?;
    }

    log::info!("Subscribed to Redis pub/sub channels: {CHANNELS:?}");

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        let channel = msg.get_channel_name().to_string();
        let payload: String = match msg.get_payload() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to decode message payload from {channel}: {e}");
                continue;
            }
        };

        handle_event(pool.as_ref(), &channel, &payload, &discord_webhook_url).await;
    }

    Ok(())
}

/// Handle an incoming Redis event message.
///
/// # Expected Behavior
///
/// Parses the message as JSON, extracts the `event_type` field (falling
/// back to `type` if `event_type` is absent), finds all active webhooks
/// whose events array contains the event type, and delivers the event
/// payload to each matching webhook. For notable events (`team.created`,
/// `project.submitted`, `event.rsvp`), sends a Discord notification via
/// the configured webhook URL. If the message cannot be parsed as JSON,
/// it is logged and skipped. Discord notification failures do not
/// prevent webhook delivery.
///
/// # Errors
///
/// Errors are logged but not propagated. Individual delivery failures
/// do not affect other deliveries.
///
/// # Side Effects
///
/// - Reads from `notify.webhooks` table.
/// - Writes to `notify.webhook_deliveries` table.
/// - Makes HTTP POST requests to matching webhook URLs.
/// - Makes HTTP POST request to Discord webhook URL for notable events.
/// - Logs at INFO on successful dispatch, WARN on parse errors or
///   Discord failures, ERROR on webhook delivery failures.
pub async fn handle_event(pool: &PgPool, channel: &str, message: &str, discord_webhook_url: &str) {
    let event: serde_json::Value = match serde_json::from_str(message) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse event from {channel}: {e}");
            return;
        }
    };

    let event_type = event
        .get("event_type")
        .and_then(|v| v.as_str())
        .or_else(|| event.get("type").and_then(|v| v.as_str()));

    let Some(event_type) = event_type else {
        log::warn!("Event from {channel} missing event_type/type field");
        return;
    };

    log::info!("Received event '{event_type}' from channel {channel}");

    if NOTABLE_EVENTS.contains(&event_type) && !discord_webhook_url.is_empty() {
        let content = format!("**{event_type}** event received on `{channel}`");
        if let Err(e) = crate::services::discord::send_discord_webhook(
            pool,
            discord_webhook_url,
            &content,
            None,
            None,
        )
        .await
        {
            log::warn!("Failed to send Discord notification for '{event_type}': {e}");
        }
    }

    match crate::services::webhook::find_matching_webhooks(pool, event_type).await {
        Ok(webhooks) => {
            for wh in webhooks {
                if let Err(e) =
                    crate::services::webhook::deliver_webhook(pool, wh.id, event_type, &event).await
                {
                    log::error!(
                        "Failed to deliver event '{}' to webhook {}: {}",
                        event_type,
                        wh.id,
                        e
                    );
                }
            }
        }
        Err(e) => {
            log::error!("Failed to find matching webhooks for '{event_type}': {e}");
        }
    }
}

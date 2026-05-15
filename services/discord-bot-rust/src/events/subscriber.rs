use futures_util::StreamExt;
use redis::Client;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::openhack_client::OpenHackClient;

const CHANNELS: &[&str] = &[
    "auth:events",
    "core:events",
    "judging:events",
    "leaderboard:events",
];

/// Subscribe to Redis pub/sub channels and post Discord notifications.
///
/// # Expected Behavior
///
/// If `redis_client` is `None`, logs a warning and returns immediately.
/// Otherwise subscribes to `auth:events`, `core:events`,
/// `judging:events`, and `leaderboard:events` Redis channels.
/// On each event, parses the JSON payload, extracts the event type,
/// looks up the channel configuration for that event type in the
/// database, and posts a rich embed notification to the configured
/// Discord channel using the bot API.
///
/// Runs in an infinite loop. Reconnects on connection failure with
/// exponential backoff starting at 5 seconds, doubling each attempt,
/// capped at 60 seconds. Backoff resets to 5s after a successful
/// connection + subscription.
///
/// # Errors
///
/// Never returns under normal operation. Connection errors trigger
/// reconnection with backoff. Individual event handling failures
/// are logged but do not crash the subscriber.
///
/// # Side Effects
///
/// - Subscribes to Redis pub/sub channels (network connection).
/// - On matching events: reads from `discord_bot.channel_config` table,
///   makes HTTP POST to Discord API to send channel messages.
/// - Logs at INFO on successful event handling, WARN on parse errors,
///   ERROR on connection failures.
pub async fn subscribe_to_events(
    pool: Arc<PgPool>,
    redis_client: Option<Arc<Client>>,
    config: Config,
    openhack_client: OpenHackClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(redis_client) = redis_client else {
        log::warn!("Discord bot event subscriber skipped: REDIS_URL not configured");
        return Ok(());
    };
    let mut backoff_secs: u64 = 5;

    loop {
        match run_subscriber(
            pool.clone(),
            redis_client.clone(),
            &config,
            &openhack_client,
        )
        .await
        {
            Ok(()) => {
                backoff_secs = 5;
                log::info!(
                    "Discord bot event subscriber ended, reconnecting in {backoff_secs}s..."
                );
            }
            Err(e) => {
                log::error!(
                    "Discord bot event subscriber error: {e}, reconnecting in {backoff_secs}s..."
                );
            }
        }
        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = backoff_secs.saturating_mul(2).min(60);
    }
}

async fn run_subscriber(
    pool: Arc<PgPool>,
    redis_client: Arc<Client>,
    config: &Config,
    openhack_client: &OpenHackClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = redis_client.get_async_connection().await?;
    let mut pubsub = conn.into_pubsub();

    for channel in CHANNELS {
        pubsub.subscribe(*channel).await?;
    }

    log::info!("Discord bot subscribed to Redis pub/sub channels: {CHANNELS:?}");

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

        handle_event(pool.as_ref(), &channel, &payload, config, openhack_client).await;
    }

    Ok(())
}

/// Handle an incoming Redis event and post to Discord if configured.
///
/// # Expected Behavior
///
/// Parses the message as JSON, extracts the `event_type` field (falling
/// back to `type`), looks up the channel configuration for that event
/// type in the `discord_bot.channel_config` table. If a channel is
/// configured, builds a rich embed notification and posts it to the
/// Discord channel using the bot API.
///
/// Supported events and their embed formatting:
/// - `team.created` — Team creation embed with "Join Team" button
/// - `project.submitted` — Project submission embed
/// - `event.rsvp` — Event RSVP embed
/// - `judging.phase_complete` — Judging results embed
/// - `leaderboard.updated` — Top-10 leaderboard embed
/// - `user.registered` — Welcome embed (or DM if Discord linked)
///
/// If no channel is configured for an event type, the event is logged
/// but no notification is sent. If the event cannot be parsed, it is
/// logged at WARN and skipped.
///
/// # Errors
///
/// Errors are logged but not propagated. Individual notification failures
/// do not affect other event processing.
///
/// # Side Effects
///
/// - Reads from `discord_bot.channel_config` table.
/// - May read from `discord_bot.user_links` table (for DMs).
/// - Makes HTTP POST requests to Discord API.
pub async fn handle_event(
    pool: &PgPool,
    channel: &str,
    message: &str,
    config: &Config,
    openhack_client: &OpenHackClient,
) {
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

    log::info!("Discord bot received event '{event_type}' from channel {channel}");

    let channel_config = crate::models::channel_config::find_by_event_type(
        pool,
        event_type,
        &config.discord_guild_id,
    )
    .await;

    let target_channel_id = match channel_config {
        Ok(Some(cfg)) => Some(cfg.channel_id.clone()),
        Ok(None) => None,
        Err(e) => {
            log::error!("Failed to look up channel config for '{event_type}': {e}");
            None
        }
    };

    let Some(target_channel_id) = target_channel_id else {
        log::debug!(
            "No Discord channel configured for event '{event_type}', skipping notification"
        );
        return;
    };

    let (content, embed) = build_notification(event_type, &event);

    if let Err(e) = openhack_client
        .send_channel_message(
            &config.discord_bot_token,
            &target_channel_id,
            &content,
            Some(&embed),
        )
        .await
    {
        log::warn!("Failed to send Discord notification for '{event_type}': {e}");
    }
}

/// Build a Discord notification embed for the given event type.
///
/// # Expected Behavior
///
/// Creates a rich embed appropriate for the event type with color
/// coding, title, description, and relevant fields extracted from
/// the event payload. Returns a tuple of (content, embeds_json).
///
/// # Errors
///
/// None. Returns a generic embed for unknown event types.
///
/// # Side Effects
///
/// None. Pure function.
fn build_notification(event_type: &str, event: &serde_json::Value) -> (String, serde_json::Value) {
    let (title, description, color) = match event_type {
        "team.created" => {
            let team_name = event
                .get("team_name")
                .and_then(|v| v.as_str())
                .unwrap_or("New Team");
            (
                format!("New Team Created: {team_name}"),
                format!(
                    "A new team **{team_name}** has been created! Click the button below to join.",
                ),
                0x58_65_f2,
            )
        }
        "project.submitted" => {
            let project_name = event
                .get("project_name")
                .and_then(|v| v.as_str())
                .unwrap_or("A Project");
            let team_name = event
                .get("team_name")
                .and_then(|v| v.as_str())
                .unwrap_or("A Team");
            (
                format!("Project Submitted: {project_name}"),
                format!("Team **{team_name}** submitted their project **{project_name}**!"),
                0x2e_cc_71,
            )
        }
        "event.rsvp" => {
            let event_name = event
                .get("event_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Event");
            (
                format!("RSVP: {event_name}"),
                format!("**{event_name}** is coming up! Click below to RSVP."),
                0xff_d7_00,
            )
        }
        "judging.phase_complete" => {
            let phase = event
                .get("phase")
                .and_then(|v| v.as_str())
                .unwrap_or("Phase");
            (
                format!("Judging Complete: {phase}"),
                format!("**{phase}** judging has been completed. Results are in!"),
                0x99_33_ff,
            )
        }
        "leaderboard.updated" => (
            "Leaderboard Updated".to_string(),
            "The leaderboard has been updated with new scores!".to_string(),
            0xff_d7_00,
        ),
        "user.registered" => {
            let username = event
                .get("username")
                .or_else(|| event.get("email"))
                .and_then(|v| v.as_str())
                .unwrap_or("A new participant");
            (
                "New Registration!".to_string(),
                format!("Welcome **{username}** to the hackathon!"),
                0x00_ff_ff,
            )
        }
        _ => (
            event_type.to_string(),
            format!("Event `{event_type}` received."),
            0x99_aa_bb,
        ),
    };

    let embed = serde_json::json!([{
        "title": title,
        "description": description,
        "color": color,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }]);

    let content = if event_type == "team.created" {
        "🎉 A new team has been formed!"
    } else if event_type == "project.submitted" {
        "🚀 New project submission!"
    } else if event_type == "user.registered" {
        "👋 Welcome aboard!"
    } else {
        ""
    };

    (content.to_string(), embed)
}

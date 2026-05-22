use chrono::{Duration, Utc};
use futures_util::StreamExt;
use redis::Client;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration as StdDuration;

const CHANNELS: &[&str] = &["auth:events", "core:events", "mail:events"];

/// Subscribe to Redis pub/sub channels and process mail-related events.
///
/// # Expected Behavior
///
/// If `redis_client` is `None`, logs a warning and returns immediately.
/// Otherwise, subscribes to Redis channels and processes events:
/// - `user.registered` → Enroll in welcome drip campaign + send welcome email
/// - `user.verified` → Send verification confirmation
/// - `team.formed` → Send team formation notification to all members
/// - `project.submitted` → Send submission confirmation
/// - Any event matching active drip campaigns → Auto-enroll user
///
/// Runs in an infinite loop with exponential backoff on connection loss.
///
/// # Errors
///
/// Never returns under normal operation. Connection errors trigger
/// reconnection with backoff rather than propagating.
///
/// # Side Effects
///
/// - Subscribes to Redis pub/sub channels.
/// - Reads from `auth.users`, `mail.drip_campaigns`, `mail.templates`.
/// - Inserts into `mail.sent_emails`, `mail.drip_enrollments`, `mail.drip_logs`.
/// - Publishes to Redis pub/sub for internal events.
pub async fn subscribe_to_events(
    pool: Arc<PgPool>,
    redis_client: Option<Arc<Client>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(redis_client) = redis_client else {
        log::warn!("Mail event subscriber skipped: REDIS_URL not configured");
        return Ok(());
    };
    let mut backoff_secs: u64 = 5;

    loop {
        match run_subscriber(pool.clone(), redis_client.clone()).await {
            Ok(()) => {}
            Err(e) => {
                log::error!("Mail event subscriber error: {e}, reconnecting in {backoff_secs}s...");
            }
        }
        tokio::time::sleep(StdDuration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}

async fn run_subscriber(
    pool: Arc<PgPool>,
    redis_client: Arc<Client>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = redis_client.get_async_connection().await?;
    let mut pubsub = conn.into_pubsub();

    for channel in CHANNELS {
        pubsub.subscribe(*channel).await?;
    }

    log::info!("Mail subscriber subscribed to channels: {CHANNELS:?}");

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

        handle_event(pool.as_ref(), &channel, &payload).await;
    }

    Ok(())
}

async fn handle_event(pool: &PgPool, _channel: &str, message: &str) {
    let event: serde_json::Value = match serde_json::from_str(message) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse event: {e}");
            return;
        }
    };

    let event_type = event
        .get("event_type")
        .and_then(|v| v.as_str())
        .or_else(|| event.get("type").and_then(|v| v.as_str()));

    let Some(event_type) = event_type else {
        log::warn!("Event missing event_type/type field");
        return;
    };

    log::info!("Mail received event: {event_type}");

    match event_type {
        "user.registered" => handle_user_registered(pool, &event).await,
        "user.verified" => handle_user_verified(pool, &event).await,
        "team.formed" => handle_team_formed(pool, &event).await,
        "project.submitted" => handle_project_submitted(pool, &event).await,
        _ => {
            // Check if event matches any drip campaign triggers
            handle_drip_enrollment(pool, event_type, &event).await;
        }
    }
}

async fn handle_user_registered(pool: &PgPool, event: &serde_json::Value) {
    let Some(user_id_str) = event.get("user_id").and_then(|v| v.as_str()) else {
        log::warn!("user.registered event missing user_id");
        return;
    };

    let Ok(user_id) = uuid::Uuid::parse_str(user_id_str) else {
        log::warn!("Invalid user_id in event: {user_id_str}");
        return;
    };

    let Some(email) = sqlx::query_scalar::<_, String>(
        "SELECT email FROM auth.users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    else {
        log::warn!("User {user_id} not found");
        return;
    };

    // Send welcome email
    let _ = sqlx::query(
        "INSERT INTO mail.sent_emails (recipient, subject, body_html, body_text, status)
         VALUES ($1, $2, $3, $4, 'pending')",
    )
    .bind(&email)
    .bind("Welcome to OpenHack!")
    .bind("<h1>Welcome!</h1><p>Get ready to build amazing things.</p>")
    .bind("Welcome! Get ready to build amazing things.")
    .execute(pool)
    .await;

    log::info!("Welcome email queued for user {user_id}");

    // Enroll in welcome drip campaign
    let _ = enroll_in_drip_campaign(pool, user_id, "user.registered", event).await;
}

async fn handle_user_verified(pool: &PgPool, event: &serde_json::Value) {
    let Some(user_id_str) = event.get("user_id").and_then(|v| v.as_str()) else {
        return;
    };

    let Ok(user_id) = uuid::Uuid::parse_str(user_id_str) else {
        return;
    };

    let Some(email) = sqlx::query_scalar::<_, String>(
        "SELECT email FROM auth.users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    else {
        return;
    };

    let _ = sqlx::query(
        "INSERT INTO mail.sent_emails (recipient, subject, body_html, body_text, status)
         VALUES ($1, $2, $3, $4, 'pending')",
    )
    .bind(&email)
    .bind("Email Verified!")
    .bind("<h1>Verified!</h1><p>Your email has been confirmed.</p>")
    .bind("Verified! Your email has been confirmed.")
    .execute(pool)
    .await;

    log::info!("Verification confirmation queued for user {user_id}");
}

async fn handle_team_formed(pool: &PgPool, event: &serde_json::Value) {
    let Some(team_id_str) = event.get("team_id").and_then(|v| v.as_str()) else {
        return;
    };

    let Ok(team_id) = uuid::Uuid::parse_str(team_id_str) else {
        return;
    };

    let team_name: String = event
        .get("team_name")
        .and_then(|v| v.as_str())
        .unwrap_or("your team")
        .to_string();

    // Get all team members
    let members: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT u.id, u.email FROM auth.users u
         JOIN core.team_members tm ON u.id = tm.user_id
         WHERE tm.team_id = $1",
    )
    .bind(team_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (member_id, email) in members {
        let _ = sqlx::query(
            "INSERT INTO mail.sent_emails (recipient, subject, body_html, body_text, status)
             VALUES ($1, $2, $3, $4, 'pending')",
        )
        .bind(&email)
        .bind(format!("Team '{team_name}' Formed!"))
        .bind(format!(
            "<h1>Team Formed!</h1><p>You're now part of '{team_name}'. Good luck!</p>"
        ))
        .bind(format!("Team Formed! You're now part of '{team_name}'. Good luck!"))
        .execute(pool)
        .await;

        log::info!("Team formation email queued for member {member_id}");
    }
}

async fn handle_project_submitted(pool: &PgPool, event: &serde_json::Value) {
    let Some(project_id_str) = event.get("project_id").and_then(|v| v.as_str()) else {
        return;
    };

    let Ok(project_id) = uuid::Uuid::parse_str(project_id_str) else {
        return;
    };

    let project_name: String = event
        .get("project_name")
        .and_then(|v| v.as_str())
        .unwrap_or("your project")
        .to_string();

    // Get project owner
    let owner: Option<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT u.id, u.email FROM auth.users u
         JOIN core.projects p ON u.id = p.owner_id
         WHERE p.id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if let Some((_, email)) = owner {
        let _ = sqlx::query(
            "INSERT INTO mail.sent_emails (recipient, subject, body_html, body_text, status)
             VALUES ($1, $2, $3, $4, 'pending')",
        )
        .bind(&email)
        .bind(format!("Project '{project_name}' Submitted!"))
        .bind(format!(
            "<h1>Submission Confirmed!</h1><p>Your project '{project_name}' has been submitted.</p>"
        ))
        .bind(format!(
            "Submission Confirmed! Your project '{project_name}' has been submitted."
        ))
        .execute(pool)
        .await;

        log::info!("Submission confirmation queued for project {project_id}");
    }
}

async fn handle_drip_enrollment(
    pool: &PgPool,
    event_type: &str,
    event: &serde_json::Value,
) {
    // Find active drip campaigns triggered by this event
    let campaigns: Vec<uuid::Uuid> = sqlx::query_scalar(
        "SELECT id FROM mail.drip_campaigns 
         WHERE trigger_event = $1 AND status = 'active'",
    )
    .bind(event_type)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let Some(user_id_str) = event.get("user_id").and_then(|v| v.as_str()) else {
        return;
    };

    let Ok(user_id) = uuid::Uuid::parse_str(user_id_str) else {
        return;
    };

    for campaign_id in campaigns {
        let trigger_data = serde_json::json!({
            "event_type": event_type,
            "event_data": event
        });

        let now = Utc::now();
        let first_delay: Option<i32> = sqlx::query_scalar(
            "SELECT MIN(delay_hours) FROM mail.drip_steps WHERE campaign_id = $1",
        )
        .bind(campaign_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let next_step_at = first_delay.map(|h| now + Duration::hours(h as i64));

        let _ = sqlx::query(
            "INSERT INTO mail.drip_enrollments (campaign_id, user_id, trigger_event, trigger_data, next_step_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (campaign_id, user_id) DO NOTHING",
        )
        .bind(campaign_id)
        .bind(user_id)
        .bind(event_type)
        .bind(trigger_data.to_string())
        .bind(next_step_at)
        .execute(pool)
        .await;

        log::info!("User {user_id} enrolled in drip campaign {campaign_id}");
    }
}

async fn enroll_in_drip_campaign(
    pool: &PgPool,
    user_id: uuid::Uuid,
    trigger_event: &str,
    event: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    let campaigns: Vec<uuid::Uuid> = sqlx::query_scalar(
        "SELECT id FROM mail.drip_campaigns 
         WHERE trigger_event = $1 AND status = 'active'",
    )
    .bind(trigger_event)
    .fetch_all(pool)
    .await?;

    let now = Utc::now();

    for campaign_id in campaigns {
        let first_delay: Option<i32> = sqlx::query_scalar(
            "SELECT MIN(delay_hours) FROM mail.drip_steps WHERE campaign_id = $1",
        )
        .bind(campaign_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        let next_step_at = first_delay.map(|h| now + Duration::hours(h as i64));
        let trigger_data = serde_json::json!({
            "event_type": trigger_event,
            "event_data": event
        });

        sqlx::query(
            "INSERT INTO mail.drip_enrollments (campaign_id, user_id, trigger_event, trigger_data, next_step_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (campaign_id, user_id) DO NOTHING",
        )
        .bind(campaign_id)
        .bind(user_id)
        .bind(trigger_event)
        .bind(trigger_data.to_string())
        .bind(next_step_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

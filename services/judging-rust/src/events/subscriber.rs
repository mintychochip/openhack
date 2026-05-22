use futures_util::StreamExt;
use redis::Client;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use uuid::Uuid;

const CHANNELS: &[&str] = &["core:events", "judging:events"];

/// Subscribe to Redis pub/sub channels and process judging-related events.
///
/// # Expected Behavior
///
/// If `redis_client` is `None`, logs a warning and returns immediately.
/// Otherwise, subscribes to Redis channels and processes events:
/// - `project.submitted` → Auto-assign judges based on rubric/priority
/// - `phase.opened` → Notify judges about open phase
/// - `team.advanced` → Create assignments for next phase
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
/// - Reads from `judging.assignments`, `judging.phases`, `auth.users`.
/// - Inserts into `judging.assignments`, `judging.scores`.
/// - Publishes to Redis pub/sub for internal events.
pub async fn subscribe_to_events(
    pool: Arc<PgPool>,
    redis_client: Option<Arc<Client>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(redis_client) = redis_client else {
        log::warn!("Judging event subscriber skipped: REDIS_URL not configured");
        return Ok(());
    };
    let mut backoff_secs: u64 = 5;

    loop {
        match run_subscriber(pool.clone(), redis_client.clone()).await {
            Ok(()) => {}
            Err(e) => {
                log::error!("Judging event subscriber error: {e}, reconnecting in {backoff_secs}s...");
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

    log::info!("Judging subscriber subscribed to channels: {CHANNELS:?}");

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

    log::info!("Judging received event: {event_type}");

    match event_type {
        "project.submitted" => handle_project_submitted(pool, &event).await,
        "phase.opened" => handle_phase_opened(pool, &event).await,
        "team.advanced" => handle_team_advanced(pool, &event).await,
        _ => log::debug!("Unhandled judging event: {event_type}"),
    }
}

async fn handle_project_submitted(pool: &PgPool, event: &serde_json::Value) {
    let Some(project_id_str) = event.get("project_id").and_then(|v| v.as_str()) else {
        log::warn!("project.submitted event missing project_id");
        return;
    };

    let Ok(project_id) = Uuid::parse_str(project_id_str) else {
        log::warn!("Invalid project_id: {project_id_str}");
        return;
    };

    // Get current open phase
    let phase_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM judging.phases WHERE status = 'open' AND type = 'preliminary' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(phase_id) = phase_id else {
        log::debug!("No open preliminary phase for auto-assignment");
        return;
    };

    // Get judges (users with judge/admin/organizer role)
    let judge_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM auth.users 
         WHERE roles && ARRAY['judge'::text, 'admin'::text, 'organizer'::text]",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if judge_ids.is_empty() {
        log::warn!("No judges available for auto-assignment");
        return;
    }

    // Assign 2-3 judges per project (round-robin style)
    let assignments_per_project = 2.min(judge_ids.len());
    let mut assigned_count = 0;

    for i in 0..assignments_per_project {
        let judge_id = judge_ids[i % judge_ids.len()];

        // Check if assignment already exists
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM judging.assignments 
                WHERE judge_id = $1 AND project_id = $2 AND phase_id = $3
            )",
        )
        .bind(judge_id)
        .bind(project_id)
        .bind(phase_id)
        .fetch_one(pool)
        .await
        .unwrap_or(true);

        if exists {
            continue;
        }

        let result = sqlx::query(
            "INSERT INTO judging.assignments (judge_id, project_id, phase_id, status, priority)
             VALUES ($1, $2, $3, 'pending', 50) RETURNING id",
        )
        .bind(judge_id)
        .bind(project_id)
        .bind(phase_id)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(Some(_)) => {
                assigned_count += 1;
                log::info!("Auto-assigned judge {judge_id} to project {project_id}");
            }
            Ok(None) => {
                log::warn!("Failed to create assignment for judge {judge_id}, project {project_id}");
            }
            Err(e) => {
                log::error!("Database error creating assignment: {e}");
            }
        }
    }

    log::info!("Auto-assigned {assigned_count} judges to project {project_id}");
}

async fn handle_phase_opened(pool: &PgPool, event: &serde_json::Value) {
    let Some(phase_id_str) = event.get("phase_id").and_then(|v| v.as_str()) else {
        return;
    };

    let Ok(phase_id) = Uuid::parse_str(phase_id_str) else {
        return;
    };

    let phase_name: &str = event
        .get("phase_name")
        .and_then(|v| v.as_str())
        .unwrap_or("new phase");

    // Get all judges
    let judges: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, email FROM auth.users 
         WHERE roles && ARRAY['judge'::text, 'admin'::text, 'organizer'::text]",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Notify each judge via mail service event
    for (judge_id, email) in judges {
        let notification = serde_json::json!({
            "event_type": "judging.phase_opened.notify",
            "judge_id": judge_id.to_string(),
            "phase_id": phase_id.to_string(),
            "phase_name": phase_name,
            "email": email
        });

        // Publish to mail events channel
        if let Ok(redis_client) = redis::Client::open(std::env::var("REDIS_URL").unwrap_or_default())
        {
            if let Ok(mut conn) = redis_client.get_async_connection().await {
                let _: Result<(), redis::RedisError> = redis::cmd("PUBLISH")
                    .arg("mail:events")
                    .arg(notification.to_string())
                    .query_async(&mut conn)
                    .await;
            }
        }

        log::info!("Queued phase opened notification for judge {judge_id}");
    }
}

async fn handle_team_advanced(pool: &PgPool, event: &serde_json::Value) {
    let Some(team_id_str) = event.get("team_id").and_then(|v| v.as_str()) else {
        return;
    };

    let Ok(team_id) = Uuid::parse_str(team_id_str) else {
        return;
    };

    let Some(to_phase_id_str) = event.get("to_phase_id").and_then(|v| v.as_str()) else {
        return;
    };

    let Ok(to_phase_id) = Uuid::parse_str(to_phase_id_str) else {
        return;
    };

    // Get project for this team
    let project_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM core.projects WHERE team_id = $1 LIMIT 1",
    )
    .bind(team_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(project_id) = project_id else {
        log::warn!("Team {team_id} has no project");
        return;
    };

    // Get judges for the new phase
    let judge_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM auth.users 
         WHERE roles && ARRAY['judge'::text, 'admin'::text, 'organizer'::text]
         LIMIT 3",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut assigned_count = 0;

    for judge_id in judge_ids {
        let result = sqlx::query(
            "INSERT INTO judging.assignments (judge_id, project_id, phase_id, status, priority)
             VALUES ($1, $2, $3, 'pending', 50)
             ON CONFLICT DO NOTHING RETURNING id",
        )
        .bind(judge_id)
        .bind(project_id)
        .bind(to_phase_id)
        .fetch_optional(pool)
        .await;

        if let Ok(Some(_)) = result {
            assigned_count += 1;
        }
    }

    log::info!("Created {assigned_count} assignments for advanced team {team_id} in phase {to_phase_id}");
}

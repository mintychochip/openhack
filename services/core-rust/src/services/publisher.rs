use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;

/// Publish an event to the `core:events` Redis channel.
///
/// # Expected Behavior
///
/// If a Redis connection is provided, serializes the event payload to JSON and
/// publishes it to the `core:events` Redis channel. If Redis is None, the event
/// is silently dropped — the service degrades gracefully without Redis.
/// The payload includes an `eventType` field and a `timestamp` field (ISO 8601 UTC).
/// Returns Ok(()) on success. Redis publish errors are logged at WARN level but not
/// propagated, since event publishing is non-critical and should not
/// block the main request flow.
///
/// # Errors
///
/// None. Failures are logged but not returned as Result to avoid disrupting
/// the main business logic.
///
/// # Side Effects
///
/// - Publishes a message to the `core:events` Redis channel (network write).
/// - Logs at WARN level on publish failure.
pub async fn publish_event(
    redis_conn: Option<&mut MultiplexedConnection>,
    event_type: &str,
    payload: Value,
) {
    let Some(conn) = redis_conn else {
        return;
    };

    let mut event = payload;
    event["eventType"] = Value::String(event_type.to_string());
    event["timestamp"] = Value::String(chrono::Utc::now().to_rfc3339());

    let message = match serde_json::to_string(&event) {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to serialize event: {e}");
            return;
        }
    };

    let result: Result<(), _> = conn.publish("core:events", &message).await;
    if let Err(e) = result {
        log::warn!("Failed to publish core event: {e}");
    }
}

/// Publish a team.created event.
///
/// # Expected Behavior
///
/// Publishes an event with the team ID and creator's user ID.
///
/// # Side Effects
///
/// - Publishes to `core:events` Redis channel.
pub async fn team_created(
    redis_conn: Option<&mut MultiplexedConnection>,
    team_id: &str,
    user_id: &str,
) {
    publish_event(
        redis_conn,
        "team.created",
        serde_json::json!({
            "teamId": team_id,
            "userId": user_id,
        }),
    )
    .await;
}

/// Publish a team.joined event.
///
/// # Expected Behavior
///
/// Publishes an event with the team ID and joining user's ID.
///
/// # Side Effects
///
/// - Publishes to `core:events` Redis channel.
pub async fn team_joined(
    redis_conn: Option<&mut MultiplexedConnection>,
    team_id: &str,
    user_id: &str,
) {
    publish_event(
        redis_conn,
        "team.joined",
        serde_json::json!({
            "teamId": team_id,
            "userId": user_id,
        }),
    )
    .await;
}

/// Publish a team.left event.
///
/// # Expected Behavior
///
/// Publishes an event with the team ID and leaving user's ID.
///
/// # Side Effects
///
/// - Publishes to `core:events` Redis channel.
pub async fn team_left(
    redis_conn: Option<&mut MultiplexedConnection>,
    team_id: &str,
    user_id: &str,
) {
    publish_event(
        redis_conn,
        "team.left",
        serde_json::json!({
            "teamId": team_id,
            "userId": user_id,
        }),
    )
    .await;
}

/// Publish a project.submitted event.
///
/// # Expected Behavior
///
/// Publishes an event with the project ID and team ID.
///
/// # Side Effects
///
/// - Publishes to `core:events` Redis channel.
pub async fn project_submitted(
    redis_conn: Option<&mut MultiplexedConnection>,
    project_id: &str,
    team_id: &str,
) {
    publish_event(
        redis_conn,
        "project.submitted",
        serde_json::json!({
            "projectId": project_id,
            "teamId": team_id,
        }),
    )
    .await;
}

/// Publish a project.updated event.
///
/// # Expected Behavior
///
/// Publishes an event with the project ID and team ID.
///
/// # Side Effects
///
/// - Publishes to `core:events` Redis channel.
pub async fn project_updated(
    redis_conn: Option<&mut MultiplexedConnection>,
    project_id: &str,
    team_id: &str,
) {
    publish_event(
        redis_conn,
        "project.updated",
        serde_json::json!({
            "projectId": project_id,
            "teamId": team_id,
        }),
    )
    .await;
}

/// Publish an event.rsvp event.
///
/// # Expected Behavior
///
/// Publishes an event with the event ID and user ID.
///
/// # Side Effects
///
/// - Publishes to `core:events` Redis channel.
pub async fn event_rsvp(
    redis_conn: Option<&mut MultiplexedConnection>,
    event_id: &str,
    user_id: &str,
) {
    publish_event(
        redis_conn,
        "event.rsvp",
        serde_json::json!({
            "eventId": event_id,
            "userId": user_id,
        }),
    )
    .await;
}

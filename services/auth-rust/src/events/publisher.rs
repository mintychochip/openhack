use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;

/// Publish an event to the `auth:events` Redis channel.
///
/// # Expected Behavior
///
/// If a Redis connection is provided, serializes the event payload to JSON and
/// publishes it to the `auth:events` Redis channel. The payload includes an `eventType`
/// field and a `timestamp` field (ISO 8601 UTC). If Redis is None, the event is
/// silently dropped — the service degrades gracefully without Redis.
/// Returns Ok(()) on success. Redis publish errors are logged at WARN level but not
/// propagated, since event publishing is non-critical and should not
/// block the main request flow.
///
/// # Errors
///
/// Returns `Err(())` if the Redis publish command fails.
/// Errors are logged but not returned as `Result` to avoid disrupting
/// the main business logic.
///
/// # Side Effects
///
/// - Publishes a message to the `auth:events` Redis channel (network write).
/// - Logs at WARN level on publish failure.
pub async fn publish_event(conn: Option<&MultiplexedConnection>, event_type: &str, payload: Value) {
    let Some(conn) = conn else {
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

    let mut c = conn.clone();
    let result: Result<(), _> = c.publish("auth:events", &message).await;
    if let Err(e) = result {
        log::warn!("Failed to publish auth event: {e}");
    }
}

/// Publish a user.registered event.
///
/// # Expected Behavior
///
/// Publishes an event with the user's ID, email, and name when a new
/// user registers.
///
/// # Errors
///
/// None. Failures are logged internally.
///
/// # Side Effects
///
/// - Publishes to `auth:events` Redis channel.
pub async fn user_registered(
    conn: Option<&MultiplexedConnection>,
    user_id: &str,
    email: &str,
    name: &str,
) {
    publish_event(
        conn,
        "user.registered",
        serde_json::json!({
            "userId": user_id,
            "email": email,
            "name": name,
        }),
    )
    .await;
}

/// Publish a `user.logged_in` event.
///
/// # Expected Behavior
///
/// Publishes an event with the user's ID and email when a user logs in.
///
/// # Errors
///
/// None. Failures are logged internally.
///
/// # Side Effects
///
/// - Publishes to `auth:events` Redis channel.
pub async fn user_logged_in(conn: Option<&MultiplexedConnection>, user_id: &str, email: &str) {
    publish_event(
        conn,
        "user.logged_in",
        serde_json::json!({
            "userId": user_id,
            "email": email,
        }),
    )
    .await;
}

/// Publish a `user.logged_out` event.
///
/// # Expected Behavior
///
/// Publishes an event with the user's ID when a user logs out.
///
/// # Errors
///
/// None. Failures are logged internally.
///
/// # Side Effects
///
/// - Publishes to `auth:events` Redis channel.
pub async fn user_logged_out(conn: Option<&MultiplexedConnection>, user_id: &str) {
    publish_event(
        conn,
        "user.logged_out",
        serde_json::json!({
            "userId": user_id,
        }),
    )
    .await;
}

/// Publish a `user.password_reset` event.
///
/// # Expected Behavior
///
/// Publishes an event with the user's ID when a password reset is completed.
///
/// # Errors
///
/// None. Failures are logged internally.
///
/// # Side Effects
///
/// - Publishes to `auth:events` Redis channel.
pub async fn user_password_reset(conn: Option<&MultiplexedConnection>, user_id: &str) {
    publish_event(
        conn,
        "user.password_reset",
        serde_json::json!({
            "userId": user_id,
        }),
    )
    .await;
}

/// Publish a `user.mfa_enabled` event.
///
/// # Expected Behavior
///
/// Publishes an event with the user's ID and MFA type when MFA is enabled.
///
/// # Errors
///
/// None. Failures are logged internally.
///
/// # Side Effects
///
/// - Publishes to `auth:events` Redis channel.
pub async fn user_mfa_enabled(conn: Option<&MultiplexedConnection>, user_id: &str, mfa_type: &str) {
    publish_event(
        conn,
        "user.mfa_enabled",
        serde_json::json!({
            "userId": user_id,
            "mfaType": mfa_type,
        }),
    )
    .await;
}

/// Publish a `user.mfa_disabled` event.
///
/// # Expected Behavior
///
/// Publishes an event with the user's ID when MFA is disabled.
///
/// # Errors
///
/// None. Failures are logged internally.
///
/// # Side Effects
///
/// - Publishes to `auth:events` Redis channel.
pub async fn user_mfa_disabled(conn: Option<&MultiplexedConnection>, user_id: &str) {
    publish_event(
        conn,
        "user.mfa_disabled",
        serde_json::json!({
            "userId": user_id,
        }),
    )
    .await;
}

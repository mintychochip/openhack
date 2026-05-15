use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::errors::NotifyError;
use crate::models::webhook::{
    CreateWebhookRequest, ListDeliveryLogsQuery, ListWebhooksQuery, UpdateWebhookRequest, Webhook,
    WebhookDelivery,
};

/// Create a new webhook configuration.
///
/// # Expected Behavior
///
/// Inserts a new row into `notify.webhooks` with a generated UUID.
/// The `active` field defaults to true if not provided. The `secret`
/// defaults to an empty string if not provided. Returns the newly
/// created webhook with all fields populated from the database.
///
/// # Errors
///
/// Returns `NotifyError::Database` if the insert fails.
///
/// # Side Effects
///
/// - Writes one row to `notify.webhooks` table.
/// - Logs at INFO level on success.
pub async fn create_webhook(
    pool: &PgPool,
    req: &CreateWebhookRequest,
) -> Result<Webhook, NotifyError> {
    let active = req.active.unwrap_or(true);
    let secret = req.secret.as_deref().unwrap_or("");
    let webhook = sqlx::query_as::<_, Webhook>(
        "INSERT INTO notify.webhooks (id, name, url, events, secret, active, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) \
         RETURNING id, name, url, events, secret, active, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(&req.name)
    .bind(&req.url)
    .bind(&req.events)
    .bind(secret)
    .bind(active)
    .fetch_one(pool)
    .await?;

    log::info!("Created webhook {} ({})", webhook.id, webhook.name);
    Ok(webhook)
}

/// List webhooks with pagination.
///
/// # Expected Behavior
///
/// Returns a paginated list of webhooks ordered by `created_at` descending.
/// Limit defaults to 50 (max 200), offset defaults to 0.
///
/// # Errors
///
/// Returns `NotifyError::Database` if the query fails.
///
/// # Side Effects
///
/// - Reads from `notify.webhooks` table.
pub async fn list_webhooks(
    pool: &PgPool,
    query: &ListWebhooksQuery,
) -> Result<Vec<Webhook>, NotifyError> {
    let limit = query.effective_limit();
    let offset = query.effective_offset();

    let webhooks = sqlx::query_as::<_, Webhook>(
        "SELECT id, name, url, events, secret, active, created_at, updated_at \
         FROM notify.webhooks \
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(webhooks)
}

/// Update a webhook configuration.
///
/// # Expected Behavior
///
/// Updates only the fields provided in the request (partial update).
/// Sets `updated_at` to `NOW()`. Returns the updated webhook. Returns
/// `NotifyError::WebhookNotFound` if no webhook with that ID exists.
///
/// # Errors
///
/// Returns `NotifyError::WebhookNotFound` if the ID does not exist.
/// Returns `NotifyError::Database` on database failure.
///
/// # Side Effects
///
/// - Writes updated fields to `notify.webhooks` table.
/// - Logs at INFO level on success.
pub async fn update_webhook(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateWebhookRequest,
) -> Result<Webhook, NotifyError> {
    let current = sqlx::query_as::<_, Webhook>(
        "SELECT id, name, url, events, secret, active, created_at, updated_at \
         FROM notify.webhooks WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| NotifyError::WebhookNotFound(id.to_string()))?;

    let name = req.name.as_deref().unwrap_or(&current.name);
    let url = req.url.as_deref().unwrap_or(&current.url);
    let events = req.events.as_ref().unwrap_or(&current.events);
    let secret = req.secret.as_deref().unwrap_or(&current.secret);
    let active = req.active.unwrap_or(current.active);

    let webhook = sqlx::query_as::<_, Webhook>(
        "UPDATE notify.webhooks SET name = $1, url = $2, events = $3, secret = $4, active = $5, updated_at = NOW() \
         WHERE id = $6 \
         RETURNING id, name, url, events, secret, active, created_at, updated_at",
    )
    .bind(name)
    .bind(url)
    .bind(events)
    .bind(secret)
    .bind(active)
    .bind(id)
    .fetch_one(pool)
    .await?;

    log::info!("Updated webhook {id}");
    Ok(webhook)
}

/// Delete a webhook by ID.
///
/// # Expected Behavior
///
/// Deletes the webhook and all associated delivery logs (via CASCADE).
/// Returns `NotifyError::WebhookNotFound` if no webhook with that ID exists.
///
/// # Errors
///
/// Returns `NotifyError::WebhookNotFound` if the ID does not exist.
/// Returns `NotifyError::Database` on database failure.
///
/// # Side Effects
///
/// - Deletes a row from `notify.webhooks` table (cascades to deliveries).
/// - Logs at INFO level on success.
pub async fn delete_webhook(pool: &PgPool, id: Uuid) -> Result<(), NotifyError> {
    let result = sqlx::query("DELETE FROM notify.webhooks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(NotifyError::WebhookNotFound(id.to_string()));
    }

    log::info!("Deleted webhook {id}");
    Ok(())
}

/// Deliver a webhook by posting the payload to the webhook URL with HMAC signature.
///
/// # Expected Behavior
///
/// Sends an HTTP POST to the webhook URL with:
/// - `Content-Type: application/json`
/// - `X-Webhook-Signature: sha256=<HMAC-SHA256 hex>` using the webhook secret
/// - `X-Webhook-Event: <event_type>`
/// - 15 second timeout
///
/// On success (2xx): records delivery with status "delivered".
/// On 5xx: records delivery with status "`pending_retry`", increments attempts,
///   sets `next_retry_at` using exponential backoff (1s, 5s, 25s, 125s), max 5 attempts.
///   After 5 failed attempts, marks as "failed".
/// On 4xx: records delivery with status "failed" immediately.
///
/// # Errors
///
/// Returns `NotifyError::WebhookNotFound` if the webhook ID doesn't exist.
/// Returns `NotifyError::WebhookDeliveryFailed` on delivery failure (non-2xx).
///
/// # Side Effects
///
/// - Makes an HTTP POST request to the webhook URL (network I/O).
/// - Writes a row to `notify.webhook_deliveries` table.
/// - Logs at INFO on success, WARN on retryable failure, ERROR on permanent failure.
pub async fn deliver_webhook(
    pool: &PgPool,
    webhook_id: Uuid,
    event_type: &str,
    payload: &serde_json::Value,
) -> Result<WebhookDelivery, NotifyError> {
    let webhook = sqlx::query_as::<_, Webhook>(
        "SELECT id, name, url, events, secret, active, created_at, updated_at \
         FROM notify.webhooks WHERE id = $1",
    )
    .bind(webhook_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| NotifyError::WebhookNotFound(webhook_id.to_string()))?;

    if !webhook.active {
        let delivery = sqlx::query_as::<_, WebhookDelivery>(
            "INSERT INTO notify.webhook_deliveries (id, webhook_id, event_type, payload, status, attempts, created_at) \
             VALUES ($1, $2, $3, $4, 'skipped', 0, NOW()) \
             RETURNING id, webhook_id, event_type, payload, status, response_code, response_body, attempts, next_retry_at, created_at",
        )
        .bind(Uuid::new_v4())
        .bind(webhook_id)
        .bind(event_type)
        .bind(payload)
        .fetch_one(pool)
        .await?;

        return Ok(delivery);
    }

    let payload_bytes = serde_json::to_vec(payload).unwrap_or_default();
    let signature = compute_hmac(&webhook.secret, &payload_bytes);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let response = client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", format!("sha256={signature}"))
        .header("X-Webhook-Event", event_type)
        .body(payload_bytes.clone())
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status_u16 = resp.status().as_u16();
            let status_code = i32::from(status_u16);
            let body = resp.text().await.unwrap_or_default();

            if (200..300).contains(&status_u16) {
                let delivery = sqlx::query_as::<_, WebhookDelivery>(
                    "INSERT INTO notify.webhook_deliveries \
                     (id, webhook_id, event_type, payload, status, response_code, response_body, attempts, created_at) \
                     VALUES ($1, $2, $3, $4, 'delivered', $5, $6, 1, NOW()) \
                     RETURNING id, webhook_id, event_type, payload, status, response_code, response_body, attempts, next_retry_at, created_at",
                )
                .bind(Uuid::new_v4())
                .bind(webhook_id)
                .bind(event_type)
                .bind(payload)
                .bind(status_code)
                .bind(&body)
                .fetch_one(pool)
                .await?;

                log::info!("Webhook {webhook_id} delivered successfully ({status_code})");
                Ok(delivery)
            } else if (500..600).contains(&status_u16) {
                let delivery = record_retryable_failure(
                    pool,
                    webhook_id,
                    event_type,
                    payload,
                    status_code,
                    &body,
                    1,
                )
                .await?;
                log::warn!("Webhook {webhook_id} returned {status_code}, will retry");
                Ok(delivery)
            } else {
                let delivery = sqlx::query_as::<_, WebhookDelivery>(
                    "INSERT INTO notify.webhook_deliveries \
                     (id, webhook_id, event_type, payload, status, response_code, response_body, attempts, created_at) \
                     VALUES ($1, $2, $3, $4, 'failed', $5, $6, 1, NOW()) \
                     RETURNING id, webhook_id, event_type, payload, status, response_code, response_body, attempts, next_retry_at, created_at",
                )
                .bind(Uuid::new_v4())
                .bind(webhook_id)
                .bind(event_type)
                .bind(payload)
                .bind(status_code)
                .bind(&body)
                .fetch_one(pool)
                .await?;

                log::error!("Webhook {webhook_id} failed with client error {status_code}");
                Ok(delivery)
            }
        }
        Err(e) => {
            let delivery = record_retryable_failure(
                pool,
                webhook_id,
                event_type,
                payload,
                0,
                &e.to_string(),
                1,
            )
            .await?;
            log::warn!("Webhook {webhook_id} request failed: {e}, will retry");
            Ok(delivery)
        }
    }
}

/// Record a retryable webhook delivery failure with exponential backoff.
///
/// # Expected Behavior
///
/// Creates a delivery record with status "`pending_retry`" if attempts < 5,
/// or "failed" if attempts >= 5. Sets `next_retry_at` to `NOW()` + backoff delay
/// where delay = 5^(attempts-1) seconds (1s, 5s, 25s, 125s, 625s).
///
/// # Errors
///
/// Returns `NotifyError::Database` if the insert fails.
///
/// # Side Effects
///
/// - Writes a row to `notify.webhook_deliveries` table.
#[allow(clippy::cast_sign_loss)]
async fn record_retryable_failure(
    pool: &PgPool,
    webhook_id: Uuid,
    event_type: &str,
    payload: &serde_json::Value,
    response_code: i32,
    response_body: &str,
    attempts: i32,
) -> Result<WebhookDelivery, NotifyError> {
    let max_attempts = 5;
    let status = if attempts >= max_attempts {
        "failed"
    } else {
        "pending_retry"
    };

    let backoff_secs = 5_i64.pow((attempts - 1) as u32);
    let next_retry_at = if attempts < max_attempts {
        Some(chrono::Utc::now().naive_utc() + chrono::Duration::seconds(backoff_secs))
    } else {
        None
    };

    let delivery = sqlx::query_as::<_, WebhookDelivery>(
        "INSERT INTO notify.webhook_deliveries \
         (id, webhook_id, event_type, payload, status, response_code, response_body, attempts, next_retry_at, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW()) \
         RETURNING id, webhook_id, event_type, payload, status, response_code, response_body, attempts, next_retry_at, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(webhook_id)
    .bind(event_type)
    .bind(payload)
    .bind(status)
    .bind(response_code)
    .bind(response_body)
    .bind(attempts)
    .bind(next_retry_at)
    .fetch_one(pool)
    .await?;

    Ok(delivery)
}

/// Retry pending webhook deliveries that are past their `next_retry_at`.
///
/// # Expected Behavior
///
/// Queries for deliveries with status "`pending_retry`" and `next_retry_at` <= `NOW()`,
/// increments the attempt count, and re-delivers. If the attempt count reaches 5,
/// marks as "failed". Uses exponential backoff for subsequent retries.
///
/// # Errors
///
/// Errors are logged but not propagated. Individual delivery failures do not
/// affect other deliveries.
///
/// # Side Effects
///
/// - Reads from `notify.webhook_deliveries` table.
/// - Makes HTTP POST requests to webhook URLs.
/// - Updates delivery records in the database.
/// - Logs at INFO on successful retry, WARN on retryable failure.
#[allow(clippy::too_many_lines, clippy::cast_sign_loss)]
pub async fn retry_pending_deliveries(pool: &PgPool) {
    let pending = sqlx::query_as::<_, WebhookDelivery>(
        "SELECT id, webhook_id, event_type, payload, status, response_code, response_body, attempts, next_retry_at, created_at \
         FROM notify.webhook_deliveries \
         WHERE status = 'pending_retry' AND next_retry_at <= NOW() \
         ORDER BY created_at ASC LIMIT 50",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for delivery in pending {
        let attempts = delivery.attempts.unwrap_or(0) + 1;
        let max_attempts = 5;

        if attempts > max_attempts {
            let _ =
                sqlx::query("UPDATE notify.webhook_deliveries SET status = 'failed' WHERE id = $1")
                    .bind(delivery.id)
                    .execute(pool)
                    .await;
            continue;
        }

        let Ok(Some(webhook)) = sqlx::query_as::<_, Webhook>(
            "SELECT id, name, url, events, secret, active, created_at, updated_at \
             FROM notify.webhooks WHERE id = $1",
        )
        .bind(delivery.webhook_id)
        .fetch_optional(pool)
        .await
        else {
            continue;
        };

        if !webhook.active {
            continue;
        }

        let payload_bytes = serde_json::to_vec(&delivery.payload).unwrap_or_default();
        let signature = compute_hmac(&webhook.secret, &payload_bytes);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let result = client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", format!("sha256={signature}"))
            .header("X-Webhook-Event", &delivery.event_type)
            .body(payload_bytes)
            .send()
            .await;

        match result {
            Ok(resp) => {
                let code_u16 = resp.status().as_u16();
                let code = i32::from(code_u16);
                let body = resp.text().await.unwrap_or_default();

                if (200..300).contains(&code_u16) {
                    let _ = sqlx::query(
                        "UPDATE notify.webhook_deliveries SET status = 'delivered', response_code = $1, response_body = $2, attempts = $3 WHERE id = $4",
                    )
                    .bind(code)
                    .bind(&body)
                    .bind(attempts)
                    .bind(delivery.id)
                    .execute(pool)
                    .await;
                    log::info!(
                        "Webhook delivery {} succeeded on attempt {}",
                        delivery.id,
                        attempts
                    );
                } else if (500..600).contains(&code_u16) {
                    let backoff_secs = 5_i64.pow((attempts - 1) as u32);
                    let next_retry =
                        chrono::Utc::now().naive_utc() + chrono::Duration::seconds(backoff_secs);
                    let new_status = if attempts >= max_attempts {
                        "failed"
                    } else {
                        "pending_retry"
                    };
                    let _ = sqlx::query(
                        "UPDATE notify.webhook_deliveries SET status = $1, response_code = $2, response_body = $3, attempts = $4, next_retry_at = $5 WHERE id = $6",
                    )
                    .bind(new_status)
                    .bind(code)
                    .bind(&body)
                    .bind(attempts)
                    .bind(next_retry)
                    .bind(delivery.id)
                    .execute(pool)
                    .await;
                    log::warn!(
                        "Webhook delivery {} retry {} returned {}",
                        delivery.id,
                        attempts,
                        code
                    );
                } else {
                    let _ = sqlx::query(
                        "UPDATE notify.webhook_deliveries SET status = 'failed', response_code = $1, response_body = $2, attempts = $3 WHERE id = $4",
                    )
                    .bind(code)
                    .bind(&body)
                    .bind(attempts)
                    .bind(delivery.id)
                    .execute(pool)
                    .await;
                    log::error!(
                        "Webhook delivery {} failed with client error {}",
                        delivery.id,
                        code
                    );
                }
            }
            Err(e) => {
                let backoff_secs = 5_i64.pow((attempts - 1) as u32);
                let next_retry =
                    chrono::Utc::now().naive_utc() + chrono::Duration::seconds(backoff_secs);
                let new_status = if attempts >= max_attempts {
                    "failed"
                } else {
                    "pending_retry"
                };
                let _ = sqlx::query(
                    "UPDATE notify.webhook_deliveries SET status = $1, response_body = $2, attempts = $3, next_retry_at = $4 WHERE id = $5",
                )
                .bind(new_status)
                .bind(e.to_string())
                .bind(attempts)
                .bind(next_retry)
                .bind(delivery.id)
                .execute(pool)
                .await;
                log::warn!(
                    "Webhook delivery {} retry {} failed: {}",
                    delivery.id,
                    attempts,
                    e
                );
            }
        }
    }
}

/// Get delivery logs for a specific webhook.
///
/// # Expected Behavior
///
/// Returns paginated delivery log entries for the given `webhook_id`,
/// ordered by `created_at` descending. Returns `NotifyError::WebhookNotFound`
/// if the webhook does not exist.
///
/// # Errors
///
/// Returns `NotifyError::WebhookNotFound` if the webhook ID doesn't exist.
/// Returns `NotifyError::Database` on query failure.
///
/// # Side Effects
///
/// - Reads from `notify.webhook_deliveries` table.
pub async fn get_delivery_logs(
    pool: &PgPool,
    webhook_id: Uuid,
    query: &ListDeliveryLogsQuery,
) -> Result<Vec<WebhookDelivery>, NotifyError> {
    let exists = sqlx::query_as::<_, Webhook>(
        "SELECT id, name, url, events, secret, active, created_at, updated_at \
         FROM notify.webhooks WHERE id = $1",
    )
    .bind(webhook_id)
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        return Err(NotifyError::WebhookNotFound(webhook_id.to_string()));
    }

    let limit = query.effective_limit();
    let offset = query.effective_offset();

    let logs = sqlx::query_as::<_, WebhookDelivery>(
        "SELECT id, webhook_id, event_type, payload, status, response_code, response_body, attempts, next_retry_at, created_at \
         FROM notify.webhook_deliveries WHERE webhook_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(webhook_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

/// Find all active webhooks whose events array overlaps with the given event type.
///
/// # Expected Behavior
///
/// Queries `notify.webhooks` where `active = true` and the `events` array
/// contains the given `event_type` string. Returns all matching webhooks.
///
/// # Errors
///
/// Returns `NotifyError::Database` on query failure.
///
/// # Side Effects
///
/// - Reads from `notify.webhooks` table.
pub async fn find_matching_webhooks(
    pool: &PgPool,
    event_type: &str,
) -> Result<Vec<Webhook>, NotifyError> {
    let webhooks = sqlx::query_as::<_, Webhook>(
        "SELECT id, name, url, events, secret, active, created_at, updated_at \
         FROM notify.webhooks WHERE active = true AND $1 = ANY(events)",
    )
    .bind(event_type)
    .fetch_all(pool)
    .await?;

    Ok(webhooks)
}

/// Compute HMAC-SHA256 signature for webhook payload.
///
/// # Expected Behavior
///
/// Computes HMAC-SHA256 of the payload using the provided secret key.
/// Returns the hex-encoded digest string. If the secret is empty, returns
/// an empty string (no signature). If the HMAC key initialization fails,
/// returns an empty string.
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None. Pure function.
pub(crate) fn compute_hmac(secret: &str, payload: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    if secret.is_empty() {
        return String::new();
    }

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return String::new();
    };
    mac.update(payload);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

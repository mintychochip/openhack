use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::middleware::auth::AuthUser;
use crate::models::webhook::{
    CreateWebhookRequest, ListDeliveryLogsQuery, ListWebhooksQuery, TestWebhookRequest,
    UpdateWebhookRequest,
};
use crate::services::webhook;

/// Create a new webhook.
///
/// # Expected Behavior
///
/// Accepts a JSON body with name, url, events, optional secret and active flag.
/// Creates the webhook in the database and returns 201 with the created webhook.
/// Requires authentication.
///
/// # Errors
///
/// Returns 400 if the request body is malformed. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Writes to `notify.webhooks` table.
pub async fn create_webhook(
    pool: web::Data<PgPool>,
    req: web::Json<CreateWebhookRequest>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    match webhook::create_webhook(pool.get_ref(), &req).await {
        Ok(w) => HttpResponse::Created().json(w),
        Err(e) => e.to_http_response(),
    }
}

/// List webhooks with pagination.
///
/// # Expected Behavior
///
/// Returns a paginated list of webhooks ordered by `created_at` descending.
/// Supports `limit` and `offset` query parameters. Requires authentication.
///
/// # Errors
///
/// Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `notify.webhooks` table.
pub async fn list_webhooks(
    pool: web::Data<PgPool>,
    query: web::Query<ListWebhooksQuery>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    match webhook::list_webhooks(pool.get_ref(), &query).await {
        Ok(webhooks) => HttpResponse::Ok().json(webhooks),
        Err(e) => e.to_http_response(),
    }
}

/// Update a webhook.
///
/// # Expected Behavior
///
/// Accepts a JSON body with partial update fields (name, url, events, secret, active).
/// Updates the webhook in the database and returns the updated webhook.
/// Requires authentication.
///
/// # Errors
///
/// Returns 404 if the webhook does not exist. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Writes updated fields to `notify.webhooks` table.
pub async fn update_webhook(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    req: web::Json<UpdateWebhookRequest>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    let id = id.into_inner();
    match webhook::update_webhook(pool.get_ref(), id, &req).await {
        Ok(w) => HttpResponse::Ok().json(w),
        Err(e) => e.to_http_response(),
    }
}

/// Delete a webhook.
///
/// # Expected Behavior
///
/// Deletes the webhook with the given UUID. Returns 404 if not found.
/// Requires authentication.
///
/// # Errors
///
/// Returns 404 if the webhook does not exist. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Deletes from `notify.webhooks` table.
pub async fn delete_webhook(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    let id = id.into_inner();
    match webhook::delete_webhook(pool.get_ref(), id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Err(e) => e.to_http_response(),
    }
}

/// Test a webhook by sending a test delivery.
///
/// # Expected Behavior
///
/// Sends a test payload to the webhook URL. Accepts optional `event_type`
/// (defaults to "test") and optional payload (defaults to a test object).
/// Records the delivery in `webhook_deliveries`. Requires authentication.
///
/// # Errors
///
/// Returns 404 if the webhook does not exist. Returns 502 on delivery failure.
///
/// # Side Effects
///
/// - Makes HTTP POST to webhook URL.
/// - Writes to `notify.webhook_deliveries` table.
pub async fn test_webhook(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    req: web::Json<TestWebhookRequest>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    let id = id.into_inner();
    let event_type = req.event_type.as_deref().unwrap_or("test");
    let payload = req.payload.clone().unwrap_or(serde_json::json!({
        "test": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    match webhook::deliver_webhook(pool.get_ref(), id, event_type, &payload).await {
        Ok(delivery) => HttpResponse::Ok().json(delivery),
        Err(e) => e.to_http_response(),
    }
}

/// Get delivery logs for a webhook.
///
/// # Expected Behavior
///
/// Returns paginated delivery log entries for the given webhook ID.
/// Supports `limit` and `offset` query parameters. Requires authentication.
///
/// # Errors
///
/// Returns 404 if the webhook does not exist. Returns 500 on database failure.
///
/// # Side Effects
///
/// - Reads from `notify.webhook_deliveries` table.
pub async fn get_webhook_logs(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    query: web::Query<ListDeliveryLogsQuery>,
    _user: web::ReqData<AuthUser>,
) -> HttpResponse {
    let id = id.into_inner();
    match webhook::get_delivery_logs(pool.get_ref(), id, &query).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => e.to_http_response(),
    }
}

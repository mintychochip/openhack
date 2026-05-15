use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub response_code: Option<i32>,
    pub response_body: Option<String>,
    pub attempts: Option<i32>,
    pub next_retry_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhookRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub secret: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWebhookRequest {
    pub event_type: Option<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListWebhooksQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl ListWebhooksQuery {
    /// Return the effective limit, clamped between 1 and 200, defaulting to 50.
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 200)
    }

    /// Return the effective offset, defaulting to 0 if None.
    pub fn effective_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListDeliveryLogsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl ListDeliveryLogsQuery {
    /// Return the effective limit, clamped between 1 and 200, defaulting to 50.
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 200)
    }

    /// Return the effective offset, defaulting to 0 if None.
    pub fn effective_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

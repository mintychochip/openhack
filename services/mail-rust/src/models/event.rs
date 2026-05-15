use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailEvent {
    pub id: Uuid,
    pub message_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub occurred_at: Option<NaiveDateTime>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct EventQuery {
    pub message_id: Option<String>,
}

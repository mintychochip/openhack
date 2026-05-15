use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub action_url: Option<String>,
    pub is_read: bool,
    pub read_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub action_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListNotificationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl ListNotificationsQuery {
    /// Return the effective limit, clamped between 1 and 200, defaulting to 50.
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 200)
    }

    /// Return the effective offset, defaulting to 0 if None.
    pub fn effective_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCount {
    pub count: i64,
}

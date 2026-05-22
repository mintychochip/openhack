use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BoothVisitor {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub user_id: Option<Uuid>,
    pub visited_at: DateTime<Utc>,
    pub duration_seconds: Option<i32>,
    pub source: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BoothMessage {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub user_id: Uuid,
    pub message: String,
    pub is_sponsor_reply: bool,
    pub parent_message_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BoothPoll {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub question: String,
    pub description: Option<String>,
    pub options: serde_json::Value,
    pub allow_multiple: bool,
    pub is_active: bool,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BoothPollResponse {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub user_id: Uuid,
    pub option_indices: Vec<i32>,
    pub responded_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BoothResource {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub url: String,
    pub download_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoothPollOption {
    pub text: String,
    pub color: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoothVisitorCreate {
    pub user_id: Option<Uuid>,
    pub source: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoothMessageCreate {
    pub user_id: Uuid,
    pub message: String,
    pub parent_message_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoothPollCreate {
    pub question: String,
    pub description: Option<String>,
    pub options: Vec<BoothPollOption>,
    pub allow_multiple: Option<bool>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoothPollVote {
    pub user_id: Uuid,
    pub option_indices: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoothResourceCreate {
    pub title: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoothVisitorResponse {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub user_id: Option<Uuid>,
    pub visited_at: DateTime<Utc>,
    pub duration_seconds: Option<i32>,
    pub source: Option<String>,
}

impl From<BoothVisitor> for BoothVisitorResponse {
    fn from(v: BoothVisitor) -> Self {
        Self {
            id: v.id,
            booth_id: v.booth_id,
            user_id: v.user_id,
            visited_at: v.visited_at,
            duration_seconds: v.duration_seconds,
            source: v.source,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BoothMessageResponse {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub user_id: Uuid,
    pub message: String,
    pub is_sponsor_reply: bool,
    pub parent_message_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

impl From<BoothMessage> for BoothMessageResponse {
    fn from(m: BoothMessage) -> Self {
        Self {
            id: m.id,
            booth_id: m.booth_id,
            user_id: m.user_id,
            message: m.message,
            is_sponsor_reply: m.is_sponsor_reply,
            parent_message_id: m.parent_message_id,
            created_at: m.created_at,
            is_read: m.is_read,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BoothPollResponseData {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub question: String,
    pub description: Option<String>,
    pub options: serde_json::Value,
    pub allow_multiple: bool,
    pub is_active: bool,
    pub ends_at: Option<DateTime<Utc>>,
    pub total_responses: i64,
    pub option_counts: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoothPollListResponse {
    pub polls: Vec<BoothPollResponseData>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoothResourceResponse {
    pub id: Uuid,
    pub booth_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub url: String,
    pub download_count: i32,
}

impl From<BoothResource> for BoothResourceResponse {
    fn from(r: BoothResource) -> Self {
        Self {
            id: r.id,
            booth_id: r.booth_id,
            title: r.title,
            description: r.description,
            resource_type: r.resource_type,
            url: r.url,
            download_count: r.download_count,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BoothAnalyticsResponse {
    pub total_visitors: i64,
    pub unique_visitors: i64,
    pub avg_duration_seconds: Option<f64>,
    pub total_messages: i64,
    pub unread_messages: i64,
    pub total_poll_responses: i64,
    pub total_resource_downloads: i64,
    pub visitors_by_source: serde_json::Value,
}

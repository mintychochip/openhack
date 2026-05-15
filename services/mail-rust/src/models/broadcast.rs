use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailBroadcast {
    pub id: Uuid,
    pub name: String,
    pub template_id: Option<Uuid>,
    pub subject: String,
    pub body_html: String,
    pub body_text: Option<String>,
    pub audience_filter: serde_json::Value,
    pub status: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub total_recipients: Option<i32>,
    pub sent_count: Option<i32>,
    pub delivered_count: Option<i32>,
    pub bounce_count: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastCreate {
    pub name: String,
    pub template_id: Option<Uuid>,
    pub subject: String,
    pub body_html: String,
    pub body_text: Option<String>,
    #[serde(default)]
    pub audience_filter: serde_json::Value,
    #[allow(dead_code)]
    #[serde(default = "default_priority")]
    pub priority: String,
    #[allow(dead_code)]
    pub scheduled_at: Option<String>,
}

fn default_priority() -> String {
    "normal".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastSchedule {
    pub scheduled_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BroadcastResponse {
    pub id: Uuid,
    pub name: String,
    pub template_id: Option<Uuid>,
    pub subject: String,
    pub audience_filter: serde_json::Value,
    pub status: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub total_recipients: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub bounce_count: i32,
    pub created_at: Option<NaiveDateTime>,
}

impl From<MailBroadcast> for BroadcastResponse {
    fn from(b: MailBroadcast) -> Self {
        Self {
            id: b.id,
            name: b.name,
            template_id: b.template_id,
            subject: b.subject,
            audience_filter: b.audience_filter,
            status: b.status,
            scheduled_at: b.scheduled_at,
            started_at: b.started_at,
            completed_at: b.completed_at,
            total_recipients: b.total_recipients.unwrap_or(0),
            sent_count: b.sent_count.unwrap_or(0),
            delivered_count: b.delivered_count.unwrap_or(0),
            bounce_count: b.bounce_count.unwrap_or(0),
            created_at: b.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BroadcastListResponse {
    pub broadcasts: Vec<BroadcastResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastQuery {
    #[serde(default = "default_page")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status_filter: Option<String>,
}

fn default_page() -> i64 {
    100
}

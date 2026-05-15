use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailMessage {
    pub id: Uuid,
    pub message_id: String,
    pub to_addrs: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub bcc_addrs: Vec<String>,
    pub subject: String,
    pub body_html: String,
    pub body_text: Option<String>,
    pub status: String,
    pub priority: String,
    pub template_id: Option<Uuid>,
    pub scheduled_at: Option<NaiveDateTime>,
    pub sent_at: Option<NaiveDateTime>,
    pub delivered_at: Option<NaiveDateTime>,
    pub bounce_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendRequest {
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    #[serde(default)]
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub template_id: Option<Uuid>,
    #[allow(dead_code)]
    pub template_data: Option<serde_json::Value>,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[allow(dead_code)]
    pub scheduled_at: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_priority() -> String {
    "normal".into()
}

fn deserialize_non_empty_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Vec<String> = Vec::deserialize(deserializer)?;
    if v.is_empty() {
        return Err(serde::de::Error::custom(
            "'to' must have at least one recipient",
        ));
    }
    Ok(v)
}

#[derive(Debug, Clone, Serialize)]
pub struct SendResponse {
    pub message_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkSendRequest {
    #[serde(deserialize_with = "deserialize_bulk_emails")]
    pub emails: Vec<SendRequest>,
    #[serde(default = "default_priority")]
    pub batch_priority: String,
}

fn deserialize_bulk_emails<'de, D>(deserializer: D) -> Result<Vec<SendRequest>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Vec<SendRequest> = Vec::deserialize(deserializer)?;
    if v.is_empty() {
        return Err(serde::de::Error::custom(
            "'emails' must have at least one entry",
        ));
    }
    if v.len() > 1000 {
        return Err(serde::de::Error::custom(
            "'emails' cannot exceed 1000 entries",
        ));
    }
    Ok(v)
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkSendResponse {
    pub batch_id: String,
    pub message_ids: Vec<String>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub message_id: String,
    pub to_addrs: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub bcc_addrs: Vec<String>,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub sent_at: Option<NaiveDateTime>,
    pub delivered_at: Option<NaiveDateTime>,
    pub bounce_reason: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<MailMessage> for MessageResponse {
    fn from(m: MailMessage) -> Self {
        Self {
            id: m.id,
            message_id: m.message_id,
            to_addrs: m.to_addrs,
            cc_addrs: m.cc_addrs,
            bcc_addrs: m.bcc_addrs,
            subject: m.subject,
            status: m.status,
            priority: m.priority,
            scheduled_at: m.scheduled_at,
            sent_at: m.sent_at,
            delivered_at: m.delivered_at,
            bounce_reason: m.bounce_reason,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    pub status_filter: Option<String>,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    50
}

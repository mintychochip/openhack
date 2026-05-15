use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MailAttachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i32,
    pub storage_path: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentUploadResponse {
    pub file_id: String,
    pub url: String,
    pub filename: String,
    pub size: i64,
    pub content_type: String,
}

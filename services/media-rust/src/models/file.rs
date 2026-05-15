use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a file record in the `media.files` table.
///
/// # Expected Behavior
///
/// Maps to the `media.files` table. Contains the file's unique ID, original
/// filename, content type, size in bytes, storage path, storage provider
/// identifier, uploader user ID, timestamps, soft-delete marker, and
/// arbitrary metadata as JSONB. Used for both database queries and as the
/// source of truth for file metadata.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileRecord {
    pub id: Uuid,
    pub original_filename: String,
    pub content_type: String,
    pub size: i64,
    pub storage_path: String,
    pub storage_provider: String,
    pub uploaded_by: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
    pub metadata: Option<serde_json::Value>,
}

/// Response returned after a successful file upload.
///
/// # Expected Behavior
///
/// Contains the file ID, accessible URL, original filename, size in bytes,
/// and content type. The `url` field contains a relative path for local
/// storage or a signed URL for S3 storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub file_id: Uuid,
    pub url: String,
    pub filename: String,
    pub size: i64,
    pub content_type: String,
}

/// Response returned after a file deletion request.
///
/// # Expected Behavior
///
/// Contains a boolean `deleted` field indicating whether the file was
/// successfully deleted. If the file was already soft-deleted or not found,
/// `deleted` will be `false`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub deleted: bool,
}

/// Response returned when requesting a signed URL.
///
/// # Expected Behavior
///
/// Contains the presigned URL string and the expiration time in seconds.
/// The signed URL grants temporary read access to the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlResponse {
    pub signed_url: String,
    pub expires_in: u64,
}

/// Query parameters for file upload.
///
/// # Expected Behavior
///
/// Deserialized from query string. `folder` defaults to "uploads" and
/// determines the subdirectory or S3 prefix for the uploaded file.
/// `category` is optional and may be validated against an allowlist.
#[derive(Debug, Clone, Deserialize)]
pub struct UploadQuery {
    pub folder: Option<String>,
    pub category: Option<String>,
}

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AppError;
use crate::middleware::auth::get_auth_user;
use crate::models::file::{
    DeleteResponse, FileRecord, SignedUrlResponse, UploadQuery, UploadResponse,
};
use crate::services::storage::StorageProvider;

/// Mapping of allowed file categories to their permitted MIME type prefixes.
///
/// # Expected Behavior
///
/// If a `category` query parameter is provided during upload, the uploaded
/// file's content type must match one of the prefixes in the corresponding
/// category entry. If the category is not in this map, any content type
/// is accepted. If no category is provided, any content type is accepted.
pub(crate) static ALLOWED_TYPES: &[(&str, &[&str])] = &[
    ("image", &["image/"]),
    (
        "document",
        &[
            "application/pdf",
            "text/",
            "application/msword",
            "application/vnd.openxmlformats-officedocument",
        ],
    ),
    ("video", &["video/"]),
    ("audio", &["audio/"]),
];

/// Upload a file via multipart/form-data.
///
/// # Expected Behavior
///
/// Accepts a multipart form with a `file` field. Reads the file content,
/// validates its size against `MAX_FILE_SIZE` (env, default 100MB), optionally
/// validates the content type against the `ALLOWED_TYPES` category map if the
/// `category` query parameter is provided, stores the file via the configured
/// storage provider, inserts a record into `media.files`, and returns an
/// `UploadResponse` with `file_id`, url, filename, size, and `content_type`.
///
/// # Errors
///
/// - `AppError::BadRequest` if no file field is found in the multipart form.
/// - `AppError::FileTooLarge` if the file exceeds `MAX_FILE_SIZE`.
/// - `AppError::InvalidFileType` if the content type does not match the category.
/// - `AppError::UploadFailed` if the storage provider fails to store the file.
/// - `AppError::DatabaseError` if the database insert fails.
///
/// # Side Effects
///
/// - Reads the entire file body into memory (bounded by `MAX_FILE_SIZE`).
/// - Writes the file to the storage provider (filesystem or S3 network call).
/// - Writes a sidecar `.meta` JSON file for local storage (filesystem).
/// - Inserts a row into the `media.files` table (database write).
/// - Logs at INFO level on success with `file_id` and size.
pub async fn upload_file(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<dyn StorageProvider>>,
    config: web::Data<Config>,
    query: web::Query<UploadQuery>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let user = get_auth_user(&req).map_err(|e| AppError::BadRequest(e.to_string()))?;
    let _ = user;

    let folder = query
        .folder
        .clone()
        .unwrap_or_else(|| "uploads".to_string());
    let category = query.category.clone();
    let max_size = config.max_file_size;

    let mut file_data: Option<(String, String, Vec<u8>)> = None;

    while let Some(item) = payload.next().await {
        let mut field =
            item.map_err(|e| AppError::UploadFailed(format!("Multipart error: {e}")))?;
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("");

        if field_name == "file" {
            let filename = content_disposition
                .get_filename()
                .unwrap_or("unknown")
                .to_string();
            let content_type = field.content_type().map_or_else(
                || detect_mime_type(&filename).to_string(),
                std::string::ToString::to_string,
            );

            if let Some(ref cat) = category {
                validate_content_type(&content_type, cat)?;
            }

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk =
                    chunk.map_err(|e| AppError::UploadFailed(format!("Chunk read error: {e}")))?;
                data.extend_from_slice(&chunk);
                validate_file_size(data.len(), max_size)?;
            }

            file_data = Some((filename, content_type, data));
            break;
        }
    }

    let (filename, content_type, data) = file_data
        .ok_or_else(|| AppError::BadRequest("No file field found in multipart form".to_string()))?;

    let file_id = Uuid::new_v4();
    #[allow(clippy::cast_possible_wrap)]
    let size = data.len() as i64;
    let storage_path = format!("{folder}/{file_id}");

    storage
        .put_object(&storage_path, &data, &content_type)
        .await
        .map_err(|e| AppError::UploadFailed(format!("Storage write failed: {e}")))?;

    let uploaded_by: Option<Uuid> = None;

    let metadata = serde_json::json!({
        "folder": folder,
        "category": category,
    });

    sqlx::query_as::<_, FileRecord>(
        r"
        INSERT INTO media.files (id, original_filename, content_type, size, storage_path, storage_provider, uploaded_by, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, original_filename, content_type, size, storage_path, storage_provider, uploaded_by, created_at, updated_at, deleted_at, metadata
        ",
    )
    .bind(file_id)
    .bind(&filename)
    .bind(&content_type)
    .bind(size)
    .bind(&storage_path)
    .bind(storage.provider_name())
    .bind(uploaded_by)
    .bind(&metadata)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database insert failed: {e}");
        AppError::DatabaseError(e)
    })?;

    let url = format!("/api/media/{file_id}");

    log::info!("File uploaded: id={file_id}, name={filename}, size={size}");

    Ok(HttpResponse::Ok().json(UploadResponse {
        file_id,
        url,
        filename,
        size,
        content_type,
    }))
}

/// Download a file by its ID.
///
/// # Expected Behavior
///
/// Looks up the file record by ID in `media.files`. If found and not
/// soft-deleted, retrieves the file content from the storage provider
/// and returns it as a streaming response with `Content-Type` and
/// `Content-Disposition: attachment; filename="<original_filename>"`.
///
/// # Errors
///
/// - `AppError::NotFound` if the file record does not exist or is soft-deleted.
/// - `AppError::StorageError` if the storage provider fails to retrieve the file.
///
/// # Side Effects
///
/// - Reads from the `media.files` table (database read).
/// - Reads the file from the storage provider (filesystem or S3 network call).
/// - Logs at INFO level on successful download.
pub async fn download_file(
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<dyn StorageProvider>>,
    file_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let record = sqlx::query_as::<_, FileRecord>(
        "SELECT id, original_filename, content_type, size, storage_path, storage_provider, uploaded_by, created_at, updated_at, deleted_at, metadata FROM media.files WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(file_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(AppError::DatabaseError)?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let data = storage
        .get_object(&record.storage_path)
        .await
        .map_err(|e| AppError::StorageError(format!("Failed to retrieve file: {e}")))?;

    log::info!(
        "File downloaded: id={}, name={}",
        record.id,
        record.original_filename
    );

    Ok(HttpResponse::Ok()
        .content_type(record.content_type.clone())
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", record.original_filename),
        ))
        .body(data))
}

/// Delete a file by its ID (soft delete).
///
/// # Expected Behavior
///
/// Looks up the file record by ID in `media.files`. If found and not already
/// soft-deleted, sets `deleted_at` to the current timestamp and deletes the
/// file from the storage provider. Returns `{"deleted": true}`. If the file
/// is already soft-deleted or not found, returns `{"deleted": false}`.
///
/// # Errors
///
/// - `AppError::DatabaseError` if the database update fails.
/// - `AppError::StorageError` if the storage provider fails to delete the file.
///
/// # Side Effects
///
/// - Reads from and updates the `media.files` table (database read/write).
/// - Deletes the file from the storage provider (filesystem delete or S3 network call).
/// - Logs at INFO level on successful deletion.
pub async fn delete_file(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<dyn StorageProvider>>,
    file_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user = get_auth_user(&req).map_err(|e| AppError::BadRequest(e.to_string()))?;
    let _ = user;

    let id = file_id.into_inner();

    let record = sqlx::query_as::<_, FileRecord>(
        "SELECT id, original_filename, content_type, size, storage_path, storage_provider, uploaded_by, created_at, updated_at, deleted_at, metadata FROM media.files WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(AppError::DatabaseError)?;

    match record {
        Some(rec) => {
            let now = chrono::Utc::now().naive_utc();

            sqlx::query("UPDATE media.files SET deleted_at = $1, updated_at = $1 WHERE id = $2")
                .bind(now)
                .bind(id)
                .execute(pool.get_ref())
                .await
                .map_err(AppError::DatabaseError)?;

            if let Err(e) = storage.delete_object(&rec.storage_path).await {
                log::warn!("Failed to delete file from storage: {e}");
            }

            log::info!("File deleted: id={id}");

            Ok(HttpResponse::Ok().json(DeleteResponse { deleted: true }))
        }
        None => Ok(HttpResponse::Ok().json(DeleteResponse { deleted: false })),
    }
}

/// Get a presigned URL for a file.
///
/// # Expected Behavior
///
/// Looks up the file record by ID in `media.files`. If found and not
/// soft-deleted, requests a presigned URL from the storage provider with
/// the specified expiration time (query parameter `expiration`, default
/// 3600 seconds). Returns `{"signed_url": "...", "expires_in": N}`.
/// For local storage, returns a relative URL path since presigned URLs
/// are not applicable.
///
/// # Errors
///
/// - `AppError::NotFound` if the file record does not exist or is soft-deleted.
/// - `AppError::StorageError` if the storage provider fails to generate a signed URL.
/// - `AppError::BadRequest` if the expiration parameter cannot be parsed.
///
/// # Side Effects
///
/// - Reads from the `media.files` table (database read).
/// - May perform S3 API call to generate a presigned URL (network).
/// - Logs at INFO level on successful URL generation.
pub async fn get_signed_url(
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<dyn StorageProvider>>,
    file_id: web::Path<Uuid>,
    query: web::Query<SignedUrlQuery>,
) -> Result<HttpResponse, AppError> {
    let expiration = query.expiration.unwrap_or(3600);

    let record = sqlx::query_as::<_, FileRecord>(
        "SELECT id, original_filename, content_type, size, storage_path, storage_provider, uploaded_by, created_at, updated_at, deleted_at, metadata FROM media.files WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(file_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(AppError::DatabaseError)?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let signed_url = storage
        .presign_get(&record.storage_path, expiration)
        .await
        .map_err(|e| AppError::StorageError(format!("Failed to generate signed URL: {e}")))?;

    log::info!(
        "Signed URL generated: id={}, expires_in={}",
        record.id,
        expiration
    );

    Ok(HttpResponse::Ok().json(SignedUrlResponse {
        signed_url,
        expires_in: expiration,
    }))
}

/// Validate that a file size does not exceed the maximum allowed size.
///
/// # Expected Behavior
///
/// Compares `size` against `max_size`. If `size` exceeds `max_size`, returns
/// `AppError::FileTooLarge` with a descriptive message. A file whose size
/// equals `max_size` is accepted (inclusive upper bound). A zero-byte file
/// is always accepted.
///
/// # Errors
///
/// - `AppError::FileTooLarge` if `size > max_size`.
///
/// # Side Effects
///
/// None. Pure validation with no I/O.
pub(crate) fn validate_file_size(size: usize, max_size: usize) -> Result<(), AppError> {
    if size > max_size {
        Err(AppError::FileTooLarge(format!(
            "File exceeds maximum size of {max_size} bytes"
        )))
    } else {
        Ok(())
    }
}

/// Detect a MIME type from a file's extension.
///
/// # Expected Behavior
///
/// Extracts the last segment after the final '.' in `filename` (case-insensitive)
/// and maps it to a well-known MIME type. Supports common image, document,
/// video, and audio formats. Returns "application/octet-stream" for unknown
/// extensions, files without extensions, or empty filenames.
///
/// # Errors
///
/// None. Always returns a valid `&'static str`.
///
/// # Side Effects
///
/// None. Pure function with no I/O.
pub(crate) fn detect_mime_type(filename: &str) -> &'static str {
    let extension = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

/// Query parameters for signed URL requests.
///
/// # Expected Behavior
///
/// Deserialized from query string. `expiration` defaults to 3600 seconds
/// if not provided. Must be a positive integer.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SignedUrlQuery {
    pub expiration: Option<u64>,
}

/// Validate that a content type matches the allowed types for a category.
///
/// # Expected Behavior
///
/// Looks up the category in the `ALLOWED_TYPES` static map. If found,
/// checks that the content type starts with at least one of the category's
/// allowed prefixes. If the category is not in the map, the content type
/// is accepted (no restriction). If the content type does not match any
/// allowed prefix, returns `AppError::InvalidFileType`.
///
/// # Errors
///
/// - `AppError::InvalidFileType` if the content type does not match the
///   allowed prefixes for the given category.
///
/// # Side Effects
///
/// None. Pure validation with no I/O.
pub(crate) fn validate_content_type(content_type: &str, category: &str) -> Result<(), AppError> {
    for (cat, prefixes) in ALLOWED_TYPES {
        if cat == &category {
            let allowed = prefixes
                .iter()
                .any(|prefix| content_type.starts_with(prefix));
            if !allowed {
                return Err(AppError::InvalidFileType(format!(
                    "Content type '{content_type}' is not allowed for category '{category}'"
                )));
            }
            return Ok(());
        }
    }
    Ok(())
}

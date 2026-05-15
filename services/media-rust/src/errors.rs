use actix_web::{http::StatusCode, HttpResponse};
use thiserror::Error;

/// Error types for the media service.
///
/// # Expected Behavior
///
/// Used throughout the media service to represent storage failures,
/// database errors, file not found, validation errors, and I/O errors.
/// Implements `actix_web::ResponseError` so handlers can return `AppError`
/// directly and it will be converted to an appropriate HTTP response.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("File too large: {0}")]
    FileTooLarge(String),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("S3 error: {0}")]
    S3Error(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    _InternalError(String),
}

impl actix_web::ResponseError for AppError {
    /// Map `AppError` variants to HTTP status codes.
    ///
    /// # Expected Behavior
    ///
    /// Returns 404 for `NotFound`, 413 for `FileTooLarge`, 415 for
    /// `InvalidFileType`, 400 for `BadRequest`, 500 for all others.
    ///
    /// # Errors
    ///
    /// None. Always returns a valid `StatusCode`.
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::FileTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::InvalidFileType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::UploadFailed(_)
            | AppError::StorageError(_)
            | AppError::DatabaseError(_)
            | AppError::IoError(_)
            | AppError::S3Error(_)
            | AppError::_InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convert `AppError` into an HTTP response with a JSON error body.
    ///
    /// # Expected Behavior
    ///
    /// Produces a JSON body `{"error": "<message>"}` with the appropriate
    /// status code. The error message is the display representation of the
    /// `AppError` variant.
    ///
    /// # Errors
    ///
    /// None. Always produces a valid `HttpResponse`.
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}

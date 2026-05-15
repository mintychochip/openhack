use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;

/// Error types for the sponsors service.
///
/// # Expected Behavior
///
/// Used throughout the sponsors service to represent not-found conditions,
/// validation failures, authorization errors, and database/Redis operation
/// failures. Each variant can be converted to an appropriate HTTP response
/// via `to_http_response()`.
#[derive(Debug, Error)]
pub enum SponsorError {
    /// A requested resource (booth, prize, submission) was not found.
    #[error("{0} not found: {1}")]
    NotFound(String, String),

    /// The request data failed validation.
    #[error("Validation error: {0}")]
    _Validation(String),

    /// The authenticated user lacks the required role.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// The request lacks valid authentication credentials.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// A database operation failed.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// A Redis operation failed.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// An internal/unexpected error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl SponsorError {
    /// Convert this error into an appropriate HTTP response.
    ///
    /// # Expected Behavior
    ///
    /// Maps each error variant to an HTTP status code and JSON body:
    /// - `NotFound` → 404 Not Found
    /// - `Validation` → 400 Bad Request
    /// - `Forbidden` → 403 Forbidden
    /// - `Unauthorized` → 401 Unauthorized
    /// - `Database`/`Redis`/`Internal` → 500 Internal Server Error (message
    ///   is sanitized to "Internal server error" to avoid leaking details)
    ///
    /// # Errors
    ///
    /// None. Always returns a valid `HttpResponse`.
    ///
    /// # Side Effects
    ///
    /// - Logs at ERROR level for internal/database/redis errors.
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            SponsorError::NotFound(_, _) => HttpResponse::NotFound().json(ErrorBody {
                error: self.to_string(),
            }),
            SponsorError::_Validation(_) => HttpResponse::BadRequest().json(ErrorBody {
                error: self.to_string(),
            }),
            SponsorError::Forbidden(_) => HttpResponse::Forbidden().json(ErrorBody {
                error: self.to_string(),
            }),
            SponsorError::Unauthorized(_) => HttpResponse::Unauthorized().json(ErrorBody {
                error: self.to_string(),
            }),
            _ => {
                log::error!("Internal error: {self:?}");
                HttpResponse::InternalServerError().json(ErrorBody {
                    error: "Internal server error".into(),
                })
            }
        }
    }
}

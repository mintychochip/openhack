use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;

/// Error types for the Analytics service.
///
/// # Expected Behavior
///
/// Used throughout the Analytics service to represent database failures,
/// missing resources, invalid requests, CSV generation errors, and
/// general internal errors.
#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}: {1}")]
    NotFound(String, String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("CSV export error: {0}")]
    CsvExport(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl AnalyticsError {
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            AnalyticsError::NotFound(_, _) => HttpResponse::NotFound().json(ErrorBody {
                error: self.to_string(),
            }),
            AnalyticsError::BadRequest(_) | AnalyticsError::Validation(_) => {
                HttpResponse::BadRequest().json(ErrorBody {
                    error: self.to_string(),
                })
            }
            AnalyticsError::Unauthorized(_) => HttpResponse::Unauthorized().json(ErrorBody {
                error: self.to_string(),
            }),
            AnalyticsError::Forbidden(_) => HttpResponse::Forbidden().json(ErrorBody {
                error: self.to_string(),
            }),
            AnalyticsError::CsvExport(_) | AnalyticsError::Internal(_) => {
                log::error!("Internal error: {self:?}");
                HttpResponse::InternalServerError().json(ErrorBody {
                    error: "Internal server error".into(),
                })
            }
            AnalyticsError::Database(_) => {
                log::error!("Database error: {self:?}");
                HttpResponse::InternalServerError().json(ErrorBody {
                    error: "Database error".into(),
                })
            }
        }
    }
}

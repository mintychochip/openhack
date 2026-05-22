use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JudgingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Rubric not found: {0}")]
    RubricNotFound(String),
    #[error("Assignment not found: {0}")]
    AssignmentNotFound(String),
    #[error("Score not found: {0}")]
    ScoreNotFound(String),
    #[error("Phase not found: {0}")]
    PhaseNotFound(String),
    #[error("Phase is closed: {0}")]
    PhaseClosed(String),
    #[error("Phase is not finalized: {0}")]
    PhaseNotFinalized(String),
    #[error("Score validation error: {0}")]
    ScoreValidation(String),
    #[error("Normalization error: {0}")]
    Normalization(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl JudgingError {
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            JudgingError::RubricNotFound(_)
            | JudgingError::AssignmentNotFound(_)
            | JudgingError::ScoreNotFound(_)
            | JudgingError::PhaseNotFound(_) => HttpResponse::NotFound().json(ErrorBody {
                error: self.to_string(),
            }),
            JudgingError::PhaseClosed(_) | JudgingError::PhaseNotFinalized(_) => {
                HttpResponse::Conflict().json(ErrorBody {
                    error: self.to_string(),
                })
            }
            JudgingError::ScoreValidation(_) | JudgingError::Validation(_) => {
                HttpResponse::BadRequest().json(ErrorBody {
                    error: self.to_string(),
                })
            }
            JudgingError::Unauthorized(_) => HttpResponse::Unauthorized().json(ErrorBody {
                error: self.to_string(),
            }),
            JudgingError::Forbidden(_) => HttpResponse::Forbidden().json(ErrorBody {
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

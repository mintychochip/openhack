use actix_web::{http::StatusCode, HttpResponse};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ServiceError {
    #[must_use]
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::NotFound(format!("{entity} not found: {id}"))
    }

    #[must_use]
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    #[must_use]
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    #[must_use]
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    #[must_use]
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    #[must_use]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) | Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Database(_) | Self::Redis(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    #[must_use]
    pub fn user_facing_message(&self) -> String {
        match self {
            Self::Database(_) | Self::Redis(_) | Self::Internal(_) => {
                "Internal server error".to_string()
            }
            Self::Jwt(_) => "Invalid or expired token".to_string(),
            Self::RateLimited => "Rate limit exceeded".to_string(),
            other => other.to_string(),
        }
    }

    #[must_use]
    pub fn to_http_response(&self) -> HttpResponse {
        let status = self.status_code();
        if status >= StatusCode::INTERNAL_SERVER_ERROR {
            log::error!("Internal error: {self:?}");
        }
        HttpResponse::build(status).json(ErrorBody {
            error: self.user_facing_message(),
        })
    }
}

impl actix_web::ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        self.to_http_response()
    }

    fn status_code(&self) -> StatusCode {
        self.status_code()
    }
}

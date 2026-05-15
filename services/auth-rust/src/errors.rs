use actix_web::{http::StatusCode, HttpResponse};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
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
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Password hash error: {0}")]
    HashError(String),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("MFA error: {0}")]
    MfaError(String),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl actix_web::ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            AuthError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AuthError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AuthError::BadRequest(msg) | AuthError::MfaError(msg) | AuthError::OAuth(msg) => {
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AuthError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AuthError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AuthError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),
            AuthError::DatabaseError(e) => {
                log::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AuthError::RedisError(e) => {
                log::error!("Redis error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AuthError::HashError(msg) => {
                log::error!("Hash error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AuthError::JwtError(e) => {
                log::error!("JWT error: {e}");
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token".to_string(),
                )
            }
            AuthError::InternalError(msg) => {
                log::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        HttpResponse::build(status).json(serde_json::json!({ "error": message }))
    }
}

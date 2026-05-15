use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Message not found: {0}")]
    MessageNotFound(String),
    #[error("Broadcast not found: {0}")]
    BroadcastNotFound(String),
    #[allow(dead_code)]
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    #[allow(dead_code)]
    #[error("Validation error: {0}")]
    Validation(String),
    #[allow(dead_code)]
    #[error("SMTP error: {0}")]
    Smtp(String),
    #[error("Template render error: {0}")]
    TemplateRender(String),
    #[allow(dead_code)]
    #[error("Attachment error: {0}")]
    Attachment(String),
    #[allow(dead_code)]
    #[error("Webhook error: {0}")]
    Webhook(String),
    #[allow(dead_code)]
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl MailError {
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            MailError::TemplateNotFound(_)
            | MailError::MessageNotFound(_)
            | MailError::BroadcastNotFound(_) => HttpResponse::NotFound().json(ErrorBody {
                error: self.to_string(),
            }),
            MailError::InvalidEmail(_) | MailError::Validation(_) => HttpResponse::BadRequest()
                .json(ErrorBody {
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

impl From<tera::Error> for MailError {
    fn from(e: tera::Error) -> Self {
        MailError::TemplateRender(e.to_string())
    }
}

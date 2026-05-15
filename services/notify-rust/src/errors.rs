use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotifyError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Announcement not found: {0}")]
    AnnouncementNotFound(String),
    #[error("Webhook not found: {0}")]
    WebhookNotFound(String),
    #[error("Notification not found: {0}")]
    NotificationNotFound(String),
    #[allow(dead_code)]
    #[error("Webhook delivery failed: {0}")]
    WebhookDeliveryFailed(String),
    #[error("Discord error: {0}")]
    DiscordError(String),
    #[error("Slack error: {0}")]
    SlackError(String),
    #[allow(dead_code)]
    #[error("Validation error: {0}")]
    Validation(String),
    #[allow(dead_code)]
    #[error("Unauthorized")]
    Unauthorized,
    #[allow(dead_code)]
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl NotifyError {
    /// Convert this error to an appropriate HTTP response.
    ///
    /// # Expected Behavior
    ///
    /// Maps each error variant to the most appropriate HTTP status code.
    /// `NotFound` variants return 404, Unauthorized returns 401, Validation
    /// returns 400, `WebhookDeliveryFailed` returns 502, `DiscordError` and
    /// `SlackError` return 502. All other errors return 500 with a generic
    /// message and log the full error at ERROR level.
    ///
    /// # Errors
    ///
    /// None. Always produces a valid `HttpResponse`.
    ///
    /// # Side Effects
    ///
    /// - Logs at ERROR level for internal/unexpected errors.
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            NotifyError::AnnouncementNotFound(_)
            | NotifyError::WebhookNotFound(_)
            | NotifyError::NotificationNotFound(_) => HttpResponse::NotFound().json(ErrorBody {
                error: self.to_string(),
            }),
            NotifyError::Unauthorized => HttpResponse::Unauthorized().json(ErrorBody {
                error: "Unauthorized".into(),
            }),
            NotifyError::Validation(_) => HttpResponse::BadRequest().json(ErrorBody {
                error: self.to_string(),
            }),
            NotifyError::WebhookDeliveryFailed(_)
            | NotifyError::DiscordError(_)
            | NotifyError::SlackError(_) => HttpResponse::BadGateway().json(ErrorBody {
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

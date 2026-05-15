use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Channel config not found: {0}")]
    ChannelConfigNotFound(String),

    #[error("User link not found: {0}")]
    UserLinkNotFound(String),

    #[error("Link token not found or expired")]
    LinkTokenInvalid,

    #[error("Discord API error: {0}")]
    DiscordError(String),

    #[error("OpenHack API error: {0}")]
    OpenHackError(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bot not configured: {0}")]
    NotConfigured(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl BotError {
    /// Convert this error to an appropriate HTTP response.
    ///
    /// # Expected Behavior
    ///
    /// Maps each error variant to the most appropriate HTTP status code.
    /// `NotFound` variants return 404, `Unauthorized` returns 401,
    /// `Validation` returns 400, `DiscordError` and `OpenHackError` return 502,
    /// `NotConfigured` returns 503. All other errors return 500 with a
    /// generic message and log the full error at ERROR level.
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
            BotError::ChannelConfigNotFound(_) | BotError::UserLinkNotFound(_) => {
                HttpResponse::NotFound().json(ErrorBody {
                    error: self.to_string(),
                })
            }
            BotError::LinkTokenInvalid => HttpResponse::Gone().json(ErrorBody {
                error: self.to_string(),
            }),
            BotError::Unauthorized => HttpResponse::Unauthorized().json(ErrorBody {
                error: "Unauthorized".into(),
            }),
            BotError::Validation(_) => HttpResponse::BadRequest().json(ErrorBody {
                error: self.to_string(),
            }),
            BotError::DiscordError(_) | BotError::OpenHackError(_) => HttpResponse::BadGateway()
                .json(ErrorBody {
                    error: self.to_string(),
                }),
            BotError::NotConfigured(_) => HttpResponse::ServiceUnavailable().json(ErrorBody {
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

impl actix_web::ResponseError for BotError {
    /// Convert this error into an Actix-web error with the appropriate HTTP status.
    ///
    /// # Expected Behavior
    ///
    /// Delegates to `to_http_response()` for status code and body mapping.
    /// The `ResponseError` trait provides `From<Self> for Error` via blanket
    /// impl, enabling the `?` operator in handlers that return
    /// `Result<_, actix_web::Error>`.
    ///
    /// # Errors
    ///
    /// None. Always produces a valid error.
    ///
    /// # Side Effects
    ///
    /// - Logs at ERROR level for internal/unexpected errors (via `to_http_response`).
    fn error_response(&self) -> HttpResponse {
        self.to_http_response()
    }
}

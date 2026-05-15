use thiserror::Error;

/// Error types for the AI service.
///
/// # Expected Behavior
///
/// Used throughout the AI service to represent database failures, Redis failures,
/// LLM provider errors, embedding errors, missing resources, disabled features,
/// invalid requests, and HTTP client failures.
#[derive(Debug, Error)]
pub enum AiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("LLM provider error: {0}")]
    Llm(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Feature disabled: {0}")]
    #[allow(dead_code)]
    FeatureDisabled(String),

    #[error("Bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),
}

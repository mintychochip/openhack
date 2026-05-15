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

    #[error("Not found: {0}")]
    _NotFound(String),

    #[error("Bad request: {0}")]
    _BadRequest(String),

    #[error("CSV export error: {0}")]
    CsvExport(String),

    #[error("Internal error: {0}")]
    _Internal(String),
}

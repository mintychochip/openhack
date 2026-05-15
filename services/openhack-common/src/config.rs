use std::env;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingVar(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct BaseConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub rust_log: String,
}

impl BaseConfig {
    /// Load shared service configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::MissingVar` if `DATABASE_URL` or `JWT_SECRET` is not set.
    /// Returns `ConfigError::Invalid` if `JWT_SECRET` is less than 32 characters
    /// or if `SERVICE_PORT`/`PORT` cannot be parsed as `u16`.
    pub fn from_env(default_port: u16) -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingVar("DATABASE_URL".to_string()))?;
        let redis_url = env::var("REDIS_URL").unwrap_or_default();
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingVar("JWT_SECRET".to_string()))?;
        if jwt_secret.len() < 32 {
            return Err(ConfigError::Invalid(
                "JWT_SECRET must be at least 32 characters".to_string(),
            ));
        }
        let port: u16 = env::var("SERVICE_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| default_port.to_string())
            .parse::<u16>()
            .map_err(|e| ConfigError::Invalid(format!("PORT must be a valid u16: {e}")))?;
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            database_url,
            redis_url,
            jwt_secret,
            port,
            rust_log,
        })
    }

    #[must_use]
    pub fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    #[must_use]
    pub fn env_or_empty(key: &str) -> String {
        env::var(key).unwrap_or_default()
    }
}

/// Parse a duration environment variable into seconds.
///
/// # Errors
///
/// Returns `ConfigError::Invalid` if the value cannot be parsed as a duration
/// (e.g., non-numeric suffix, overflow).
pub fn duration_secs(key: &str, default: &str) -> Result<i64, ConfigError> {
    let val = env::var(key).unwrap_or_else(|_| default.to_string());
    parse_duration(&val).map_err(|e| ConfigError::Invalid(format!("{key}: {e}")))
}

fn parse_duration(s: &str) -> Result<i64, String> {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix('s') {
        stripped
            .parse::<i64>()
            .map_err(|e| format!("Invalid duration: {e}"))
    } else if let Some(stripped) = s.strip_suffix('m') {
        stripped
            .parse::<i64>()
            .map(|v| v.saturating_mul(60))
            .map_err(|e| format!("Invalid duration: {e}"))
    } else if let Some(stripped) = s.strip_suffix('h') {
        stripped
            .parse::<i64>()
            .map(|v| v.saturating_mul(3600))
            .map_err(|e| format!("Invalid duration: {e}"))
    } else if let Some(stripped) = s.strip_suffix('d') {
        stripped
            .parse::<i64>()
            .map(|v| v.saturating_mul(86400))
            .map_err(|e| format!("Invalid duration: {e}"))
    } else {
        s.parse::<i64>()
            .map_err(|e| format!("Invalid duration: {e}"))
    }
}

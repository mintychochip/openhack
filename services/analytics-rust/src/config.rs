use std::env;

/// Application configuration loaded from environment variables.
///
/// # Expected Behavior
///
/// Reads `DATABASE_URL`, `REDIS_URL`, `SERVICE_PORT/PORT`, and `RUST_LOG` from
/// environment variables. Falls back to sensible defaults for optional
/// values. `DATABASE_URL` is required. `SERVICE_PORT` defaults to 3008.
/// `REDIS_URL` defaults to "<redis://redis:6379>". `RUST_LOG` defaults to "info".
///
/// # Errors
///
/// Panics if `DATABASE_URL` is not set in the environment.
///
/// # Side Effects
///
/// - Calls `std::env::var` for each configuration key (read-only, no mutations).
/// - No I/O, network, or file operations.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub _redis_url: String,
    pub port: u16,
    pub _rust_log: String,
}

impl Config {
    /// Build configuration from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads each required and optional environment variable. `SERVICE_PORT` or PORT
    /// defaults to 3008. `REDIS_URL` defaults to "<redis://redis:6379>". `RUST_LOG`
    /// defaults to "info". `DATABASE_URL` is required and will cause a panic if missing.
    ///
    /// # Errors
    ///
    /// Panics with a descriptive message if `DATABASE_URL` environment variable
    /// is not set. Panics if `SERVICE_PORT/PORT` cannot be parsed as u16.
    ///
    /// # Side Effects
    ///
    /// - Reads environment variables (read-only).
    pub fn from_env() -> Self {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".to_string());
        let port: u16 = env::var("SERVICE_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "3008".to_string())
            .parse()
            .expect("SERVICE_PORT/PORT must be a valid u16");
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Self {
            database_url,
            _redis_url: redis_url,
            port,
            _rust_log: rust_log,
        }
    }
}

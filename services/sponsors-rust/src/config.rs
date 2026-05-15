use std::env;

/// Application configuration loaded from environment variables.
///
/// # Expected Behavior
///
/// Reads `DATABASE_URL`, `REDIS_URL`, `JWT_SECRET`, `SERVICE_PORT` (or PORT), and
/// `RUST_LOG` from environment variables. Falls back to sensible defaults for
/// `SERVICE_PORT` (3009) and `RUST_LOG` ("info"). `DATABASE_URL` and `JWT_SECRET`
/// have no defaults and must be set or the service will panic at startup.
/// `REDIS_URL` is optional; if not set, caching is disabled and the service
/// operates without Redis. `SERVICE_PORT` is read first; if unset, PORT is used
/// as fallback.
///
/// # Errors
///
/// Panics if `DATABASE_URL` or `JWT_SECRET` are not set in the environment,
/// or if the port value cannot be parsed as u16.
///
/// # Side Effects
///
/// - Calls `std::env::var` for each configuration key (read-only, no mutations).
/// - No I/O, network, or file operations.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub jwt_secret: String,
    pub port: u16,
    pub rust_log: String,
}

impl Config {
    /// Build configuration from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads each required and optional environment variable. `SERVICE_PORT`
    /// defaults to 3009 if unset; PORT is used as fallback if `SERVICE_PORT` is
    /// not set. `RUST_LOG` defaults to "info". `DATABASE_URL` and `JWT_SECRET`
    /// are required and will cause a panic if missing. `REDIS_URL` is optional;
    /// if unset, caching is disabled and the service operates without Redis.
    ///
    /// # Errors
    ///
    /// Panics with a descriptive message if `DATABASE_URL` or `JWT_SECRET`
    /// environment variables are not set, or if the port value cannot be parsed
    /// as u16.
    ///
    /// # Side Effects
    ///
    /// - Reads environment variables (read-only).
    pub fn from_env() -> Self {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
        let redis_url = env::var("REDIS_URL").ok();
        let jwt_secret =
            env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set");
        let port: u16 = env::var("SERVICE_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "3009".to_string())
            .parse()
            .expect("PORT must be a valid u16");
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Self {
            database_url,
            redis_url,
            jwt_secret,
            port,
            rust_log,
        }
    }
}

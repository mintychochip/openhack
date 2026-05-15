use std::env;

/// Application configuration loaded from environment variables.
///
/// # Expected Behavior
///
/// Reads `DATABASE_URL`, `SERVICE_PORT` (or PORT), `STORAGE_PATH`, `STORAGE_PROVIDER`,
/// `MAX_FILE_SIZE`, `S3_BUCKET_NAME`, `S3_REGION`, `S3_ENDPOINT`, `S3_ACCESS_KEY`,
/// `S3_SECRET_KEY`, `LOG_LEVEL`, and `RUST_LOG` from environment variables.
/// Falls back to sensible defaults for `SERVICE_PORT` (3010), `STORAGE_PATH` ("./uploads"),
/// `STORAGE_PROVIDER` ("local"), and `MAX_FILE_SIZE` (104857600 = 100MB).
/// `DATABASE_URL` is required and will panic if missing.
///
/// # Errors
///
/// Panics if `DATABASE_URL` environment variable is not set.
/// Panics if `SERVICE_PORT` or `MAX_FILE_SIZE` cannot be parsed as their expected types.
///
/// # Side Effects
///
/// - Calls `std::env::var` for each configuration key (read-only, no mutations).
/// - No I/O, network, or file operations.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub storage_path: String,
    pub storage_provider: String,
    pub max_file_size: usize,
    pub _s3_bucket_name: String,
    pub _s3_region: String,
    pub _s3_endpoint: String,
    pub _s3_access_key: String,
    pub _s3_secret_key: String,
    pub _log_level: String,
    pub rust_log: String,
}

impl Config {
    /// Build configuration from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads each required and optional environment variable. `SERVICE_PORT`
    /// takes precedence over PORT; if neither is set, defaults to 3010.
    /// `STORAGE_PATH` defaults to "./uploads". `STORAGE_PROVIDER` defaults to "local".
    /// `MAX_FILE_SIZE` defaults to 104857600 (100MB). S3_* variables default to
    /// empty strings and are only required when `STORAGE_PROVIDER` is "s3".
    /// `DATABASE_URL` is required and will cause a panic if missing.
    ///
    /// # Errors
    ///
    /// Panics with a descriptive message if `DATABASE_URL` is not set.
    /// Panics if `SERVICE_PORT` or `MAX_FILE_SIZE` cannot be parsed.
    ///
    /// # Side Effects
    ///
    /// - Reads environment variables (read-only).
    pub fn from_env() -> Self {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        let port: u16 = env::var("SERVICE_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "3010".to_string())
            .parse()
            .expect("SERVICE_PORT/PORT must be a valid u16");

        let storage_path = env::var("STORAGE_PATH").unwrap_or_else(|_| "./uploads".to_string());
        let storage_provider = env::var("STORAGE_PROVIDER").unwrap_or_else(|_| "local".to_string());
        let max_file_size: usize = env::var("MAX_FILE_SIZE")
            .unwrap_or_else(|_| "104857600".to_string())
            .parse()
            .expect("MAX_FILE_SIZE must be a valid usize");

        let s3_bucket_name = env::var("S3_BUCKET_NAME").unwrap_or_default();
        let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let s3_endpoint = env::var("S3_ENDPOINT").unwrap_or_default();
        let s3_access_key = env::var("S3_ACCESS_KEY").unwrap_or_default();
        let s3_secret_key = env::var("S3_SECRET_KEY").unwrap_or_default();

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Self {
            database_url,
            port,
            storage_path,
            storage_provider,
            max_file_size,
            _s3_bucket_name: s3_bucket_name,
            _s3_region: s3_region,
            _s3_endpoint: s3_endpoint,
            _s3_access_key: s3_access_key,
            _s3_secret_key: s3_secret_key,
            _log_level: log_level,
            rust_log,
        }
    }
}

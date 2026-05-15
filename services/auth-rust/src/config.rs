use std::env;

/// Application configuration loaded from environment variables.
///
/// # Expected Behavior
///
/// Reads `DATABASE_URL`, `SERVICE_PORT/PORT`, `JWT_SECRET`, `JWT_EXPIRY`,
/// `REFRESH_TOKEN_EXPIRY`, `REDIS_URL`, and optional OAuth provider credentials from
/// environment variables. Falls back to sensible defaults for PORT (3001),
/// `JWT_EXPIRY` ("15m"), and `REFRESH_TOKEN_EXPIRY` ("7d"). `DATABASE_URL`
/// and `JWT_SECRET` are required and will cause a panic if missing.
/// `REDIS_URL` defaults to empty if not set, which disables Redis (caching and events).
/// `JWT_SECRET` must be at least 32 characters. OAuth credentials default to
/// empty strings if not set, disabling that provider.
///
/// # Errors
///
/// Panics if `DATABASE_URL` or `JWT_SECRET` are not set, or if
/// `JWT_SECRET` is shorter than 32 characters. Panics if PORT cannot be
/// parsed as u16. `REDIS_URL` is optional and defaults to empty.
///
/// # Side Effects
///
/// - Calls `std::env::var` for each configuration key (read-only, no mutations).
/// - No I/O, network, or file operations.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub _jwt_expiry: String,
    pub jwt_expiry_secs: i64,
    pub refresh_token_expiry_secs: i64,
    pub rust_log: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub frontend_url: String,
}

/// Parse a human-readable duration string into seconds.
///
/// # Expected Behavior
///
/// Accepts strings like "30s", "15m", "1h", "7d". The suffix indicates the
/// unit: s=seconds, m=minutes, h=hours, d=days. If no suffix is provided,
/// the value is interpreted as seconds. Returns the total number of seconds.
///
/// # Errors
///
/// Returns a String describing the parse error if the numeric portion cannot
/// be parsed as i64.
///
/// # Side Effects
///
/// None.
fn parse_duration(s: &str) -> Result<i64, String> {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix('s') {
        stripped
            .parse::<i64>()
            .map_err(|e| format!("Invalid duration: {e}"))
    } else if let Some(stripped) = s.strip_suffix('m') {
        stripped
            .parse::<i64>()
            .map(|v| v * 60)
            .map_err(|e| format!("Invalid duration: {e}"))
    } else if let Some(stripped) = s.strip_suffix('h') {
        stripped
            .parse::<i64>()
            .map(|v| v * 3600)
            .map_err(|e| format!("Invalid duration: {e}"))
    } else if let Some(stripped) = s.strip_suffix('d') {
        stripped
            .parse::<i64>()
            .map(|v| v * 86400)
            .map_err(|e| format!("Invalid duration: {e}"))
    } else {
        s.parse::<i64>()
            .map_err(|e| format!("Invalid duration: {e}"))
    }
}

impl Config {
    /// Build configuration from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads each required and optional environment variable. PORT or
    /// `SERVICE_PORT` defaults to 3001 if unset. `JWT_EXPIRY` defaults to "15m"
    /// (900 seconds). `REFRESH_TOKEN_EXPIRY` defaults to "7d" (604800 seconds).
    /// `DATABASE_URL`, `REDIS_URL`, and `JWT_SECRET` are required. `JWT_SECRET` must
    /// be at least 32 characters. OAuth credentials default to empty strings.
    /// `FRONTEND_URL` defaults to "<http://localhost:3000>".
    ///
    /// # Errors
    ///
    /// Panics with a descriptive message if `DATABASE_URL` or
    /// `JWT_SECRET` environment variables are not set, or if `JWT_SECRET` is
    /// shorter than 32 characters. `REDIS_URL` is optional and defaults to empty.
    ///
    /// # Side Effects
    ///
    /// - Reads environment variables (read-only).
    pub fn from_env() -> Self {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
        let redis_url = env::var("REDIS_URL").unwrap_or_default();
        let jwt_secret =
            env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set");
        assert!(
            jwt_secret.len() >= 32,
            "JWT_SECRET must be at least 32 characters"
        );

        let port: u16 = env::var("SERVICE_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "3001".to_string())
            .parse()
            .expect("PORT must be a valid u16");

        let jwt_expiry = env::var("JWT_EXPIRY").unwrap_or_else(|_| "15m".to_string());
        let jwt_expiry_secs = parse_duration(&jwt_expiry)
            .expect("JWT_EXPIRY must be a valid duration (e.g., 15m, 1h, 7d)");

        let refresh_token_expiry =
            env::var("REFRESH_TOKEN_EXPIRY").unwrap_or_else(|_| "7d".to_string());
        let refresh_token_expiry_secs = parse_duration(&refresh_token_expiry)
            .expect("REFRESH_TOKEN_EXPIRY must be a valid duration (e.g., 7d, 30d)");

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let github_client_id = env::var("GITHUB_CLIENT_ID").unwrap_or_default();
        let github_client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
        let google_client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
        let discord_client_id = env::var("DISCORD_CLIENT_ID").unwrap_or_default();
        let discord_client_secret = env::var("DISCORD_CLIENT_SECRET").unwrap_or_default();

        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        Self {
            database_url,
            redis_url,
            port,
            jwt_secret,
            _jwt_expiry: jwt_expiry,
            jwt_expiry_secs,
            refresh_token_expiry_secs,
            rust_log,
            github_client_id,
            github_client_secret,
            google_client_id,
            google_client_secret,
            discord_client_id,
            discord_client_secret,
            frontend_url,
        }
    }
}

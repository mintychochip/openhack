use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub port: u16,
    pub rust_log: String,
    pub jwt_secret: String,
    pub discord_webhook_url: String,
    pub discord_bot_token: String,
    pub discord_guild_id: String,
    pub slack_bot_token: String,
    #[allow(dead_code)]
    pub mail_service_url: String,
    #[allow(dead_code)]
    pub from_email: String,
}

impl Config {
    /// Build configuration from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads each required and optional environment variable. `SERVICE_PORT`
    /// or PORT defaults to 3006 if unset. `JWT_SECRET` and `DATABASE_URL`
    /// are required and will cause a panic if missing. `REDIS_URL` is
    /// optional — if unset, event subscription is disabled. All Discord,
    /// Slack, and mail settings default to empty strings when unset.
    ///
    /// # Errors
    ///
    /// Panics with a descriptive message if `DATABASE_URL` or
    /// `JWT_SECRET` environment variables are not set. Panics if PORT cannot
    /// be parsed as u16. `REDIS_URL` is optional and does not panic if missing.
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
            .unwrap_or_else(|_| "3006".to_string())
            .parse()
            .expect("PORT must be a valid u16");
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        let discord_webhook_url = env::var("DISCORD_WEBHOOK_URL").unwrap_or_default();
        let discord_bot_token = env::var("DISCORD_BOT_TOKEN").unwrap_or_default();
        let discord_guild_id = env::var("DISCORD_GUILD_ID").unwrap_or_default();
        let slack_bot_token = env::var("SLACK_BOT_TOKEN").unwrap_or_default();
        let mail_service_url = env::var("MAIL_SERVICE_URL").unwrap_or_default();
        let from_email = env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@openhack.local".into());

        Self {
            database_url,
            redis_url,
            port,
            rust_log,
            jwt_secret,
            discord_webhook_url,
            discord_bot_token,
            discord_guild_id,
            slack_bot_token,
            mail_service_url,
            from_email,
        }
    }
}

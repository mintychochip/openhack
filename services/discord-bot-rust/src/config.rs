use openhack_common::config::{BaseConfig, ConfigError};

#[derive(Debug, Clone)]
pub struct Config {
    pub base: BaseConfig,
    pub lambda_internal_token: String,
    pub discord_bot_token: String,
    pub discord_application_id: String,
    pub discord_guild_id: String,
    pub discord_public_key: String,
    pub discord_bot_enabled: bool,
    pub discord_log_interactions: bool,
    pub gateway_url: String,
    pub openhack_domain: String,
}

impl Config {
    /// Build configuration from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads each required and optional environment variable. Composes
    /// `BaseConfig` for shared fields (database_url, redis_url, jwt_secret,
    /// port, rust_log). Discord-specific fields default to empty strings when
    /// unset; the bot will not start if `DISCORD_BOT_TOKEN` is empty.
    /// `DISCORD_BOT_ENABLED` defaults to "true".
    /// `DISCORD_BOT_LOG_INTERACTIONS` defaults to "true".
    /// `GATEWAY_URL` defaults to "http://gateway-svc:8000".
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::MissingVar` if `DATABASE_URL`, `JWT_SECRET`, or
    /// `LAMBDA_INTERNAL_TOKEN` environment variables are not set.
    /// Returns `ConfigError::Invalid` if `JWT_SECRET` is less than 32
    /// characters or PORT cannot be parsed as u16.
    ///
    /// # Side Effects
    ///
    /// - Reads environment variables (read-only).
    pub fn from_env() -> Result<Self, ConfigError> {
        let base = BaseConfig::from_env(3011)?;
        let lambda_internal_token = std::env::var("LAMBDA_INTERNAL_TOKEN")
            .map_err(|_| ConfigError::MissingVar("LAMBDA_INTERNAL_TOKEN".to_string()))?;
        let discord_bot_token = std::env::var("DISCORD_BOT_TOKEN").unwrap_or_default();
        let discord_application_id = std::env::var("DISCORD_APPLICATION_ID").unwrap_or_default();
        let discord_guild_id = std::env::var("DISCORD_GUILD_ID").unwrap_or_default();
        let discord_public_key = std::env::var("DISCORD_PUBLIC_KEY").unwrap_or_default();
        let discord_bot_enabled = std::env::var("DISCORD_BOT_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let discord_log_interactions = std::env::var("DISCORD_BOT_LOG_INTERACTIONS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let gateway_url =
            std::env::var("GATEWAY_URL").unwrap_or_else(|_| "http://gateway-svc:8000".to_string());
        let openhack_domain = std::env::var("OPENHACK_DOMAIN")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        if cfg!(not(debug_assertions)) && openhack_domain.contains("localhost") {
            log::warn!("OPENHACK_DOMAIN defaults to localhost — set OPENHACK_DOMAIN for production");
        }

        Ok(Self {
            base,
            lambda_internal_token,
            discord_bot_token,
            discord_application_id,
            discord_guild_id,
            discord_public_key,
            discord_bot_enabled,
            discord_log_interactions,
            gateway_url,
            openhack_domain,
        })
    }

    pub fn database_url(&self) -> &str {
        &self.base.database_url
    }

    pub fn redis_url(&self) -> Option<&str> {
        let url = &self.base.redis_url;
        if url.is_empty() {
            None
        } else {
            Some(url)
        }
    }

    pub fn jwt_secret(&self) -> &str {
        &self.base.jwt_secret
    }

    pub fn port(&self) -> u16 {
        self.base.port
    }
}

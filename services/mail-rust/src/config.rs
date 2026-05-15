use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub port: u16,
    pub rust_log: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub mail_from: String,
    #[allow(dead_code)]
    pub mail_provider: String,
    pub media_service_url: String,
    #[allow(dead_code)]
    pub max_attachment_size: usize,
    #[allow(dead_code)]
    pub app_domain: String,
    #[allow(dead_code)]
    pub hackathon_name: String,
    #[allow(dead_code)]
    pub hackathon_start_date: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL is required"),
            redis_url: env::var("REDIS_URL").ok(),
            port: env::var("SERVICE_PORT")
                .or_else(|_| env::var("PORT"))
                .unwrap_or_else(|_| "3005".into())
                .parse()
                .expect("PORT must be a number"),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "postfix".into()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "25".into())
                .parse()
                .unwrap_or(25),
            smtp_user: env::var("SMTP_USER").unwrap_or_default(),
            smtp_pass: env::var("SMTP_PASS").unwrap_or_default(),
            mail_from: env::var("MAIL_FROM").unwrap_or_else(|_| "noreply@openhack.local".into()),
            mail_provider: env::var("MAIL_PROVIDER").unwrap_or_else(|_| "smtp".into()),
            media_service_url: env::var("MEDIA_SERVICE_URL")
                .unwrap_or_else(|_| "http://media-svc:3010".into()),
            max_attachment_size: env::var("MAX_ATTACHMENT_SIZE")
                .unwrap_or_else(|_| "10485760".into())
                .parse()
                .unwrap_or(10_485_760),
            app_domain: env::var("APP_DOMAIN").unwrap_or_else(|_| "http://localhost:3000".into()),
            hackathon_name: env::var("HACKATHON_NAME").unwrap_or_else(|_| "OpenHack".into()),
            hackathon_start_date: env::var("HACKATHON_START_DATE").unwrap_or_default(),
        }
    }
}

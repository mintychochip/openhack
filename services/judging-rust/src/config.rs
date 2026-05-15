use std::env;

pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub port: u16,
    pub rust_log: String,
    pub _default_judge_ids: String,
    pub _default_rubric_id: String,
    pub _judge_distribution: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL is required"),
            redis_url: env::var("REDIS_URL").ok(),
            port: env::var("SERVICE_PORT")
                .or_else(|_| env::var("PORT"))
                .unwrap_or_else(|_| "3003".into())
                .parse()
                .expect("PORT must be a number"),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            _default_judge_ids: env::var("DEFAULT_JUDGE_IDS").unwrap_or_default(),
            _default_rubric_id: env::var("DEFAULT_RUBRIC_ID").unwrap_or_default(),
            _judge_distribution: env::var("JUDGE_DISTRIBUTION").unwrap_or_else(|_| "random".into()),
        }
    }
}

use std::env;

/// Application configuration loaded from environment variables.
///
/// # Expected Behavior
///
/// Reads `DATABASE_URL`, `REDIS_URL`, `SERVICE_PORT/PORT`, `LLM_PROVIDER`, `OPENAI_API_KEY`,
/// `ANTHROPIC_API_KEY`, `OPENAI_MODEL`, `ANTHROPIC_MODEL`, `EMBEDDINGS_MODEL`, `RAG_TOP_K`,
/// `RAG_SIMILARITY_THRESHOLD`, and FEATURE_* flags from environment variables.
/// Falls back to sensible defaults for optional values. `DATABASE_URL` is required.
/// `REDIS_URL` is optional — if not set or unreachable, Redis caching is disabled.
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
    pub redis_url: Option<String>,
    pub port: u16,
    pub llm_provider: String,
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub openai_model: String,
    pub anthropic_model: String,
    pub embeddings_model: String,
    pub rag_top_k: i64,
    pub rag_similarity_threshold: f64,
    pub feature_chat: bool,
    pub feature_idea_generator: bool,
    pub feature_team_matcher: bool,
    pub feature_code_review: bool,
    pub jwt_secret: String,
    pub cors_allowed_origins: Option<Vec<String>>,
}

impl Config {
    /// Build configuration from environment variables.
    ///
    /// # Expected Behavior
    ///
    /// Reads each required and optional environment variable. `SERVICE_PORT` or PORT
    /// defaults to 3007. `LLM_PROVIDER` defaults to "openai". `OPENAI_MODEL` defaults
    /// to "gpt-4-turbo". `ANTHROPIC_MODEL` defaults to "claude-3-5-sonnet-20241022".
    /// `EMBEDDINGS_MODEL` defaults to "text-embedding-ada-002". `RAG_TOP_K` defaults to 5.
    /// `RAG_SIMILARITY_THRESHOLD` defaults to 0.7. Feature flags default to
    /// chat=true, `idea_generator=true`, `team_matcher=true`, `code_review=false`.
    /// `DATABASE_URL` is required and will cause a panic if missing. `REDIS_URL` is optional;
    /// if not set, Redis caching will be disabled.
    ///
    /// # Errors
    ///
    /// Panics with a descriptive message if `DATABASE_URL` or `REDIS_URL`
    /// environment variables are not set.
    ///
    /// # Side Effects
    ///
    /// - Reads environment variables (read-only).
    pub fn from_env() -> Self {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
        let redis_url = env::var("REDIS_URL").ok();
        let port: u16 = env::var("SERVICE_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "3007".to_string())
            .parse()
            .expect("SERVICE_PORT/PORT must be a valid u16");
        let llm_provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
        let openai_api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
        let anthropic_api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let openai_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4-turbo".to_string());
        let anthropic_model = env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
        let embeddings_model =
            env::var("EMBEDDINGS_MODEL").unwrap_or_else(|_| "text-embedding-ada-002".to_string());
        let rag_top_k: i64 = env::var("RAG_TOP_K")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .expect("RAG_TOP_K must be a valid integer");
        let rag_similarity_threshold: f64 = env::var("RAG_SIMILARITY_THRESHOLD")
            .unwrap_or_else(|_| "0.7".to_string())
            .parse()
            .expect("RAG_SIMILARITY_THRESHOLD must be a valid float");
        let feature_chat = env::var("FEATURE_CHAT")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        let feature_idea_generator = env::var("FEATURE_IDEA_GENERATOR")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        let feature_team_matcher = env::var("FEATURE_TEAM_MATCHER")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        let feature_code_review = env::var("FEATURE_CODE_REVIEW")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            log::warn!("JWT_SECRET not set, using default (INSECURE for production)");
            "change-me-in-production-32ch".to_string()
        });

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

        Self {
            database_url,
            redis_url,
            port,
            llm_provider,
            openai_api_key,
            anthropic_api_key,
            openai_model,
            anthropic_model,
            embeddings_model,
            rag_top_k,
            rag_similarity_threshold,
            feature_chat,
            feature_idea_generator,
            feature_team_matcher,
            feature_code_review,
            jwt_secret,
            cors_allowed_origins,
        }
    }
}

#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::struct_excessive_bools
)]

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod config;
mod errors;
mod middleware;
mod models;
mod routes;
mod services;
mod tests;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer};
use openhack_common::{auth::JwtSecret, db, redis_ext};
use sqlx::postgres::PgPool;

/// Start the Actix-web HTTP server for the AI service.
///
/// # Expected Behavior
///
/// Loads configuration from environment variables, initializes the `PostgreSQL`
/// connection pool, attempts to establish an optional Redis connection, creates a shared reqwest HTTP client,
/// runs database health checks, and starts the Actix-web server on the
/// configured port (default 3007). Registers all API routes under `/api/ai`,
/// a health check at `/health`, and a service info endpoint at `/`.
/// CORS is configured to allow any origin, method, and header.
/// If `REDIS_URL` is not set or the Redis connection fails, caching is disabled
/// and the service continues without Redis. Graceful shutdown is handled by Actix-web's built-in signal handling.
///
/// # Errors
///
/// Panics if configuration is missing (see `Config::from_env`).
/// Returns an error if the HTTP server fails to bind to the configured port
/// or if the database connections cannot be established at startup.
///
/// # Side Effects
///
/// - Reads environment variables via `Config::from_env`.
/// - Initializes `dotenvy` for .env file loading.
/// - Creates a `PostgreSQL` connection pool (network connections).
/// - Attempts to create a Redis client and connection (network I/O);
///   logs a warning and continues if unavailable.
/// - Runs database health check (SQL query).
/// - Binds and listens on the configured TCP port.
/// - Logs at INFO on startup with port number.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let mut config = config::Config::from_env();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    log::info!("Starting AI service on port {}", config.port);
    log::info!("LLM provider: {}", config.llm_provider);
    log::info!(
        "Features - chat: {}, ideas: {}, team_matcher: {}, code_review: {}, brand_extraction: {}",
        config.feature_chat,
        config.feature_idea_generator,
        config.feature_team_matcher,
        config.feature_code_review,
        config.feature_brand_extraction
    );

    let pool = db::create_pool(&config.database_url, 5)
        .await
        .expect("Failed to create database pool");

    db::health_check(&pool)
        .await
        .expect("Database health check failed");

    // Merge dashboard settings from DB (env vars take precedence)
    if let Err(e) = config.merge_from_db(&pool).await {
        log::warn!("Failed to load AI config from database, using env defaults: {}", e);
    } else {
        log::info!("AI config merged from database (env vars still take precedence)");
    }

    let redis_url = config.redis_url.as_deref().unwrap_or("");
    let redis_conn_data = redis_ext::create_connection(redis_url).await;

    let http_client = reqwest::Client::new();

    let port = config.port;
    let pool_data = web::Data::new(pool);
    let config_data = web::Data::new(config);
    let http_data = web::Data::new(http_client);

    HttpServer::new(move || {
        let cors_origins: Vec<String> = config_data
            .cors_allowed_origins
            .clone()
            .unwrap_or_else(|| vec!["*".to_string()]);

        let cors = if cors_origins.iter().any(|o| o == "*") {
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600)
        } else {
            let mut cors_builder = Cors::default()
                .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
                .allowed_headers(vec!["Content-Type", "Accept", "Authorization"])
                .max_age(3600);

            for origin in &cors_origins {
                cors_builder = cors_builder.allowed_origin(origin);
            }
            cors_builder
        };

        App::new()
            .wrap(cors)
            .wrap(openhack_common::security_headers::SecurityHeadersMiddleware::new())
            .wrap(openhack_common::metrics::MetricsMiddleware::new("ai"))
            .wrap(actix_web::middleware::Logger::default())
            .app_data(pool_data.clone())
            .app_data(redis_conn_data.clone())
            .app_data(config_data.clone())
            .app_data(http_data.clone())
            .app_data(web::Data::new(JwtSecret(config_data.jwt_secret.clone())))
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/", web::get().to(service_info))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

/// Health check endpoint.
///
/// # Expected Behavior
///
/// Returns a 200 OK with a JSON body indicating the service is healthy,
/// the database connectivity status, and the LLM provider name.
/// Checks database connectivity with a simple query.
///
/// # Errors
///
/// None. Always returns 200 OK (database status reflected in the body).
///
/// # Side Effects
///
/// - May execute a database health check query (database I/O).
async fn health_check(pool: web::Data<PgPool>, config: web::Data<config::Config>) -> HttpResponse {
    let db_status = match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "database": db_status,
        "llm_provider": config.llm_provider,
    }))
}

/// Service info endpoint.
///
/// # Expected Behavior
///
/// Returns a 200 OK with a JSON body containing the service name, version,
/// and a list of enabled features.
///
/// # Errors
///
/// None. Always returns 200 OK.
///
/// # Side Effects
///
/// None. Pure response generation with no I/O.
async fn service_info(config: web::Data<config::Config>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "ai-service",
        "version": "0.1.0",
        "features": {
            "chat": config.feature_chat,
            "idea_generator": config.feature_idea_generator,
            "team_matcher": config.feature_team_matcher,
            "code_review": config.feature_code_review,
        }
    }))
}

async fn metrics_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(openhack_common::metrics::render_metrics())
}

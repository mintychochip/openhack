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
mod models;
mod routes;
mod services;
mod tests;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use openhack_common::{db, redis_ext};
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

    let config = config::Config::from_env();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    log::info!("Starting AI service on port {}", config.port);
    log::info!("LLM provider: {}", config.llm_provider);
    log::info!(
        "Features - chat: {}, ideas: {}, team_matcher: {}, code_review: {}",
        config.feature_chat,
        config.feature_idea_generator,
        config.feature_team_matcher,
        config.feature_code_review
    );

    let pool = db::create_pool(&config.database_url, 5)
        .await
        .expect("Failed to create database pool");

    db::health_check(&pool)
        .await
        .expect("Database health check failed");

    let redis_url = config.redis_url.as_deref().unwrap_or("");
    let redis_conn_data = redis_ext::create_connection(redis_url).await;

    let http_client = reqwest::Client::new();

    let port = config.port;
    let pool_data = web::Data::new(pool);
    let config_data = web::Data::new(config);
    let http_data = web::Data::new(http_client);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(pool_data.clone())
            .app_data(redis_conn_data.clone())
            .app_data(config_data.clone())
            .app_data(http_data.clone())
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/", web::get().to(service_info))
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

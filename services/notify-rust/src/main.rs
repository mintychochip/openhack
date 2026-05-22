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
mod events;
mod middleware;
mod models;
mod routes;
mod services;
mod tests;

use actix_cors::Cors;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use openhack_common::{auth::JwtSecret, db};
use std::sync::Arc;

/// Health check endpoint (liveness probe).
///
/// # Expected Behavior
///
/// Returns a 200 OK with a JSON body indicating the service is healthy.
/// Checks database connectivity. For readiness checks, use /ready.
///
/// # Errors
///
/// Returns 503 Service Unavailable if the database is unreachable.
///
/// # Side Effects
///
/// - Executes `SELECT 1` on the database pool.
async fn health_check(pool: web::Data<sqlx::postgres::PgPool>) -> HttpResponse {
    match db::health_check(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "service": "notify"
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "database": e.to_string()
        })),
    }
}

/// Readiness check endpoint.
///
/// # Expected Behavior
///
/// Returns 200 OK if the service is ready to accept requests (database
/// connection is verified). Returns 503 if the database is not reachable.
///
/// # Errors
///
/// Returns 503 Service Unavailable if the database health check fails.
///
/// # Side Effects
///
/// - Executes `SELECT 1` on the database pool.
async fn ready_check(pool: web::Data<sqlx::postgres::PgPool>) -> HttpResponse {
    match db::health_check(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": "notify"
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "database": e.to_string()
        })),
    }
}

/// Start the Actix-web HTTP server.
///
/// # Expected Behavior
///
/// Loads configuration from environment variables, initializes the `PostgreSQL`
/// connection pool and optional Redis client, runs database health checks, spawns
/// the Redis event subscriber as a background task (if Redis is configured), starts
/// a periodic webhook retry task, and starts the Actix-web server on the configured
/// port (default 3006). Graceful shutdown is handled by Actix-web's built-in
/// signal handling. If `REDIS_URL` is not set or Redis is unreachable, the event
/// subscriber is skipped and the service starts normally.
///
/// # Errors
///
/// Panics if configuration is missing (see `Config::from_env`).
/// Returns an error if the HTTP server fails to bind to the configured port
/// or if the database connection cannot be established at startup.
/// Redis unavailability does not prevent startup.
///
/// # Side Effects
///
/// - Reads environment variables via `Config::from_env`.
/// - Initializes `dotenvy` for .env file loading.
/// - Creates a `PostgreSQL` connection pool (network connections).
/// - Creates a Redis client (lazy connection, optional — skipped if
///   `REDIS_URL` is not set).
/// - Runs database health check (SQL query).
/// - Spawns a Tokio task for Redis pub/sub event subscription (if Redis
///   is available).
/// - Spawns a Tokio task for periodic webhook retry processing.
/// - Binds and listens on the configured TCP port.
/// - Logs at INFO on startup with port number.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env();
    std::env::set_var("RUST_LOG", &cfg.rust_log);
    env_logger::init();

    log::info!("Starting notify service on port {}", cfg.port);

    let pool = db::create_pool(&cfg.database_url, 5)
        .await
        .expect("Failed to create database pool");

    db::health_check(&pool)
        .await
        .expect("Database health check failed");

    log::info!("Database connection established");

    if let Some(ref redis_url) = cfg.redis_url {
        match redis::Client::open(redis_url.clone()) {
            Ok(redis_client) => match redis_client.get_multiplexed_async_connection().await {
                Ok(_) => {
                    log::info!("Redis connection verified");
                    let pool_arc = Arc::new(pool.clone());
                    let redis_arc = Arc::new(redis_client);
                    let event_pool = pool_arc.clone();
                    let event_redis = redis_arc.clone();
                    let event_discord_url = cfg.discord_webhook_url.clone();
                    tokio::spawn(async move {
                        let _ = events::subscriber::subscribe_to_events(
                            event_pool,
                            Some(event_redis),
                            event_discord_url,
                        )
                        .await;
                    });
                }
                Err(e) => log::warn!(
                    "Redis connection test failed: {e}. Event subscriber will not start."
                ),
            },
            Err(e) => {
                log::warn!("Failed to create Redis client: {e}. Event subscriber will not start.")
            }
        }
    } else {
        log::warn!("REDIS_URL not set. Event subscriber will not start.");
    }

    let retry_pool = Arc::new(pool.clone());
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            services::webhook::retry_pending_deliveries(retry_pool.as_ref()).await;
        }
    });

    let pool_data = web::Data::new(pool);
    let jwt_secret_data = web::Data::new(JwtSecret(cfg.jwt_secret.clone()));
    let config_data = web::Data::new(cfg.clone());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(openhack_common::metrics::MetricsMiddleware::new("notify"))
            .wrap(actix_middleware::Logger::default())
            .app_data(pool_data.clone())
            .app_data(jwt_secret_data.clone())
            .app_data(config_data.clone())
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(ready_check))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("0.0.0.0", cfg.port))?
    .run()
    .await
}

async fn metrics_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(openhack_common::metrics::render_metrics())
}

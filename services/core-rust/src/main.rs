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
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use openhack_common::{auth::JwtSecret, db, redis_ext};

use config::Config;

#[derive(Debug, Clone)]
struct MediaServiceUrl(String);

/// Start the Actix-web HTTP server for the core service.
///
/// # Expected Behavior
///
/// Loads configuration from environment variables, initializes the `PostgreSQL`
/// connection pool and optionally the Redis client, runs database health checks, spawns the
/// phase scheduler background task, and starts the Actix-web server on the
/// configured port (default 3002). If `REDIS_URL` is not set or the Redis connection fails,
/// the service starts without Redis — event publishing is silently skipped.
/// Graceful shutdown is handled by Actix-web's built-in signal handling. The JWT secret
/// and media service URL are shared via app data for auth middleware and demo upload use.
///
/// # Errors
///
/// Panics if configuration is missing (see `Config::from_env`).
/// Returns an error if the HTTP server fails to bind to the configured port.
/// If the database connection cannot be established at startup, panics.
/// Redis connection failure is non-fatal — the service starts without Redis.
///
/// # Side Effects
///
/// - Reads environment variables via `Config::from_env`.
/// - Initializes `dotenvy` for .env file loading.
/// - Creates a `PostgreSQL` connection pool (network connections).
/// - Optionally creates a Redis client and connection (network connection).
///   If Redis is unavailable, the service starts without it.
/// - Runs database health check (SQL query).
/// - Spawns a background tokio task for phase scheduling (periodic DB reads/writes).
/// - Binds and listens on the configured TCP port.
/// - Logs at INFO on startup with port number.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    std::env::set_var("RUST_LOG", &config.rust_log);
    env_logger::init();

    log::info!("Starting core service on port {}", config.port);

    let pool = db::create_pool(&config.database_url, 5)
        .await
        .expect("Failed to create database pool");

    db::health_check(&pool)
        .await
        .expect("Database health check failed");

    let redis_conn_data = redis_ext::create_connection(&config.redis_url).await;

    let jwt_secret = config.jwt_secret.clone();
    let media_url = config.media_service_url.clone();
    let pool_clone = pool.clone();

    let pool_data = web::Data::new(pool);
    let jwt_data = web::Data::new(JwtSecret(jwt_secret));
    let media_data = web::Data::new(MediaServiceUrl(media_url));

    tokio::spawn(async move {
        let scheduler_pool = pool_clone;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) =
                services::phase::PhaseService::check_and_transition(&scheduler_pool).await
            {
                log::warn!("Phase scheduler error: {e}");
            }
        }
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(actix_middleware::Logger::default())
            .app_data(pool_data.clone())
            .app_data(redis_conn_data.clone())
            .app_data(jwt_data.clone())
            .app_data(media_data.clone())
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

/// Liveness health check endpoint.
///
/// # Expected Behavior
///
/// Returns a 200 OK with a JSON body indicating the service is healthy.
/// Does not check database or Redis connectivity. Use for liveness probes.
///
/// # Errors
///
/// None. Always returns 200 OK.
///
/// # Side Effects
///
/// None. Pure response generation with no I/O.
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "core"
    }))
}

/// Readiness check endpoint.
///
/// # Expected Behavior
///
/// Checks database connectivity by executing `SELECT 1`. Returns 200 OK
/// with JSON indicating readiness if the database is reachable. Returns
/// 503 Service Unavailable if the database query fails.
///
/// # Errors
///
/// None. Returns appropriate HTTP status codes for both healthy and
/// unhealthy states without propagating errors.
///
/// # Side Effects
///
/// - Executes a `SELECT 1` query against the `PostgreSQL` pool (read).
/// - Logs at WARN level if the database check fails.
async fn readiness_check(pool: web::Data<sqlx::postgres::PgPool>) -> HttpResponse {
    match db::health_check(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": "core"
        })),
        Err(e) => {
            log::warn!("Readiness check failed: {e}");
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "not_ready",
                "error": e.to_string()
            }))
        }
    }
}

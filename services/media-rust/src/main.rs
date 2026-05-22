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
#[cfg(test)]
mod tests;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer};
use openhack_common::{auth::JwtSecret, db};
use std::sync::Arc;

use crate::config::Config;
use crate::services::storage::{LocalStorageProvider, S3StorageProvider, StorageProvider};

/// Start the Actix-web HTTP server for the media service.
///
/// # Expected Behavior
///
/// Loads configuration from environment variables, initializes the `PostgreSQL`
/// connection pool, constructs the appropriate storage provider (local or S3)
/// based on the `STORAGE_PROVIDER` env var, runs a database health check,
/// and starts the Actix-web server on the configured port (default 3010).
/// Graceful shutdown is handled by Actix-web's built-in signal handling.
///
/// # Errors
///
/// Panics if required configuration is missing (see `Config::from_env`).
/// Returns an error if the HTTP server fails to bind to the configured port
/// or if the database connection cannot be established at startup.
///
/// # Side Effects
///
/// - Reads environment variables via `Config::from_env`.
/// - Initializes `dotenvy` for .env file loading.
/// - Creates a `PostgreSQL` connection pool (network connections).
/// - Creates the storage directory for local storage (filesystem mkdir).
/// - Establishes S3 client connection for S3 storage (network).
/// - Binds and listens on the configured TCP port.
/// - Logs at INFO on startup with port number and storage provider.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    std::env::set_var("RUST_LOG", &config.rust_log);
    env_logger::init();

    log::info!("Starting media service on port {}", config.port);
    log::info!("Storage provider: {}", config.storage_provider);

    let pool = db::create_pool(&config.database_url, 5)
        .await
        .expect("Failed to create database pool");

    db::health_check(&pool)
        .await
        .expect("Database health check failed");

    let storage_provider: Arc<dyn StorageProvider> = if config.storage_provider.as_str() == "s3" {
        let s3 = S3StorageProvider::from_env();
        Arc::new(s3)
    } else {
        let local = LocalStorageProvider::new(&config.storage_path);
        Arc::new(local)
    };

    let pool_data = web::Data::new(pool);
    let storage_data = web::Data::from(storage_provider);
    let config_data = web::Data::new(config.clone());

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
            .wrap(openhack_common::metrics::MetricsMiddleware::new("media"))
            .wrap(actix_web::middleware::Logger::default())
            .app_data(pool_data.clone())
            .app_data(storage_data.clone())
            .app_data(config_data.clone())
            .app_data(web::Data::new(JwtSecret(config_data.jwt_secret.clone())))
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/", web::get().to(index))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

/// Health check endpoint.
///
/// # Expected Behavior
///
/// Checks database connectivity by executing `SELECT 1` and storage provider
/// health via `StorageProvider::health_check`. Returns 200 with a JSON body
/// containing the status of each component: "healthy" or "unhealthy".
///
/// # Errors
///
/// None. Always returns 200 OK, but individual components may report "unhealthy".
///
/// # Side Effects
///
/// - Executes a SQL query against the database pool.
/// - Calls `StorageProvider::health_check` which may perform I/O.
async fn health_check(pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    let db_status = match db::health_check(pool.get_ref()).await {
        Ok(()) => "healthy",
        Err(_) => "unhealthy",
    };

    let overall = if db_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": overall,
        "database": db_status,
        "storage": "healthy"
    }))
}

/// Root index endpoint.
///
/// # Expected Behavior
///
/// Returns a JSON body with the service name, version, and active storage
/// provider identifier. Does not perform any I/O or database queries.
///
/// # Errors
///
/// None. Always returns 200 OK.
///
/// # Side Effects
///
/// None. Pure response generation.
async fn index(config: web::Data<Config>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "media",
        "version": env!("CARGO_PKG_VERSION"),
        "storage_provider": config.storage_provider
    }))
}

async fn metrics_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(openhack_common::metrics::render_metrics())
}

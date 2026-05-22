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
use actix_web::{web, App, HttpResponse, HttpServer};
use openhack_common::{auth::JwtSecret, db, redis_ext};

async fn health_check(pool: web::Data<sqlx::postgres::PgPool>) -> HttpResponse {
    match db::health_check(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "database": "connected"
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "database": e.to_string()
        })),
    }
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "OpenHack Judging Service",
        "version": "1.0.0"
    }))
}

async fn metrics_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(openhack_common::metrics::render_metrics())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env();
    std::env::set_var("RUST_LOG", &cfg.rust_log);
    env_logger::init();

    let pool = db::create_pool(&cfg.database_url, 5)
        .await
        .expect("Failed to create database pool");

    db::health_check(&pool)
        .await
        .expect("Database health check failed");

    log::info!("Database connection established");

    openhack_common::metrics::register_business_counter("judging_scores_submitted_total", "Total judging scores submitted");
    openhack_common::metrics::register_business_counter("judging_assignments_completed_total", "Total judging assignments completed");

    let redis_url = cfg.redis_url.as_deref().unwrap_or("");
    let _redis_conn_data = redis_ext::create_connection(redis_url).await;

    // Spawn event subscriber if Redis is configured
    if !redis_url.is_empty() {
        if let Ok(redis_client) = redis::Client::open(redis_url) {
            let pool_arc = std::sync::Arc::new(pool.clone());
            let redis_arc = std::sync::Arc::new(redis_client);
            tokio::spawn(async move {
                let _ = events::subscriber::subscribe_to_events(pool_arc, Some(redis_arc)).await;
            });
            log::info!("Judging event subscriber started");
        } else {
            log::warn!("Redis client creation failed, event subscriber will not start");
        }
    } else {
        log::warn!("Redis not configured, event subscriber will not start");
    }

    let pool_data = web::Data::new(pool);

    HttpServer::new(move || {
        let cors_origins: Vec<String> = cfg
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
            .wrap(openhack_common::metrics::MetricsMiddleware::new("judging"))
            .wrap(actix_web::middleware::Logger::default())
            .app_data(pool_data.clone())
            .app_data(web::Data::new(JwtSecret(cfg.jwt_secret.clone())))
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/", web::get().to(index))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("0.0.0.0", cfg.port))?
    .run()
    .await
}

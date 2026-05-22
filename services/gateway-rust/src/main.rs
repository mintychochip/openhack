#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::struct_excessive_bools,
    clippy::too_many_lines
)]

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod rate_limiter;
mod routes;
#[cfg(test)]
mod tests;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub redis_url: Option<String>,
    pub log_level: String,
    pub cors_origin: String,
    pub sse_enabled: bool,
}

impl Config {
    #[allow(clippy::missing_panics_doc)]
    pub fn from_env() -> Self {
        let port: u16 = env::var("PORT")
            .or_else(|_| env::var("SERVICE_PORT"))
            .unwrap_or_else(|_| "8000".to_string())
            .parse()
            .expect("PORT must be a valid u16");
        let redis_url = env::var("REDIS_URL").ok();
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());
        let sse_enabled = env::var("SSE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Self {
            port,
            redis_url,
            log_level,
            cors_origin,
            sse_enabled,
        }
    }
}

async fn readiness_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ready",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    std::env::set_var("RUST_LOG", &config.log_level);
    env_logger::init();

    log::info!(
        "Starting gateway service (api gateway + sse) on port {}",
        config.port
    );

    if config.redis_url.is_none() {
        log::warn!("REDIS_URL not set. SSE streams will not receive Redis events.");
    }

    let event_bus = actix_web::web::Data::new(routes::sse::EventBus::new(
        config.redis_url.clone(),
        config.sse_enabled,
    ));

    let redis_url_for_limiter = config.redis_url.clone();
    let rate_limiter = rate_limiter::RedisRateLimiter::new_with_url(redis_url_for_limiter);
    let rate_limiter_data = web::Data::new(rate_limiter);
    let cors_origin = config.cors_origin.clone();

    HttpServer::new(move || {
        let cors = if cors_origin == "*" {
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
        } else {
            Cors::default()
                .allowed_origin(&cors_origin)
                .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
                .allowed_headers(vec!["Content-Type", "Accept", "Authorization"])
        };

        App::new()
            .wrap(openhack_common::metrics::MetricsMiddleware::new("gateway"))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(event_bus.clone())
            .app_data(rate_limiter_data.clone())
            .route("/ready", web::get().to(readiness_check))
            .route("/metrics", web::get().to(metrics_handler))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

async fn metrics_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(openhack_common::metrics::render_metrics())
}

#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::struct_excessive_bools,
    clippy::unused_self,
    clippy::new_without_default
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
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use openhack_common::{auth::JwtSecret, db, redis_ext};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::Config;

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            crate::models::user::UserProfile,
            crate::models::user::RegisterRequest,
            crate::models::user::RegisterResponse,
            crate::services::gdpr::GdprExport,
        )
    ),
    tags(
        (name = "auth", description = "Authentication and user management"),
        (name = "users", description = "User operations"),
        (name = "gdpr", description = "GDPR compliance endpoints"),
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    std::env::set_var("RUST_LOG", &config.rust_log);
    env_logger::init();

    log::info!("Starting auth service on port {}", config.port);

    let pool = db::create_pool(&config.database_url, 5)
        .await
        .expect("Failed to create database pool");

    db::health_check(&pool)
        .await
        .expect("Database health check failed");

    let redis_conn_data = redis_ext::create_connection(&config.redis_url).await;

    let pool_data = web::Data::new(pool);
    let config_data = web::Data::new(config.clone());
    let jwt_secret_data = web::Data::new(JwtSecret(config.jwt_secret.clone()));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(openhack_common::metrics::MetricsMiddleware::new("auth"))
            .wrap(Logger::default())
            .app_data(pool_data.clone())
            .app_data(redis_conn_data.clone())
            .app_data(config_data.clone())
            .app_data(jwt_secret_data.clone())
            .configure(routes::configure)
            .route("/health", web::get().to(health_handler))
            .route("/ready", web::get().to(readiness_handler))
            .route("/metrics", web::get().to(metrics_handler))
            .service(SwaggerUi::new("/api/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "auth"
    }))
}

async fn readiness_handler(pool: web::Data<sqlx::postgres::PgPool>) -> HttpResponse {
    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": "auth"
        })),
        Err(e) => {
            log::error!("Readiness check failed: {e}");
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "not_ready",
                "service": "auth",
                "error": "Database unavailable"
            }))
        }
    }
}

async fn metrics_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(openhack_common::metrics::render_metrics())
}

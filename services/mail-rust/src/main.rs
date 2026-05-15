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
mod models;
mod routes;
mod services;
mod tests;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use openhack_common::{db, redis_ext};
use services::smtp::SmtpProvider;

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
        "service": "OpenHack Mail Service",
        "version": "1.0.0"
    }))
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

    let redis_url = cfg.redis_url.as_deref().unwrap_or("");
    let _redis_conn_data = redis_ext::create_connection(redis_url).await;

    let smtp = SmtpProvider::new(
        &cfg.smtp_host,
        cfg.smtp_port,
        &cfg.smtp_user,
        &cfg.smtp_pass,
        &cfg.mail_from,
    );

    let pool_data = web::Data::new(pool);
    let smtp_data = web::Data::new(smtp);
    let config_data = web::Data::new(cfg.clone());

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
            .app_data(smtp_data.clone())
            .app_data(config_data.clone())
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/", web::get().to(index))
    })
    .bind(("0.0.0.0", cfg.port))?
    .run()
    .await
}

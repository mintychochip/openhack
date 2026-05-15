#![deny(clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::panic_in_result_fn,
    clippy::unimplemented,
    clippy::todo,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::ok_expect
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::unused_async,
    clippy::map_unwrap_or,
    clippy::redundant_closure_for_method_calls,
    clippy::single_match_else,
    clippy::match_same_arms,
    clippy::manual_let_else,
    clippy::needless_pass_by_value,
    clippy::unnested_or_patterns,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

mod auth;
mod commands;
mod components;
mod config;
mod discord_api;
mod errors;
mod events;
mod interaction;
mod middleware;
mod models;
mod openhack_client;
mod routes;

#[cfg(test)]
mod tests;

use actix_cors::Cors;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use openhack_common::auth::JwtSecret;
use openhack_common::db;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct BotState {
    connected: Arc<std::sync::atomic::AtomicBool>,
    guild_id: String,
}

/// Health check endpoint (liveness probe).
///
/// # Expected Behavior
///
/// Returns a 200 OK with a JSON body indicating the service is healthy.
/// Checks database connectivity. Includes bot connection status.
///
/// # Errors
///
/// Returns 503 Service Unavailable if the database is unreachable.
///
/// # Side Effects
///
/// - Executes `SELECT 1` on the database pool.
async fn health_check(pool: web::Data<PgPool>, state: web::Data<BotState>) -> HttpResponse {
    match db::health_check(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "service": "discord-bot",
            "bot_connected": state.connected.load(std::sync::atomic::Ordering::Relaxed)
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "database": e.to_string(),
            "bot_connected": state.connected.load(std::sync::atomic::Ordering::Relaxed)
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
async fn ready_check(pool: web::Data<PgPool>) -> HttpResponse {
    match db::health_check(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": "discord-bot"
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "database": e.to_string()
        })),
    }
}

/// Start the Actix-web HTTP server with Discord interactions endpoint and
/// Redis event subscriber.
///
/// # Expected Behavior
///
/// Loads configuration from environment variables, initializes the PostgreSQL
/// connection pool via `openhack_common::db::create_pool`, runs database health
/// checks, and then:
///
/// 1. If `DISCORD_BOT_ENABLED` is true and `DISCORD_BOT_TOKEN` is set, spawns
///    a task to register slash commands with Discord via REST API. The bot
///    `connected` flag is set to true *inside* the spawned task *after*
///    registration succeeds.
/// 2. If `REDIS_URL` is set, spawns the Redis event subscriber as a background task.
/// 3. Spawns the HTTP server on the configured port (default 3011), which
///    exposes the `/interactions` POST endpoint that Discord calls for
///    interaction events.
///
/// If Discord or Redis is unavailable, the service starts normally with those
/// features disabled. Graceful shutdown is handled by Actix-web's built-in
/// signal handling.
///
/// # Errors
///
/// Returns an IO error if configuration is missing or invalid, if the
/// database connection pool cannot be created, if the database health
/// check fails, or if the HTTP server fails to bind to the configured port.
///
/// # Side Effects
///
/// - Reads environment variables via `Config::from_env`.
/// - Initializes `dotenvy` for .env file loading.
/// - Creates a PostgreSQL connection pool (network connections).
/// - Creates a Redis client (lazy connection, optional).
/// - Runs database health check (SQL query).
/// - Spawns Tokio task for slash command registration (if configured).
/// - Spawns Tokio tasks for Redis event subscriber (if configured).
/// - Binds and listens on the configured TCP port.
/// - Logs at INFO on startup with port number.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    env_logger::init();

    log::info!("Starting discord-bot service on port {}", cfg.port());

    let pool: PgPool = db::create_pool(cfg.database_url(), 20)
        .await
        .map_err(std::io::Error::other)?;

    db::health_check(&pool)
        .await
        .map_err(std::io::Error::other)?;

    log::info!("Database connection established");

    let bot_state = BotState {
        connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        guild_id: cfg.discord_guild_id.clone(),
    };

    let openhack_client =
        openhack_client::OpenHackClient::new(&cfg.gateway_url, &cfg.lambda_internal_token);

    if cfg.discord_bot_enabled && !cfg.discord_bot_token.is_empty() {
        let register_cfg = cfg.clone();
        let connected_flag = bot_state.connected.clone();
        tokio::spawn(async move {
            if let Err(e) = discord_api::register_commands(&register_cfg).await {
                log::error!("Failed to register slash commands: {e}");
            } else {
                connected_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                log::info!("Discord bot connected and commands registered");
            }
        });
    } else {
        log::warn!("Discord bot disabled or DISCORD_BOT_TOKEN not set. Bot will not start.");
    }

    if let Some(redis_url) = cfg.redis_url() {
        match redis::Client::open(redis_url) {
            Ok(redis_client) => match redis_client.get_multiplexed_async_connection().await {
                Ok(_) => {
                    log::info!("Redis connection verified");
                    let event_pool = Arc::new(pool.clone());
                    let event_redis = Arc::new(redis_client);
                    let event_cfg = cfg.clone();
                    let event_client = openhack_client.clone();
                    tokio::spawn(async move {
                        let _ = crate::events::subscriber::subscribe_to_events(
                            event_pool,
                            Some(event_redis),
                            event_cfg,
                            event_client,
                        )
                        .await;
                    });
                }
                Err(e) => {
                    log::warn!(
                        "Redis connection test failed: {e}. Event subscriber will not start."
                    );
                }
            },
            Err(e) => {
                log::warn!("Failed to create Redis client: {e}. Event subscriber will not start.");
            }
        }
    } else {
        log::warn!("REDIS_URL not set. Event subscriber will not start.");
    }

    let pool_data = web::Data::new(pool);
    let jwt_secret_data = web::Data::new(JwtSecret(cfg.jwt_secret().to_string()));
    let config_data = web::Data::new(cfg.clone());
    let bot_state_data = web::Data::new(bot_state);
    let client_data = web::Data::new(openhack_client);

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
            .app_data(jwt_secret_data.clone())
            .app_data(config_data.clone())
            .app_data(bot_state_data.clone())
            .app_data(client_data.clone())
            .configure(routes::configure)
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(ready_check))
    })
    .bind(("0.0.0.0", cfg.port()))?
    .run()
    .await
}

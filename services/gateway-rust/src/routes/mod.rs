pub mod proxy;
pub mod sse;

use actix_web::web;

use crate::routes::proxy::ProxyState;

/// Register all gateway API routes on the given service config.
///
/// # Expected Behavior
///
/// Configures the following endpoints:
/// - GET `/api/events/stream` → SSE real-time event stream (501 if SSE disabled)
/// - GET `/api/events/history` → stub returning empty event history
/// - GET `/api/events/poll` → polling endpoint for serverless mode (reads Redis sorted set)
/// - ALL `/api/auth/{path}` → proxy to auth-svc:3001
/// - ALL `/api/core/{path}` → proxy to core-svc:3002
/// - ALL `/api/judging/{path}` → proxy to judging-svc:3003
/// - ALL `/api/leaderboard/{path}` → proxy to leaderboard-svc:3004
/// - ALL `/api/mail/{path}` → proxy to mail-svc:3005
/// - ALL `/api/notify/{path}` → proxy to notify-svc:3006
/// - ALL `/api/ai/{path}` → proxy to ai-svc:3007
/// - ALL `/api/analytics/{path}` → proxy to analytics-svc:3008
/// - ALL `/api/sponsors/{path}` → proxy to sponsors-svc:3009
/// - ALL `/api/media/{path}` → proxy to media-svc:3010
/// - ALL `/api/discord-bot/{path}` → proxy to discord-bot-svc:3011
/// - GET `/health` → proxied health check (auth-svc /health)
///
/// The proxy state (route table, HTTP client, rate limiter) is shared
/// across all worker threads via `web::Data`.
///
/// # Errors
///
/// None. Route registration is infallible.
///
/// # Side Effects
///
/// - Registers HTTP route handlers with the Actix-web service config.
/// - Creates and shares `ProxyState` (heap allocation, `awc::Client` pool).
/// - Spawns a background task for rate limiter cleanup.
pub fn configure(cfg: &mut web::ServiceConfig) {
    let proxy_state = ProxyState::new();
    proxy::spawn_limiter_cleanup(&proxy_state.limiter);
    let proxy_data = web::Data::new(proxy_state);

    cfg.app_data(proxy_data)
        .service(
            web::scope("/api/events")
                .route("/stream", web::get().to(sse::sse_stream))
                .route("/history", web::get().to(sse::event_history))
                .route("/poll", web::get().to(sse::event_poll)),
        )
        .service(web::scope("/api/auth").default_service(web::to(proxy::proxy_auth)))
        .service(web::scope("/api/core").default_service(web::to(proxy::proxy_core)))
        .service(web::scope("/api/judging").default_service(web::to(proxy::proxy_judging)))
        .service(web::scope("/api/leaderboard").default_service(web::to(proxy::proxy_leaderboard)))
        .service(web::scope("/api/mail").default_service(web::to(proxy::proxy_mail)))
        .service(web::scope("/api/notify").default_service(web::to(proxy::proxy_notify)))
        .service(web::scope("/api/ai").default_service(web::to(proxy::proxy_ai)))
        .service(web::scope("/api/analytics").default_service(web::to(proxy::proxy_analytics)))
        .service(web::scope("/api/sponsors").default_service(web::to(proxy::proxy_sponsors)))
        .service(web::scope("/api/media").default_service(web::to(proxy::proxy_media)))
        .service(web::scope("/api/discord-bot").default_service(web::to(proxy::proxy_discord_bot)))
        .route("/health", web::get().to(proxy::proxy_health));
}

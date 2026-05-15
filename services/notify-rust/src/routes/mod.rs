pub mod admin;
pub mod announcements;
pub mod discord;
pub mod lambda;
pub mod notifications;
pub mod slack;
pub mod webhooks;

use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::middleware::auth;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(auth::validator);
    let admin_auth = HttpAuthentication::bearer(auth::admin_validator);

    cfg.service(
        web::scope("/api/notify")
            .wrap(auth)
            .route(
                "/announce",
                web::post().to(announcements::create_announcement),
            )
            .route(
                "/announcements",
                web::get().to(announcements::list_announcements),
            )
            .route(
                "/announcements/{id}",
                web::get().to(announcements::get_announcement),
            )
            .route(
                "/announcements/{id}",
                web::delete().to(announcements::delete_announcement),
            )
            .route(
                "/discord/webhook",
                web::post().to(discord::send_discord_webhook),
            )
            .route(
                "/discord/role-assign",
                web::post().to(discord::assign_discord_role),
            )
            .route("/slack/webhook", web::post().to(slack::send_slack_webhook))
            .route(
                "/slack/channel-message",
                web::post().to(slack::send_slack_channel_message),
            )
            .route("/webhooks", web::post().to(webhooks::create_webhook))
            .route("/webhooks", web::get().to(webhooks::list_webhooks))
            .route("/webhooks/{id}", web::put().to(webhooks::update_webhook))
            .route("/webhooks/{id}", web::delete().to(webhooks::delete_webhook))
            .route(
                "/webhooks/{id}/test",
                web::post().to(webhooks::test_webhook),
            )
            .route(
                "/webhooks/{id}/logs",
                web::get().to(webhooks::get_webhook_logs),
            )
            .route(
                "/notifications",
                web::get().to(notifications::get_notifications),
            )
            .route(
                "/notifications/{id}/read",
                web::post().to(notifications::mark_notification_read),
            )
            .route(
                "/notifications/read-all",
                web::post().to(notifications::mark_all_read),
            )
            .route(
                "/notifications/unread-count",
                web::get().to(notifications::get_unread_count),
            ),
    )
    .service(
        web::scope("/api/notify/admin")
            .wrap(admin_auth)
            .route("/process-event", web::post().to(admin::process_event))
            .route("/retry-webhooks", web::post().to(admin::retry_webhooks)),
    )
    .route("/lambda/event", web::post().to(lambda::dispatch));
}

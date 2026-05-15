pub mod admin;
pub mod health;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/interactions")
            .route(web::post().to(crate::interaction::handle_interaction_endpoint)),
    );

    cfg.service(
        web::scope("/api/discord-bot")
            .route(
                "/link/initiate",
                web::post().to(crate::auth::routes::initiate_link),
            )
            .route(
                "/link/confirm",
                web::post().to(crate::auth::routes::confirm_link),
            )
            .route(
                "/link/status",
                web::get().to(crate::auth::routes::get_link_status),
            )
            .route("/health", web::get().to(health::bot_health))
            .route("/status", web::get().to(health::bot_status)),
    )
    .service(
        web::scope("/api/discord-bot/admin")
            .route("/channels", web::get().to(admin::list_channels))
            .route("/channels", web::post().to(admin::create_channel))
            .route("/channels/{id}", web::put().to(admin::update_channel))
            .route("/channels/{id}", web::delete().to(admin::delete_channel))
            .route("/users", web::get().to(admin::list_users))
            .route("/users/{discord_id}", web::delete().to(admin::unlink_user))
            .route("/interactions", web::get().to(admin::list_interactions))
            .route("/announce", web::post().to(admin::announce)),
    );
}

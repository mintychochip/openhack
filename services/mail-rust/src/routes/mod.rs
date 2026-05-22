pub mod attachments;
pub mod broadcast;
pub mod drip;
pub mod logs;
pub mod send;
pub mod templates;
pub mod webhooks;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/mail")
            .route("/drip/campaigns", web::post().to(drip::create_campaign))
            .route("/drip/campaigns", web::get().to(drip::list_campaigns))
            .route("/drip/campaigns/{id}", web::get().to(drip::get_campaign))
            .route("/drip/campaigns/{id}", web::put().to(drip::update_campaign))
            .route("/drip/campaigns/{id}", web::delete().to(drip::delete_campaign))
            .route(
                "/drip/campaigns/{id}/steps",
                web::post().to(drip::create_step),
            )
            .route(
                "/drip/campaigns/{id}/steps",
                web::get().to(drip::list_steps),
            )
            .route(
                "/drip/steps/{id}",
                web::put().to(drip::update_step),
            )
            .route(
                "/drip/steps/{id}",
                web::delete().to(drip::delete_step),
            )
            .route(
                "/drip/campaigns/{id}/enroll",
                web::post().to(drip::enroll),
            )
            .route(
                "/drip/campaigns/{id}/enrollments",
                web::get().to(drip::list_enrollments),
            )
            .route(
                "/drip/enrollments/{id}/unsubscribe",
                web::post().to(drip::unsubscribe),
            )
            .route(
                "/drip/process",
                web::post().to(drip::process_pending),
            )
            .route("/send", web::post().to(send::send_email))
            .route("/send/bulk", web::post().to(send::send_bulk))
            .route("/templates", web::post().to(templates::create_template))
            .route("/templates", web::get().to(templates::list_templates))
            .route("/templates/{id}", web::get().to(templates::get_template))
            .route("/templates/{id}", web::put().to(templates::update_template))
            .route(
                "/templates/{id}",
                web::delete().to(templates::delete_template),
            )
            .route(
                "/templates/{id}/preview",
                web::post().to(templates::preview_template),
            )
            .route("/broadcast", web::post().to(broadcast::create_broadcast))
            .route("/broadcast", web::get().to(broadcast::list_broadcasts))
            .route("/broadcast/{id}", web::get().to(broadcast::get_broadcast))
            .route(
                "/broadcast/{id}/schedule",
                web::post().to(broadcast::schedule_broadcast),
            )
            .route("/logs", web::get().to(logs::list_logs))
            .route("/logs/{message_id}", web::get().to(logs::get_log))
            .route("/logs/{message_id}/events", web::get().to(logs::get_events))
            .route(
                "/webhooks/sendgrid",
                web::post().to(webhooks::sendgrid_webhook),
            )
            .route("/webhooks/ses", web::post().to(webhooks::ses_webhook))
            .route(
                "/attachments/upload",
                web::post().to(attachments::upload_attachment),
            )
            .route(
                "/attachments/{file_id}",
                web::delete().to(attachments::delete_attachment),
            ),
    );
}

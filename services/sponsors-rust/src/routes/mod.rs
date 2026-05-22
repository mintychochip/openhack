pub mod booths;
pub mod interactivity;
pub mod prizes;
pub mod sponsor;
pub mod submissions;

use actix_web::web;

/// Register all sponsor service routes on the given service config.
///
/// # Expected Behavior
///
/// Configures all API endpoints under the `/api/sponsors` prefix:
/// - POST `/booths` → create booth (sponsor role)
/// - GET `/booths` → list published booths (public)
/// - GET `/booths/{id}` → get booth with view increment (auth)
/// - PUT `/booths/{id}` → update booth (sponsor role)
/// - DELETE `/booths/{id}` → delete booth (sponsor role)
/// - POST `/booths/{id}/publish` → publish/unpublish booth (sponsor role)
/// - GET `/booths/{id}/analytics` → booth analytics (sponsor role)
/// - POST `/booths/{boothId}/prizes` → create prize (sponsor role)
/// - GET `/booths/{boothId}/prizes` → list prizes (auth)
/// - PUT `/prizes/{id}` → update prize (sponsor role)
/// - DELETE `/prizes/{id}` → delete prize (sponsor role)
/// - POST `/prizes/{id}/announce` → announce winner (sponsor role)
/// - POST `/prizes/{prizeId}/submit` → submit project for prize (auth)
/// - GET `/prizes/{prizeId}/submissions` → list submissions (sponsor role)
/// - POST `/prizes/{prizeId}/submissions/{projectId}/select` → select winner (sponsor role)
///
/// Identity-driven routes under `/api/sponsors/sponsor`:
/// - GET `/sponsor/booth` → get sponsor's own booth (sponsor role)
/// - POST `/sponsor/booth` → create booth for sponsor (sponsor role)
/// - PUT `/sponsor/booth/{id}` → update sponsor's booth (sponsor role, ownership)
/// - DELETE `/sponsor/booth/{id}` → delete sponsor's booth (sponsor role, ownership)
/// - GET `/sponsor/prizes` → list sponsor's prizes (sponsor role)
/// - POST `/sponsor/prizes` → create prize for sponsor (sponsor role)
/// - PUT `/sponsor/prizes/{id}` → update sponsor's prize (sponsor role, ownership)
/// - DELETE `/sponsor/prizes/{id}` → delete sponsor's prize (sponsor role, ownership)
/// - POST `/sponsor/prizes/{id}/winner` → select winner (sponsor role, ownership)
/// - GET `/sponsor/submissions` → list sponsor's submissions (sponsor role)
/// - POST `/sponsor/submissions/{id}/approve` → approve submission (sponsor role, ownership)
/// - POST `/sponsor/submissions/{id}/reject` → reject submission (sponsor role, ownership)
///
/// # Errors
///
/// None. Route registration is infallible.
///
/// # Side Effects
///
/// - Registers HTTP route handlers with the Actix-web service config.
///   No I/O or network calls occur during registration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/sponsors")
            .route("/booths", web::post().to(booths::create_booth))
            .route("/booths", web::get().to(booths::list_published_booths))
            .route("/booths/{id}", web::get().to(booths::get_booth))
            .route("/booths/{id}", web::put().to(booths::update_booth))
            .route("/booths/{id}", web::delete().to(booths::delete_booth))
            .route(
                "/booths/{id}/publish",
                web::post().to(booths::publish_booth),
            )
            .route(
                "/booths/{id}/analytics",
                web::get().to(booths::get_analytics),
            )
            .route(
                "/booths/{id}/visitors",
                web::post().to(interactivity::record_visit),
            )
            .route(
                "/booths/{id}/visitors",
                web::get().to(interactivity::get_visitors),
            )
            .route(
                "/visits/{id}/complete",
                web::post().to(interactivity::update_visit_duration),
            )
            .route(
                "/booths/{id}/messages",
                web::post().to(interactivity::create_message),
            )
            .route(
                "/booths/{id}/messages",
                web::get().to(interactivity::get_messages),
            )
            .route(
                "/messages/{id}/read",
                web::post().to(interactivity::mark_message_read),
            )
            .route(
                "/booths/{id}/polls",
                web::post().to(interactivity::create_poll),
            )
            .route(
                "/booths/{id}/polls",
                web::get().to(interactivity::get_polls),
            )
            .route(
                "/polls/{id}/vote",
                web::post().to(interactivity::vote_poll),
            )
            .route(
                "/booths/{id}/resources",
                web::post().to(interactivity::create_resource),
            )
            .route(
                "/booths/{id}/resources",
                web::get().to(interactivity::get_resources),
            )
            .route(
                "/resources/{id}/download",
                web::post().to(interactivity::download_resource),
            )
            .route(
                "/booths/{id}/analytics/interactivity",
                web::get().to(interactivity::get_analytics),
            )
            .route(
                "/booths/{booth_id}/prizes",
                web::post().to(prizes::create_prize),
            )
            .route(
                "/booths/{booth_id}/prizes",
                web::get().to(prizes::list_prizes),
            )
            .route("/prizes/{id}", web::put().to(prizes::update_prize))
            .route("/prizes/{id}", web::delete().to(prizes::delete_prize))
            .route(
                "/prizes/{id}/announce",
                web::post().to(prizes::announce_winner),
            )
            .route(
                "/prizes/{prize_id}/submit",
                web::post().to(submissions::submit_project),
            )
            .route(
                "/prizes/{prize_id}/submissions",
                web::get().to(submissions::list_submissions),
            )
            .route(
                "/prizes/{prize_id}/submissions/{project_id}/select",
                web::post().to(submissions::select_winner),
            )
            .service(
                web::scope("/sponsor")
                    .route("/booth", web::get().to(sponsor::get_sponsor_booth))
                    .route("/booth", web::post().to(sponsor::create_sponsor_booth))
                    .route("/booth/{id}", web::put().to(sponsor::update_sponsor_booth))
                    .route(
                        "/booth/{id}",
                        web::delete().to(sponsor::delete_sponsor_booth),
                    )
                    .route("/prizes", web::get().to(sponsor::list_sponsor_prizes))
                    .route("/prizes", web::post().to(sponsor::create_sponsor_prize))
                    .route("/prizes/{id}", web::put().to(sponsor::update_sponsor_prize))
                    .route(
                        "/prizes/{id}",
                        web::delete().to(sponsor::delete_sponsor_prize),
                    )
                    .route(
                        "/prizes/{id}/winner",
                        web::post().to(sponsor::select_sponsor_winner),
                    )
                    .route(
                        "/submissions",
                        web::get().to(sponsor::list_sponsor_submissions),
                    )
                    .route(
                        "/submissions/{id}/approve",
                        web::post().to(sponsor::approve_submission),
                    )
                    .route(
                        "/submissions/{id}/reject",
                        web::post().to(sponsor::reject_submission),
                    ),
            ),
    );
}

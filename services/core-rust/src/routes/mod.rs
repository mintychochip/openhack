pub mod admin;
pub mod checkin;
pub mod event;
pub mod hackathon;
pub mod lambda;
pub mod legal;
pub mod phase;
pub mod project;
pub mod screening;
pub mod search;
pub mod showcase;
pub mod team;

use actix_web::web;

/// Register all core service routes on the given service config.
///
/// # Expected Behavior
///
/// Configures all API endpoints under the `/api/core` prefix plus a
/// Lambda event dispatcher at `/lambda/event` for serverless routing.
///
/// # Errors
///
/// None. Route registration is infallible.
///
/// # Side Effects
///
/// - Registers HTTP route handlers with the Actix-web service config.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/core")
            .route("/info", web::get().to(hackathon::get_info))
            .route("/info", web::put().to(hackathon::update_info))
            .route("/teams", web::post().to(team::create_team))
            .route("/teams/{id}", web::get().to(team::get_team))
            .route("/teams/{id}", web::put().to(team::update_team))
            .route("/teams/{id}", web::delete().to(team::delete_team))
            .route("/teams/{id}/join", web::post().to(team::join_team))
            .route("/teams/{id}/leave", web::post().to(team::leave_team))
            .route(
                "/teams/{id}/members/invite",
                web::post().to(team::invite_member),
            )
            .route(
                "/teams/{id}/members/{user_id}/kick",
                web::post().to(team::kick_member),
            )
            .route("/teams/invites/accept", web::post().to(team::accept_invite))
            .route("/projects", web::post().to(project::create_project))
            .route("/projects", web::get().to(project::list_projects))
            .route("/projects/{id}", web::get().to(project::get_project))
            .route("/projects/{id}", web::put().to(project::update_project))
            .route("/projects/{id}", web::delete().to(project::delete_project))
            .route(
                "/projects/{id}/submit",
                web::post().to(project::submit_project),
            )
            .route("/projects/{id}/demo", web::post().to(project::upload_demo))
            .route(
                "/projects/{id}/demo",
                web::delete().to(project::delete_demo),
            )
            .route("/events", web::get().to(event::list_events))
            .route("/events", web::post().to(event::create_event))
            .route("/events/{id}", web::get().to(event::get_event))
            .route("/events/{id}", web::put().to(event::update_event))
            .route("/events/{id}", web::delete().to(event::delete_event))
            .route("/events/{id}/rsvp", web::post().to(event::rsvp_event))
            .route("/events/{id}/rsvp", web::delete().to(event::cancel_rsvp))
            .route(
                "/events/{id}/attendees",
                web::get().to(event::get_attendees),
            )
            .route(
                "/events/{id}/attendees/{user_id}",
                web::post().to(event::mark_attended),
            )
            .route("/phases", web::get().to(phase::list_phases))
            .route("/phases", web::post().to(phase::create_phase))
            .route("/phases/{id}", web::get().to(phase::get_phase))
            .route("/phases/{id}", web::put().to(phase::update_phase))
            .route("/phases/{id}", web::delete().to(phase::delete_phase))
            .route("/phases/{id}/open", web::post().to(phase::open_phase))
            .route("/phases/{id}/close", web::post().to(phase::close_phase))
            .route(
                "/phases/hackathon/{hackathon_id}/current",
                web::get().to(phase::get_current_phase),
            )
            .route(
                "/phases/hackathon/{hackathon_id}/next-transition",
                web::get().to(phase::get_next_transition),
            )
            .route("/showcase", web::get().to(showcase::get_showcase))
            .route("/search", web::get().to(search::search))
            .route("/projects/{id}/favorite", web::post().to(showcase::toggle_favorite))
            .route("/projects/{id}/favorite", web::get().to(showcase::get_favorite))
            .route("/users/looking-for-team", web::get().to(screening::list_team_seekers))
            .route("/teams/looking-for-members", web::get().to(screening::list_teams_seeking))
            .route("/profile/team-seeking", web::put().to(screening::update_team_seeking))
            .route(
                "/checkin/events/{id}/qr",
                web::post().to(checkin::create_qr),
            )
            .route("/checkin/events/{id}/qr", web::get().to(checkin::get_qr))
            .route("/checkin/scan", web::post().to(checkin::scan_checkin))
            .route(
                "/checkin/events/{id}/stats",
                web::get().to(checkin::get_stats),
            )
            .route(
                "/checkin/events/{id}/attendees",
                web::get().to(checkin::get_attendees),
            )
            .route("/rules", web::post().to(legal::create_rule))
            .route("/rules", web::get().to(legal::list_rules))
            .route("/rules/{id}", web::get().to(legal::get_rule))
            .route("/rules/{id}", web::put().to(legal::update_rule))
            .route("/rules/{id}/publish", web::post().to(legal::publish_rule))
            .route("/rules/{id}", web::delete().to(legal::delete_rule))
            .route("/fulfillment", web::post().to(legal::create_fulfillment))
            .route("/fulfillment", web::get().to(legal::list_fulfillments))
            .route("/fulfillment/{id}", web::get().to(legal::get_fulfillment))
            .route("/fulfillment/{id}", web::put().to(legal::update_fulfillment))
            .route("/fulfillment/{id}/claim", web::post().to(legal::claim_prize))
            .route("/fulfillment/{id}/ship", web::post().to(legal::ship_prize))
            .route("/fulfillment/{id}/fulfill", web::post().to(legal::fulfill_prize))
            .route("/fulfillment/{id}/decline", web::post().to(legal::decline_prize))
            .service(
                web::scope("/admin")
                    .route("/teams", web::get().to(admin::list_teams))
                    .route("/teams/{id}", web::get().to(admin::get_team))
                    .route("/teams/{id}", web::delete().to(admin::delete_team))
                    .route("/projects", web::get().to(admin::list_projects))
                    .route("/projects/{id}", web::get().to(admin::get_project))
                    .route(
                        "/projects/{id}/status",
                        web::put().to(admin::update_project_status),
                    )
                    .route("/projects/{id}", web::delete().to(admin::delete_project))
                    .route("/events", web::get().to(admin::list_events))
                    .route("/events", web::post().to(admin::create_event))
                    .route("/events/{id}", web::put().to(admin::update_event))
                    .route("/events/{id}", web::delete().to(admin::delete_event))
                    .route(
                        "/trigger-phase-check",
                        web::post().to(admin::trigger_phase_check),
                    )
                    .route("/projects/screening", web::get().to(screening::list_screening_queue))
                    .route("/projects/{id}/screen", web::post().to(screening::screen_project))
                    .route("/projects/{id}/feature", web::put().to(showcase::set_featured)),
            ),
    )
    .route("/lambda/event", web::post().to(lambda::dispatch));
}

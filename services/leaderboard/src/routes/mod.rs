pub mod admin;
pub mod lambda;
pub mod leaderboard;
pub mod recalculate;
pub mod stats;
pub mod votes;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/leaderboard")
            .route("", web::get().to(leaderboard::get_leaderboard))
            .route("/history", web::get().to(leaderboard::get_history))
            .route("/votes/{project_id}", web::get().to(votes::get_vote_count))
            .route("/vote", web::post().to(votes::cast_vote))
            .route("/formulas", web::get().to(leaderboard::list_formulas))
            .route(
                "/formulas/active",
                web::get().to(leaderboard::get_active_formula),
            )
            .route(
                "/voting-config",
                web::get().to(leaderboard::get_voting_config),
            )
            .route("/snapshots", web::get().to(leaderboard::list_snapshots))
            .route("/stats", web::get().to(stats::get_stats))
            .service(
                web::scope("/admin")
                    .route("/recalculate", web::post().to(recalculate::recalculate))
                    .route("/freeze", web::post().to(admin::freeze))
                    .route("/unfreeze", web::post().to(admin::unfreeze))
                    .route("/formulas", web::post().to(admin::create_formula))
                    .route(
                        "/formulas/{id}/activate",
                        web::put().to(admin::activate_formula),
                    )
                    .route("/voting-config", web::put().to(admin::update_voting_config))
                    .route("/snapshots", web::post().to(admin::create_snapshot))
                    .route("/votes/{id}/moderate", web::put().to(admin::moderate_vote)),
            ),
    )
    .route("/lambda/event", web::post().to(lambda::dispatch));
}

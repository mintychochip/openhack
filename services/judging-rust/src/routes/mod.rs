pub mod assignments;
pub mod dashboard;
pub mod normalization;
pub mod phases;
pub mod rubrics;
pub mod scores;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/judging")
            .route("/rubrics", web::post().to(rubrics::create_rubric))
            .route("/rubrics", web::get().to(rubrics::list_rubrics))
            .route("/rubrics/{id}", web::get().to(rubrics::get_rubric))
            .route("/rubrics/{id}", web::put().to(rubrics::update_rubric))
            .route("/rubrics/{id}", web::delete().to(rubrics::delete_rubric))
            .route(
                "/rubrics/{id}/versions",
                web::get().to(rubrics::list_versions),
            )
            .route(
                "/rubrics/{id}/versions/{version}",
                web::get().to(rubrics::get_version),
            )
            .route(
                "/rubrics/{id}/versions",
                web::post().to(rubrics::create_version),
            )
            .route(
                "/rubrics/validate-score",
                web::post().to(rubrics::validate_score),
            )
            .route(
                "/assignments",
                web::post().to(assignments::create_assignment),
            )
            .route(
                "/assignments/bulk",
                web::post().to(assignments::bulk_assign),
            )
            .route("/assignments", web::get().to(assignments::list_assignments))
            .route(
                "/assignments/{id}",
                web::put().to(assignments::update_assignment),
            )
            .route(
                "/assignments/{id}",
                web::delete().to(assignments::delete_assignment),
            )
            .route("/scores", web::post().to(scores::submit_score))
            .route("/scores/me", web::get().to(scores::get_my_scores))
            .route(
                "/scores/{project_id}",
                web::get().to(scores::get_project_scores),
            )
            .route("/scores/{score_id}", web::put().to(scores::update_score))
            .route("/dashboard", web::get().to(dashboard::get_dashboard))
            .route("/phases", web::post().to(phases::create_phase))
            .route("/phases", web::get().to(phases::list_phases))
            .route("/phases/{id}", web::get().to(phases::get_phase))
            .route("/phases/{id}", web::put().to(phases::update_phase))
            .route("/phases/{id}/open", web::post().to(phases::open_phase))
            .route("/phases/{id}/close", web::post().to(phases::close_phase))
            .route(
                "/phases/{id}/finalize",
                web::post().to(phases::finalize_phase),
            )
            .route(
                "/phases/{id}/leaderboard",
                web::get().to(phases::get_leaderboard),
            )
            .route(
                "/phases/{id}/advancements",
                web::get().to(phases::get_advancements),
            )
            .route(
                "/phases/{id}/advance",
                web::post().to(phases::advance_phase),
            )
            .route("/normalize", web::post().to(normalization::normalize))
            .route(
                "/normalize/{phase_id}/status",
                web::get().to(normalization::get_status),
            )
            .route(
                "/normalize/{phase_id}/stats",
                web::get().to(normalization::get_stats),
            )
            .route(
                "/normalize/{phase_id}/apply",
                web::post().to(normalization::apply),
            ),
    );
}

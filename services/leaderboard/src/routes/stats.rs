use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::services::ranking::RankingService;

/// Get leaderboard statistics.
///
/// # Expected Behavior
///
/// Public endpoint. Returns counts of teams, valid votes, frozen
/// teams, formulas, and snapshots as a JSON object.
///
/// # Errors
///
/// Returns 500 on database errors.
///
/// # Side Effects
///
/// - Reads from `leaderboard.ranks`, `leaderboard.votes`,
///   `leaderboard.ranking_formulas`, and `leaderboard.snapshots`
///   (database reads).
pub async fn get_stats(pool: web::Data<PgPool>) -> HttpResponse {
    match RankingService::get_stats(pool.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

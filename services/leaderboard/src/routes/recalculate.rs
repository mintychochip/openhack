use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::middleware::auth::get_auth_user;
use crate::services::ranking::RankingService;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RecalculateQuery {
    pub formula_id: Option<uuid::Uuid>,
}

/// Recalculate rankings (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Optionally accepts a `formula_id`
/// query parameter to use a specific formula; otherwise uses the active
/// formula. Recalculates `combined_score`, rank, `previous_rank`, and
/// records score history. Invalidates all caches.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 404 if no formula exists with the given ID or no active formula.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates `leaderboard.ranks` (database writes).
/// - Inserts into `leaderboard.score_history` (database write).
/// - Invalidates all leaderboard-related Redis caches (writes).
pub async fn recalculate(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    query: web::Query<RecalculateQuery>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    match RankingService::recalculate_rankings(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        query.formula_id,
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

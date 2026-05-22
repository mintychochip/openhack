use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::formula::FormulaService;
use crate::services::ranking::RankingService;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub phase_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HistoryQuery {
    pub team_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get current leaderboard sorted by rank.
///
/// # Expected Behavior
///
/// Public endpoint. Returns the leaderboard ordered by rank with
/// pagination via `limit` (default 100) and `offset` (default 0)
/// query parameters. Optional `phase_id` filters to a specific phase.
/// Checks Redis cache first.
///
/// # Errors
///
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn get_leaderboard(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    query: web::Query<LeaderboardQuery>,
) -> HttpResponse {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    let phase_id = query.phase_id;

    match RankingService::get_leaderboard(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        limit,
        offset,
        phase_id,
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Get score history for a team.
///
/// # Expected Behavior
///
/// Public endpoint. Requires `team_id` query parameter. Returns
/// score history ordered by `recorded_at` descending with pagination
/// via `limit` (default 50) and `offset` (default 0). Checks Redis
/// cache first.
///
/// # Errors
///
/// Returns 400 if `team_id` query parameter is missing.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn get_history(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    query: web::Query<HistoryQuery>,
) -> HttpResponse {
    let Some(team_id) = query.team_id else {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "team_id query parameter is required"}));
    };
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match RankingService::get_score_history(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        team_id,
        limit,
        offset,
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// List all ranking formulas.
///
/// # Expected Behavior
///
/// Public endpoint. Returns all ranking formulas ordered by `created_at`
/// descending. Checks Redis cache first.
///
/// # Errors
///
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn list_formulas(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
) -> HttpResponse {
    match FormulaService::list_formulas(pool.get_ref(), redis_conn.get_ref().as_ref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Get the currently active ranking formula.
///
/// # Expected Behavior
///
/// Public endpoint. Returns the single active ranking formula.
/// Checks Redis cache first.
///
/// # Errors
///
/// Returns 404 if no active formula exists.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn get_active_formula(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
) -> HttpResponse {
    match FormulaService::get_active_formula(pool.get_ref(), redis_conn.get_ref().as_ref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Get the active voting configuration.
///
/// # Expected Behavior
///
/// Public endpoint. Returns the active voting config including
/// voting windows, rate limits, and default vote weight. Checks
/// Redis cache first.
///
/// # Errors
///
/// Returns 404 if no active voting config exists.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn get_voting_config(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
) -> HttpResponse {
    match FormulaService::get_active_voting_config(pool.get_ref(), redis_conn.get_ref().as_ref())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// List all leaderboard snapshots.
///
/// # Expected Behavior
///
/// Public endpoint. Returns all snapshots ordered by `created_at`
/// descending. Checks Redis cache first.
///
/// # Errors
///
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries the database on cache miss (read).
/// - Writes to Redis cache on cache miss (write).
pub async fn list_snapshots(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
) -> HttpResponse {
    match FormulaService::list_snapshots(pool.get_ref(), redis_conn.get_ref().as_ref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::get_auth_user;
use crate::models::formula::FormulaCreate;
use crate::models::snapshot::SnapshotCreate;
use crate::models::vote::VoteModerate;
use crate::models::voting_config::VotingConfigUpdate;
use crate::services::formula::FormulaService;
use crate::services::ranking::RankingService;
use crate::services::voting::VotingService;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FreezeRequest {
    pub phase_id: Option<Uuid>,
}

/// Freeze the leaderboard (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Sets `is_frozen = true` for all
/// non-frozen teams. Optionally sets `phase_id` from the request body.
/// Returns the updated leaderboard.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates `leaderboard.ranks` (database write).
/// - Invalidates all leaderboard-related Redis caches (writes).
pub async fn freeze(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<FreezeRequest>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    match RankingService::freeze_leaderboard(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        body.phase_id,
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Unfreeze the leaderboard (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Sets `is_frozen = false` for all
/// frozen teams. Returns the updated leaderboard.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates `leaderboard.ranks` (database write).
/// - Invalidates all leaderboard-related Redis caches (writes).
pub async fn unfreeze(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    match RankingService::unfreeze_leaderboard(pool.get_ref(), redis_conn.get_ref().as_ref()).await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Create a new ranking formula (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Creates a new ranking formula
/// with the provided name, description, formula expression, and
/// variables. Returns 201 Created with the formula data.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Inserts a row into `leaderboard.ranking_formulas` (database write).
/// - Invalidates formulas cache (Redis write).
pub async fn create_formula(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<FormulaCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    match FormulaService::create_formula(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &body.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Activate a ranking formula (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Deactivates all formulas, then
/// activates the formula with the given ID. Returns the activated
/// formula data.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 404 if no formula exists with the given ID.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates `leaderboard.ranking_formulas` (database write).
/// - Invalidates formulas and active formula caches (Redis writes).
pub async fn activate_formula(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match FormulaService::activate_formula(pool.get_ref(), redis_conn.get_ref().as_ref(), id).await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Update voting configuration (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Updates the active voting
/// configuration with the provided fields. Returns the updated
/// config data.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 404 if no active voting config exists.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates `leaderboard.voting_config` (database write).
/// - Invalidates voting config cache (Redis write).
pub async fn update_voting_config(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<VotingConfigUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    match FormulaService::update_voting_config(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &body.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Create a leaderboard snapshot (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Captures the current leaderboard
/// state as a JSONB snapshot. Returns 201 Created with the snapshot data.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Reads from `leaderboard.ranks` (database read).
/// - Inserts a row into `leaderboard.snapshots` (database write).
/// - Invalidates snapshots cache (Redis write).
pub async fn create_snapshot(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<SnapshotCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    match FormulaService::create_snapshot(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &body.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => e.to_http_response(),
    }
}

/// Moderate a vote (admin only).
///
/// # Expected Behavior
///
/// Requires admin or organizer role. Updates the vote's `is_valid`
/// flag and `moderation_reason`. If validity changed, adjusts the
/// team's `public_votes` and `weighted_votes` accordingly. Returns the
/// updated vote data.
///
/// # Errors
///
/// Returns 401 if authentication fails.
/// Returns 403 if the user lacks admin/organizer role.
/// Returns 404 if no vote exists with the given ID.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates `leaderboard.votes` (database write).
/// - Updates `leaderboard.ranks` if validity changed (database write).
/// - Invalidates leaderboard cache (Redis write).
pub async fn moderate_vote(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<VoteModerate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = crate::middleware::auth::require_admin_role(&auth_user) {
        return e.to_http_response();
    }

    let vote_id = path.into_inner();
    let moderator_id = auth_user.user_id;

    match VotingService::moderate_vote(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        vote_id,
        moderator_id,
        &body.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

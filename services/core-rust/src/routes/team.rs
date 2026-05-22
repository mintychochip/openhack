use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::get_auth_user;
use crate::models::team::{InviteAcceptRequest, MemberInviteRequest, TeamCreate, TeamUpdate};
use crate::services::team::TeamService;

/// POST /api/core/teams — Create a new team.
///
/// # Expected Behavior
///
/// Requires authentication. Creates a team and adds the creator as leader.
///
/// # Side Effects
///
/// - Creates team and team member records (database writes).
/// - Publishes `team.created` event to Redis.
pub async fn create_team(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<TeamCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let mut redis_local = redis_conn.get_ref().clone();
    match TeamService::create_team(
        pool.get_ref(),
        redis_local.as_mut(),
        user_id,
        &body.into_inner(),
    )
    .await
    {
        Ok(team) => {
            openhack_common::metrics::inc_business_counter("core_teams_created_total");
            HttpResponse::Created().json(team)
        }
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/teams/{id} — Get a team by ID.
///
/// # Expected Behavior
///
/// Optional auth. Returns team details with member count.
///
/// # Side Effects
///
/// - Reads from `core.teams` and `core.team_members` (database reads).
pub async fn get_team(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    match TeamService::get_team(pool.get_ref(), id).await {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(e) => e.to_http_response(),
    }
}

/// PUT /api/core/teams/{id} — Update a team.
///
/// # Expected Behavior
///
/// Requires authentication. Only the team leader can update.
///
/// # Side Effects
///
/// - Updates `core.teams` (database write).
pub async fn update_team(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<TeamUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    let mut redis_local = redis_conn.get_ref().clone();
    match TeamService::update_team(
        pool.get_ref(),
        redis_local.as_mut(),
        id,
        user_id,
        &body.into_inner(),
    )
    .await
    {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/teams/{id} — Delete a team.
///
/// # Expected Behavior
///
/// Requires authentication. Only the team leader can delete.
///
/// # Side Effects
///
/// - Deletes from `core.teams` (database write, cascades).
pub async fn delete_team(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    match TeamService::delete_team(pool.get_ref(), id, user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/teams/{id}/join — Join a team.
///
/// # Expected Behavior
///
/// Requires authentication. Adds the user as a member.
///
/// # Side Effects
///
/// - Inserts into `core.team_members` (database write).
/// - Publishes `team.joined` event to Redis.
pub async fn join_team(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    let mut redis_local = redis_conn.get_ref().clone();
    match TeamService::join_team(pool.get_ref(), redis_local.as_mut(), id, user_id).await {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/teams/{id}/leave — Leave a team.
///
/// # Expected Behavior
///
/// Requires authentication. Removes the user from the team.
///
/// # Side Effects
///
/// - Deletes from `core.team_members` (database write).
/// - Publishes `team.left` event to Redis.
pub async fn leave_team(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    let mut redis_local = redis_conn.get_ref().clone();
    match TeamService::leave_team(pool.get_ref(), redis_local.as_mut(), id, user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/teams/{id}/members/invite — Invite a member.
///
/// # Expected Behavior
///
/// Requires authentication. Only team members can invite.
///
/// # Side Effects
///
/// - Inserts into `core.team_invites` (database write).
pub async fn invite_member(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<MemberInviteRequest>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let invited_by = auth_user.user_id;

    let team_id = path.into_inner();
    match TeamService::invite_member(pool.get_ref(), team_id, &body.email, invited_by).await {
        Ok(invite) => HttpResponse::Created().json(invite),
        Err(e) => e.to_http_response(),
    }
}

/// POST /`api/core/teams/{id}/members/{user_id}/kick` — Kick a member.
///
/// # Expected Behavior
///
/// Requires authentication. Only the team leader can kick.
///
/// # Side Effects
///
/// - Deletes from `core.team_members` (database write).
pub async fn kick_member(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let kicker_id = auth_user.user_id;

    let (team_id, user_id) = path.into_inner();
    match TeamService::kick_member(pool.get_ref(), team_id, user_id, kicker_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/teams/invites/accept — Accept a team invite.
///
/// # Expected Behavior
///
/// Requires authentication. Marks the invite as accepted and joins the team.
///
/// # Side Effects
///
/// - Updates `core.team_invites` (database write).
/// - Inserts into `core.team_members` (database write).
/// - Publishes `team.joined` event to Redis.
pub async fn accept_invite(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    body: web::Json<InviteAcceptRequest>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let mut redis_local = redis_conn.get_ref().clone();
    match TeamService::accept_invite(
        pool.get_ref(),
        redis_local.as_mut(),
        body.invite_id,
        user_id,
    )
    .await
    {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(e) => e.to_http_response(),
    }
}

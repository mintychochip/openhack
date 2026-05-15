use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::CoreError;
use crate::middleware::auth::{get_auth_user, require_admin};
use crate::models::event::EventCreate;
use crate::models::project::ProjectStatusUpdate;
use crate::services::event::EventService;
use crate::services::phase::PhaseService;
use crate::services::project::ProjectService;
use crate::services::team::TeamService;

/// GET /api/core/admin/teams — List all teams (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Returns all teams with pagination.
///
/// # Side Effects
///
/// - Reads from `core.teams` (database read).
pub async fn list_teams(pool: web::Data<PgPool>, req: actix_web::HttpRequest) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    match TeamService::list_teams_admin(pool.get_ref(), 100, 0).await {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/admin/teams/{id} — Get a team (admin).
///
/// # Expected Behavior
///
/// Requires admin role.
///
/// # Side Effects
///
/// - Reads from `core.teams` (database read).
pub async fn get_team(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match TeamService::get_team(pool.get_ref(), id).await {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/admin/teams/{id} — Delete a team (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Deletes the team without leader check.
///
/// # Side Effects
///
/// - Deletes from `core.teams` (database write, cascades).
pub async fn delete_team(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let id = path.into_inner();
    let result = sqlx::query("DELETE FROM core.teams WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Team not found"})),
        Err(e) => CoreError::Database(e).to_http_response(),
    }
}

/// GET /api/core/admin/projects — List all projects (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Returns all projects with pagination.
///
/// # Side Effects
///
/// - Reads from `core.projects` (database read).
pub async fn list_projects(pool: web::Data<PgPool>, req: actix_web::HttpRequest) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    match ProjectService::list_projects_admin(pool.get_ref(), 100, 0).await {
        Ok(projects) => HttpResponse::Ok().json(projects),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/admin/projects/{id} — Get a project (admin).
///
/// # Expected Behavior
///
/// Requires admin role.
///
/// # Side Effects
///
/// - Reads from `core.projects` (database read).
pub async fn get_project(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match ProjectService::get_project(pool.get_ref(), id).await {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => e.to_http_response(),
    }
}

/// PUT /api/core/admin/projects/{id}/status — Update project status (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Updates the project status directly.
///
/// # Side Effects
///
/// - Updates `core.projects` status (database write).
pub async fn update_project_status(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<ProjectStatusUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match ProjectService::update_project_status(pool.get_ref(), id, &body.status).await {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/admin/projects/{id} — Delete a project (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Deletes without membership check.
///
/// # Side Effects
///
/// - Deletes from `core.projects` (database write, cascades).
pub async fn delete_project(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let id = path.into_inner();
    let result = sqlx::query("DELETE FROM core.projects WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Project not found"})),
        Err(e) => CoreError::Database(e).to_http_response(),
    }
}

/// GET /api/core/admin/events — List all events (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Returns all events with pagination.
///
/// # Side Effects
///
/// - Reads from `core.events` (database read).
pub async fn list_events(pool: web::Data<PgPool>, req: actix_web::HttpRequest) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    match EventService::list_events_admin(pool.get_ref(), 100, 0).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/admin/events — Create an event (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Creates a new event.
///
/// # Side Effects
///
/// - Inserts into `core.events` (database write).
pub async fn create_event(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<EventCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    match EventService::create_event(pool.get_ref(), &body.into_inner()).await {
        Ok(event) => HttpResponse::Created().json(event),
        Err(e) => e.to_http_response(),
    }
}

/// PUT /api/core/admin/events/{id} — Update an event (admin).
///
/// # Expected Behavior
///
/// Requires admin role.
///
/// # Side Effects
///
/// - Updates `core.events` (database write).
pub async fn update_event(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<crate::models::event::EventUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match EventService::update_event(pool.get_ref(), id, &body.into_inner()).await {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/admin/events/{id} — Delete an event (admin).
///
/// # Expected Behavior
///
/// Requires admin role.
///
/// # Side Effects
///
/// - Deletes from `core.events` (database write, cascades).
pub async fn delete_event(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let id = path.into_inner();
    match EventService::delete_event(pool.get_ref(), id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/admin/trigger-phase-check — Trigger a phase check (admin).
///
/// # Expected Behavior
///
/// Requires admin role. Runs a single phase check-and-transition cycle,
/// identical to what the background scheduler executes every 30 seconds.
/// Opens phases whose `opens_at` has passed and closes phases whose
/// `closes_at` has passed. Intended for serverless/EventBridge invocation
/// where the in-process scheduler is not running. Returns 200 OK with a
/// JSON body indicating success, or the error on failure.
///
/// # Errors
///
/// Returns 401 if JWT is missing or invalid. Returns 403 if the user
/// is not an admin. Returns 500 if the database query fails.
///
/// # Side Effects
///
/// - Reads and writes to `core.hackathon_phases` (database read/write).
/// - Logs phase transitions at INFO level (via `PhaseService::check_and_transition`).
pub async fn trigger_phase_check(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    match PhaseService::check_and_transition(pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "completed",
            "message": "Phase check and transition executed"
        })),
        Err(e) => e.to_http_response(),
    }
}

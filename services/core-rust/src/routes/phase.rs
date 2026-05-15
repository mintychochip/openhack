use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::{get_auth_user, require_admin_or_organizer};
use crate::models::phase::{PhaseCreate, PhaseUpdate};
use crate::services::phase::PhaseService;

/// GET /api/core/phases — List all phases.
///
/// # Expected Behavior
///
/// Requires authentication. Returns all phases ordered by `opens_at`.
///
/// # Side Effects
///
/// - Reads from `core.hackathon_phases` (database read).
pub async fn list_phases(pool: web::Data<PgPool>, req: actix_web::HttpRequest) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    match PhaseService::list_phases(pool.get_ref()).await {
        Ok(phases) => HttpResponse::Ok().json(phases),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/phases/{id} — Get a phase by ID.
///
/// # Expected Behavior
///
/// Requires authentication. Returns phase details.
///
/// # Side Effects
///
/// - Reads from `core.hackathon_phases` (database read).
pub async fn get_phase(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let id = path.into_inner();
    match PhaseService::get_phase(pool.get_ref(), id).await {
        Ok(phase) => HttpResponse::Ok().json(phase),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/phases — Create a new phase.
///
/// # Expected Behavior
///
/// Requires authentication and admin/organizer role.
///
/// # Side Effects
///
/// - Inserts into `core.hackathon_phases` (database write).
pub async fn create_phase(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<PhaseCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin_or_organizer(&auth_user) {
        return e.to_http_response();
    }

    match PhaseService::create_phase(pool.get_ref(), &body.into_inner()).await {
        Ok(phase) => HttpResponse::Created().json(phase),
        Err(e) => e.to_http_response(),
    }
}

/// PUT /api/core/phases/{id} — Update a phase.
///
/// # Expected Behavior
///
/// Requires authentication and admin/organizer role.
///
/// # Side Effects
///
/// - Updates `core.hackathon_phases` (database write).
pub async fn update_phase(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<PhaseUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin_or_organizer(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::update_phase(pool.get_ref(), id, &body.into_inner()).await {
        Ok(phase) => HttpResponse::Ok().json(phase),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/phases/{id} — Delete a phase.
///
/// # Expected Behavior
///
/// Requires authentication and admin/organizer role.
///
/// # Side Effects
///
/// - Deletes from `core.hackathon_phases` (database write).
pub async fn delete_phase(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin_or_organizer(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::delete_phase(pool.get_ref(), id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/phases/{id}/open — Manually open a phase.
///
/// # Expected Behavior
///
/// Requires authentication and admin/organizer role. Opens the phase and
/// closes any other active phase for the same hackathon.
///
/// # Side Effects
///
/// - Updates `core.hackathon_phases` (database write).
pub async fn open_phase(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin_or_organizer(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::open_phase(pool.get_ref(), id).await {
        Ok(phase) => HttpResponse::Ok().json(phase),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/phases/{id}/close — Manually close a phase.
///
/// # Expected Behavior
///
/// Requires authentication and admin/organizer role.
///
/// # Side Effects
///
/// - Updates `core.hackathon_phases` (database write).
pub async fn close_phase(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin_or_organizer(&auth_user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::close_phase(pool.get_ref(), id).await {
        Ok(phase) => HttpResponse::Ok().json(phase),
        Err(e) => e.to_http_response(),
    }
}

/// GET /`api/core/phases/hackathon/{hackathon_id}/current` — Get current phase.
///
/// # Expected Behavior
///
/// Optional auth. Returns the currently active phase for the hackathon.
///
/// # Side Effects
///
/// - Reads from `core.hackathon_phases` (database read).
pub async fn get_current_phase(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let hackathon_id = path.into_inner();
    match PhaseService::get_current_phase(pool.get_ref(), hackathon_id).await {
        Ok(Some(phase)) => HttpResponse::Ok().json(phase),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "No active phase"})),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/phases/hackathon/{hackathon_id}/next-transition — Get next transition.
///
/// # Expected Behavior
///
/// Optional auth. Returns the next phase transition for the hackathon.
///
/// # Side Effects
///
/// - Reads from `core.hackathon_phases` (database read).
pub async fn get_next_transition(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let hackathon_id = path.into_inner();
    match PhaseService::get_next_transition(pool.get_ref(), hackathon_id).await {
        Ok(Some(transition)) => HttpResponse::Ok().json(transition),
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "No upcoming transitions"}))
        }
        Err(e) => e.to_http_response(),
    }
}

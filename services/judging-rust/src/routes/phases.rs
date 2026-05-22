use crate::middleware::auth::{get_auth_user, require_admin_or_organizer};
use crate::models::phase::{PhaseAdvanceRequest, PhaseCreate, PhaseQuery, PhaseUpdate};
use crate::services::phase::PhaseService;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_phase(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<PhaseCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    match PhaseService::create(pool.get_ref(), &body.into_inner()).await {
        Ok(p) => HttpResponse::Created().json(p),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_phases(pool: web::Data<PgPool>, query: web::Query<PhaseQuery>) -> HttpResponse {
    match PhaseService::list(pool.get_ref(), &query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_phase(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match PhaseService::get(pool.get_ref(), id).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_phase(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<PhaseUpdate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::update(pool.get_ref(), id, &body.into_inner()).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(e) => e.to_http_response(),
    }
}

pub async fn open_phase(req: HttpRequest, pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::open_phase(pool.get_ref(), id).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(e) => e.to_http_response(),
    }
}

pub async fn close_phase(req: HttpRequest, pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::close_phase(pool.get_ref(), id).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(e) => e.to_http_response(),
    }
}

pub async fn finalize_phase(req: HttpRequest, pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match PhaseService::finalize_phase(pool.get_ref(), id).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_leaderboard(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let phase_id = path.into_inner();
    match PhaseService::get_leaderboard(pool.get_ref(), phase_id).await {
        Ok(lb) => HttpResponse::Ok().json(lb),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_advancements(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let phase_id = path.into_inner();
    match PhaseService::get_advancements(pool.get_ref(), phase_id).await {
        Ok(a) => HttpResponse::Ok().json(a),
        Err(e) => e.to_http_response(),
    }
}

pub async fn advance_phase(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<PhaseAdvanceRequest>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin_or_organizer(&user) {
        return e.to_http_response();
    }

    let phase_id = path.into_inner();
    match PhaseService::advance_phase(pool.get_ref(), phase_id, &body.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

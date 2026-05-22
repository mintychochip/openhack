use crate::middleware::auth::{get_auth_user, require_admin};
use crate::models::drip::{
    DripCampaignCreate, DripCampaignQuery, DripCampaignUpdate, DripEnrollRequest, DripStepCreate,
    DripStepUpdate,
};
use crate::services::drip::DripService;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_campaign(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<DripCampaignCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    match DripService::create_campaign(pool.get_ref(), &body.into_inner()).await {
        Ok(campaign) => HttpResponse::Created().json(campaign),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_campaign(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match DripService::get_campaign(pool.get_ref(), id).await {
        Ok(campaign) => HttpResponse::Ok().json(campaign),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_campaigns(
    pool: web::Data<PgPool>,
    query: web::Query<DripCampaignQuery>,
) -> HttpResponse {
    match DripService::list_campaigns(pool.get_ref(), &query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_campaign(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<DripCampaignUpdate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match DripService::update_campaign(pool.get_ref(), id, &body.into_inner()).await {
        Ok(campaign) => HttpResponse::Ok().json(campaign),
        Err(e) => e.to_http_response(),
    }
}

pub async fn delete_campaign(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match DripService::delete_campaign(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => e.to_http_response(),
    }
}

pub async fn create_step(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<DripStepCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let campaign_id = path.into_inner();
    match DripService::create_step(pool.get_ref(), campaign_id, &body.into_inner()).await {
        Ok(step) => HttpResponse::Created().json(step),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_steps(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let campaign_id = path.into_inner();
    match DripService::list_steps(pool.get_ref(), campaign_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_step(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<DripStepUpdate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match DripService::update_step(pool.get_ref(), id, &body.into_inner()).await {
        Ok(step) => HttpResponse::Ok().json(step),
        Err(e) => e.to_http_response(),
    }
}

pub async fn delete_step(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match DripService::delete_step(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => e.to_http_response(),
    }
}

pub async fn enroll(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<DripEnrollRequest>,
) -> HttpResponse {
    let campaign_id = path.into_inner();
    match DripService::enroll(pool.get_ref(), campaign_id, &body.into_inner()).await {
        Ok(enrollment) => HttpResponse::Created().json(enrollment),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_enrollments(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let campaign_id = path.into_inner();
    match DripService::list_enrollments(pool.get_ref(), campaign_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn unsubscribe(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let enrollment_id = path.into_inner();
    match DripService::unsubscribe(pool.get_ref(), enrollment_id).await {
        Ok(enrollment) => HttpResponse::Ok().json(enrollment),
        Err(e) => e.to_http_response(),
    }
}

pub async fn process_pending(pool: web::Data<PgPool>) -> HttpResponse {
    match DripService::process_pending(pool.get_ref()).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => e.to_http_response(),
    }
}

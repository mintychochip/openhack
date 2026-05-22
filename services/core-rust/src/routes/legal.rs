use crate::middleware::auth::{get_auth_user, require_admin};
use crate::models::legal::{
    HackathonRuleCreate, HackathonRuleQuery, HackathonRuleUpdate, PrizeFulfillmentCreate,
    PrizeFulfillmentQuery, PrizeFulfillmentUpdate, ShippingAddress,
};
use crate::services::legal::{FulfillmentService, LegalService};
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_rule(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<HackathonRuleCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    match LegalService::create_rule(pool.get_ref(), &body.into_inner()).await {
        Ok(rule) => HttpResponse::Created().json(rule),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_rule(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match LegalService::get_rule(pool.get_ref(), id).await {
        Ok(rule) => HttpResponse::Ok().json(rule),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_rules(
    pool: web::Data<PgPool>,
    query: web::Query<HackathonRuleQuery>,
) -> HttpResponse {
    match LegalService::list_rules(pool.get_ref(), &query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_rule(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<HackathonRuleUpdate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match LegalService::update_rule(pool.get_ref(), id, &body.into_inner()).await {
        Ok(rule) => HttpResponse::Ok().json(rule),
        Err(e) => e.to_http_response(),
    }
}

pub async fn publish_rule(
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
    match LegalService::publish_rule(pool.get_ref(), id).await {
        Ok(rule) => HttpResponse::Ok().json(rule),
        Err(e) => e.to_http_response(),
    }
}

pub async fn delete_rule(
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
    match LegalService::delete_rule(pool.get_ref(), id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => e.to_http_response(),
    }
}

pub async fn create_fulfillment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<PrizeFulfillmentCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    match FulfillmentService::create(pool.get_ref(), &body.into_inner()).await {
        Ok(f) => HttpResponse::Created().json(f),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_fulfillment(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match FulfillmentService::get(pool.get_ref(), id).await {
        Ok(f) => HttpResponse::Ok().json(f),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_fulfillments(
    pool: web::Data<PgPool>,
    query: web::Query<PrizeFulfillmentQuery>,
) -> HttpResponse {
    match FulfillmentService::list(pool.get_ref(), &query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_fulfillment(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<PrizeFulfillmentUpdate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match FulfillmentService::update(pool.get_ref(), id, &body.into_inner()).await {
        Ok(f) => HttpResponse::Ok().json(f),
        Err(e) => e.to_http_response(),
    }
}

pub async fn claim_prize(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<ShippingAddress>,
) -> HttpResponse {
    let id = path.into_inner();
    
    match FulfillmentService::claim(pool.get_ref(), id, &body.into_inner()).await {
        Ok(f) => HttpResponse::Ok().json(f),
        Err(e) => e.to_http_response(),
    }
}

pub async fn ship_prize(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    let tracking = body.get("tracking_number")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let carrier = body.get("carrier")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let estimated_delivery = body.get("estimated_delivery")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    match FulfillmentService::ship(pool.get_ref(), id, tracking, carrier, estimated_delivery).await {
        Ok(f) => HttpResponse::Ok().json(f),
        Err(e) => e.to_http_response(),
    }
}

pub async fn fulfill_prize(
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
    match FulfillmentService::fulfill(pool.get_ref(), id).await {
        Ok(f) => HttpResponse::Ok().json(f),
        Err(e) => e.to_http_response(),
    }
}

pub async fn decline_prize(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    let notes = body.get("notes")
        .and_then(|v| v.as_str())
        .unwrap_or("Prize declined");

    match FulfillmentService::decline(pool.get_ref(), id, notes).await {
        Ok(f) => HttpResponse::Ok().json(f),
        Err(e) => e.to_http_response(),
    }
}

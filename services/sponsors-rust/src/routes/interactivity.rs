use crate::middleware::auth::get_auth_user;
use crate::models::interactivity::{
    BoothMessageCreate, BoothPollCreate, BoothPollVote, BoothResourceCreate, BoothVisitorCreate,
};
use crate::services::interactivity::InteractivityService;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn record_visit(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<BoothVisitorCreate>,
) -> HttpResponse {
    let booth_id = path.into_inner();
    match InteractivityService::record_visit(pool.get_ref(), booth_id, &body.into_inner()).await {
        Ok(visit) => HttpResponse::Created().json(visit),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_visit_duration(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let visit_id = path.into_inner();
    let duration = body
        .get("duration_seconds")
        .and_then(|v| v.as_i64())
        .map(|d| d as i32)
        .unwrap_or(0);

    match InteractivityService::update_visit_duration(pool.get_ref(), visit_id, duration).await {
        Ok(visit) => HttpResponse::Ok().json(visit),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_visitors(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
) -> HttpResponse {
    let booth_id = path.into_inner();
    let limit = query.get("limit").and_then(|v| v.as_i64()).unwrap_or(100);
    let offset = query.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);

    match InteractivityService::get_visitors(pool.get_ref(), booth_id, limit, offset).await {
        Ok(visitors) => HttpResponse::Ok().json(visitors),
        Err(e) => e.to_http_response(),
    }
}

pub async fn create_message(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<BoothMessageCreate>,
) -> HttpResponse {
    let booth_id = path.into_inner();
    match InteractivityService::create_message(pool.get_ref(), booth_id, &body.into_inner()).await {
        Ok(message) => HttpResponse::Created().json(message),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_messages(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
) -> HttpResponse {
    let booth_id = path.into_inner();
    let limit = query.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let offset = query.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);

    match InteractivityService::get_messages(pool.get_ref(), booth_id, limit, offset).await {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => e.to_http_response(),
    }
}

pub async fn mark_message_read(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let message_id = path.into_inner();
    match InteractivityService::mark_message_read(pool.get_ref(), message_id).await {
        Ok(message) => HttpResponse::Ok().json(message),
        Err(e) => e.to_http_response(),
    }
}

pub async fn create_poll(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<BoothPollCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let booth_id = path.into_inner();
    match InteractivityService::create_poll(pool.get_ref(), booth_id, &body.into_inner(), Some(user.user_id)).await {
        Ok(poll) => HttpResponse::Created().json(poll),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_polls(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let booth_id = path.into_inner();
    match InteractivityService::get_polls(pool.get_ref(), booth_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

pub async fn vote_poll(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<BoothPollVote>,
) -> HttpResponse {
    let poll_id = path.into_inner();
    let data = body.into_inner();
    
    match InteractivityService::vote_poll(pool.get_ref(), poll_id, data.user_id, data.option_indices).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

pub async fn create_resource(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<BoothResourceCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    // Only sponsor can add resources to their booth
    let _booth_id = path.into_inner();
    // TODO: Verify user owns the booth
    
    match InteractivityService::create_resource(pool.get_ref(), _booth_id, &body.into_inner()).await {
        Ok(resource) => HttpResponse::Created().json(resource),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_resources(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let booth_id = path.into_inner();
    match InteractivityService::get_resources(pool.get_ref(), booth_id).await {
        Ok(resources) => HttpResponse::Ok().json(resources),
        Err(e) => e.to_http_response(),
    }
}

pub async fn download_resource(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let resource_id = path.into_inner();
    match InteractivityService::increment_download(pool.get_ref(), resource_id).await {
        Ok(resource) => HttpResponse::Ok().json(resource),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_analytics(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let booth_id = path.into_inner();
    // TODO: Verify user owns the booth
    
    match InteractivityService::get_analytics(pool.get_ref(), booth_id).await {
        Ok(analytics) => HttpResponse::Ok().json(analytics),
        Err(e) => e.to_http_response(),
    }
}

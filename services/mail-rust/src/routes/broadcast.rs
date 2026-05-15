use crate::models::broadcast::{BroadcastCreate, BroadcastQuery, BroadcastSchedule};
use crate::services::mail::BroadcastService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_broadcast(
    pool: web::Data<PgPool>,
    body: web::Json<BroadcastCreate>,
) -> HttpResponse {
    match BroadcastService::create(pool.get_ref(), &body.into_inner()).await {
        Ok(b) => HttpResponse::Created().json(b),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_broadcasts(
    pool: web::Data<PgPool>,
    query: web::Query<BroadcastQuery>,
) -> HttpResponse {
    match BroadcastService::list(pool.get_ref(), &query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_broadcast(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    match BroadcastService::get(pool.get_ref(), id).await {
        Ok(b) => HttpResponse::Ok().json(b),
        Err(e) => e.to_http_response(),
    }
}

pub async fn schedule_broadcast(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<BroadcastSchedule>,
) -> HttpResponse {
    let id = path.into_inner();
    match BroadcastService::schedule(pool.get_ref(), id, &body.into_inner()).await {
        Ok(b) => HttpResponse::Ok().json(b),
        Err(e) => e.to_http_response(),
    }
}

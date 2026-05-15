use crate::models::message::{LogsQuery, MessageResponse};
use crate::services::mail::MailService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

pub async fn list_logs(pool: web::Data<PgPool>, query: web::Query<LogsQuery>) -> HttpResponse {
    let q = query.into_inner();
    match MailService::list_messages(
        pool.get_ref(),
        q.page,
        q.page_size,
        q.status_filter.as_deref(),
    )
    .await
    {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_log(pool: web::Data<PgPool>, path: web::Path<String>) -> HttpResponse {
    let message_id = path.into_inner();
    match MailService::get_message(pool.get_ref(), &message_id).await {
        Ok(msg) => HttpResponse::Ok().json(MessageResponse::from(msg)),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_events(pool: web::Data<PgPool>, path: web::Path<String>) -> HttpResponse {
    let message_id_str = path.into_inner();
    let msg = match MailService::get_message(pool.get_ref(), &message_id_str).await {
        Ok(m) => m,
        Err(e) => return e.to_http_response(),
    };
    match MailService::get_message_events(pool.get_ref(), msg.id).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => e.to_http_response(),
    }
}

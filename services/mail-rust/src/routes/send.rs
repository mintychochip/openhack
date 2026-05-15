use crate::models::message::{BulkSendRequest, BulkSendResponse, SendRequest, SendResponse};
use crate::services::mail::MailService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

pub async fn send_email(
    pool: web::Data<PgPool>,
    smtp: web::Data<crate::services::smtp::SmtpProvider>,
    body: web::Json<SendRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    let body_html = req.body_html.unwrap_or_default();

    match MailService::send_email(
        pool.get_ref(),
        smtp.get_ref(),
        &req.to,
        &req.cc,
        &req.bcc,
        &req.subject,
        &body_html,
        req.body_text.as_deref(),
        req.template_id,
        &req.priority,
        &req.metadata,
    )
    .await
    {
        Ok((_, message_id)) => HttpResponse::Created().json(SendResponse {
            message_id,
            status: "queued".into(),
        }),
        Err(e) => e.to_http_response(),
    }
}

pub async fn send_bulk(
    pool: web::Data<PgPool>,
    smtp: web::Data<crate::services::smtp::SmtpProvider>,
    body: web::Json<BulkSendRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    let emails_json: Vec<serde_json::Value> = req
        .emails
        .iter()
        .map(|e| {
            serde_json::json!({
                "to": e.to,
                "subject": e.subject,
                "body_html": e.body_html,
            })
        })
        .collect();

    match MailService::send_bulk(
        pool.get_ref(),
        smtp.get_ref(),
        &emails_json,
        &req.batch_priority,
    )
    .await
    {
        Ok((batch_id, message_ids, total)) => HttpResponse::Accepted().json(BulkSendResponse {
            batch_id,
            message_ids,
            total,
        }),
        Err(e) => e.to_http_response(),
    }
}

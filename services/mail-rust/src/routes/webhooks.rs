use crate::services::mail::MailService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

pub async fn sendgrid_webhook(
    pool: web::Data<PgPool>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let Some(events) = body.as_array() else {
        return HttpResponse::Ok().json(serde_json::json!({"status": "ok", "processed": 0}));
    };

    let mut processed = 0;
    for event in events {
        let msg_id = match event.get("message_id").and_then(|v| v.as_str()) {
            Some(id) => format!("<{id}@mail.openhack.local>"),
            None => continue,
        };

        let Some(event_type) = event.get("event").and_then(|v| v.as_str()) else {
            continue;
        };

        let status = match event_type {
            "processed" => "queued",
            "delivered" => "delivered",
            "bounce" => "bounced",
            "dropped" | "spamreport" => "failed",
            _ => continue,
        };

        if let Err(e) = MailService::update_message_status(
            pool.get_ref(),
            &msg_id,
            status,
            &serde_json::json!({"provider": "sendgrid", "event": event_type}),
        )
        .await
        {
            log::warn!("Webhook update failed for {msg_id}: {e}");
        }
        processed += 1;
    }

    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "processed": processed}))
}

pub async fn ses_webhook(
    pool: web::Data<PgPool>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let msg_type = body.get("Type").and_then(|v| v.as_str()).unwrap_or("");

    if msg_type == "SubscriptionConfirmation" {
        return HttpResponse::Ok().json(serde_json::json!({"status": "subscription_pending"}));
    }

    if msg_type != "Notification" {
        return HttpResponse::Ok().json(serde_json::json!({"status": "ok"}));
    }

    let Some(inner_msg) = body
        .get("Message")
        .and_then(|v| v.as_str())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
    else {
        return HttpResponse::Ok().json(serde_json::json!({"status": "ok"}));
    };

    let event_type = inner_msg
        .get("eventType")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let status = match event_type {
        "Send" => "sent",
        "Delivery" => "delivered",
        "Bounce" => "bounced",
        "Complaint" | "Reject" => "failed",
        _ => "queued",
    };

    if let Some(mail) = inner_msg.get("mail") {
        if let Some(msg_id) = mail.get("messageId").and_then(|v| v.as_str()) {
            let _ = MailService::update_message_status(
                pool.get_ref(),
                msg_id,
                status,
                &serde_json::json!({"provider": "ses", "event": event_type}),
            )
            .await;
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

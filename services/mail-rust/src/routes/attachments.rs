use crate::models::attachment::AttachmentUploadResponse;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn upload_attachment(
    req: HttpRequest,
    config: web::Data<crate::config::Config>,
    body: web::Bytes,
) -> HttpResponse {
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let client = reqwest::Client::new();
    let media_url = format!(
        "{}/api/media/upload?folder=email-attachments",
        config.media_service_url
    );

    let resp = client
        .post(&media_url)
        .header("authorization", auth)
        .body(body.to_vec())
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
            Ok(data) => HttpResponse::Ok().json(AttachmentUploadResponse {
                file_id: data
                    .get("file_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: data
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                filename: data
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                size: data
                    .get("size")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0),
                content_type: data
                    .get("content_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
            _ => HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to parse media service response"})),
        },
        Ok(r) => HttpResponse::BadGateway().json(serde_json::json!({
            "error": format!("Media service returned {}", r.status())
        })),
        Err(e) => HttpResponse::BadGateway().json(serde_json::json!({
            "error": format!("Failed to contact media service: {}", e)
        })),
    }
}

pub async fn delete_attachment(
    path: web::Path<String>,
    req: HttpRequest,
    config: web::Data<crate::config::Config>,
) -> HttpResponse {
    let file_id = path.into_inner();
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let client = reqwest::Client::new();
    let media_url = format!("{}/api/media/{}", config.media_service_url, file_id);

    let resp = client
        .delete(&media_url)
        .header("authorization", auth)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            HttpResponse::Ok().json(serde_json::json!({"deleted": true}))
        }
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to delete attachment"})),
    }
}

use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AiError;
use crate::models::chat::ChatRequest;
use crate::services::chat;

/// Handle a chat message request.
///
/// # Expected Behavior
///
/// Accepts a JSON `ChatRequest` body. Checks if the chat feature is enabled
/// via `FEATURE_CHAT`. If disabled, returns 403. Otherwise, delegates to
/// `chat::handle_chat` which creates or continues a conversation, performs
/// RAG, calls the LLM, and returns a `ChatResponse`. Returns 200 on success,
/// 403 if feature is disabled, 404 if conversation not found, 500 on internal error.
/// If Redis is unavailable, embedding caching is skipped.
///
/// # Errors
///
/// Returns 403 if `FEATURE_CHAT` is false. Returns 500 on database or LLM errors.
///
/// # Side Effects
///
/// - Delegates all side effects to `chat::handle_chat`.
pub async fn chat(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<redis::aio::MultiplexedConnection>>,
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    req: web::Json<ChatRequest>,
) -> HttpResponse {
    if !config.feature_chat {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Chat feature is disabled"
        }));
    }

    let redis = redis_conn.get_ref().clone();
    match chat::handle_chat(
        pool.get_ref(),
        redis,
        http_client.get_ref(),
        config.get_ref(),
        &req.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AiError::NotFound(msg)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": msg
        })),
        Err(AiError::FeatureDisabled(msg)) => HttpResponse::Forbidden().json(serde_json::json!({
            "error": msg
        })),
        Err(e) => {
            log::error!("Chat error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process chat request"
            }))
        }
    }
}

/// Get conversation history with all messages.
///
/// # Expected Behavior
///
/// Loads the conversation and its messages from the database. Returns 200 with
/// a `ConversationDetail` on success. Returns 404 if the conversation does not exist.
///
/// # Errors
///
/// Returns 404 if conversation not found. Returns 500 on database error.
///
/// # Side Effects
///
/// - Reads from `ai.conversations` and `ai.messages` via service layer.
pub async fn get_history(
    pool: web::Data<PgPool>,
    conversation_id: web::Path<Uuid>,
) -> HttpResponse {
    let conversation_id = conversation_id.into_inner();

    match chat::get_conversation_history(pool.get_ref(), conversation_id).await {
        Ok(detail) => HttpResponse::Ok().json(detail),
        Err(AiError::NotFound(msg)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": msg
        })),
        Err(e) => {
            log::error!("Get history error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get conversation history"
            }))
        }
    }
}

/// Delete a conversation and all its messages.
///
/// # Expected Behavior
///
/// Deletes the conversation and all associated messages. Returns 204 on success
/// regardless of whether the conversation existed (idempotent).
///
/// # Errors
///
/// Returns 500 on database error.
///
/// # Side Effects
///
/// - Deletes from `ai.messages` and `ai.conversations` via service layer.
pub async fn delete_conversation(
    pool: web::Data<PgPool>,
    conversation_id: web::Path<Uuid>,
) -> HttpResponse {
    let conversation_id = conversation_id.into_inner();

    match chat::delete_conversation(pool.get_ref(), conversation_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Delete conversation error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete conversation"
            }))
        }
    }
}

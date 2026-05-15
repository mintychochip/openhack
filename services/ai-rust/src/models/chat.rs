use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Request body for the chat endpoint.
///
/// # Expected Behavior
///
/// If `conversation_id` is None, a new conversation is created. If provided,
/// the existing conversation is loaded and the new message is appended.
/// `user_id` identifies the chat participant. message is the user's input text.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    pub conversation_id: Option<Uuid>,
    pub user_id: Uuid,
    pub message: String,
}

/// Response body for the chat endpoint.
///
/// # Expected Behavior
///
/// Returns the `conversation_id` (new or existing), the assistant's response
/// message, sources from the RAG knowledge base, and the `created_at` timestamp
/// of the assistant message.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub conversation_id: Uuid,
    pub message: String,
    pub sources: Value,
    pub created_at: NaiveDateTime,
}

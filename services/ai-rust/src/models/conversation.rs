use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Conversation record from the database.
///
/// # Expected Behavior
///
/// Represents a single row from `ai.conversations`. The context field is a
/// JSONB object that can store arbitrary metadata about the conversation
/// such as topic, user preferences, or session state.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub context: Value,
    pub created_at: NaiveDateTime,
    pub last_message_at: NaiveDateTime,
}

/// Message record from the database.
///
/// # Expected Behavior
///
/// Represents a single row from `ai.messages`. Role is typically "user",
/// "assistant", or "system". Sources is a JSONB field containing the
/// RAG knowledge entries that informed the assistant's response.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub sources: Value,
    pub created_at: NaiveDateTime,
}

/// Conversation detail with all messages for the history endpoint.
///
/// # Expected Behavior
///
/// Combines the conversation metadata with its ordered messages.
/// Messages are sorted by `created_at` ascending.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct ConversationDetail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub context: Value,
    pub created_at: NaiveDateTime,
    pub last_message_at: NaiveDateTime,
    pub messages: Vec<Message>,
}

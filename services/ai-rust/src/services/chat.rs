use crate::config::Config;
use crate::errors::AiError;
use crate::models::chat::{ChatRequest, ChatResponse};
use crate::models::conversation::{ConversationDetail, Message};
use crate::services::embeddings;
use crate::services::llm::{self, LlmMessage};
use crate::services::rag;
use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Handle a chat request: create or continue a conversation, perform RAG, call LLM.
///
/// # Expected Behavior
///
/// If `req.conversation_id` is `None`, creates a new conversation in `ai.conversations`
/// with an empty context `{}`. If provided, loads the existing conversation and its
/// messages. Saves the user's message to `ai.messages`. Then performs RAG search
/// using the user's message as the query to find relevant knowledge entries. Builds
/// a system prompt with the RAG context, appends conversation history, and calls
/// the LLM. Saves the assistant's response to `ai.messages` with RAG sources,
/// updates the conversation's `last_message_at`, and returns a `ChatResponse`.
/// If `redis_conn` is `None`, embedding cache reads/writes are skipped and
/// embeddings are always computed fresh from the API.
///
/// # Errors
///
/// - `AiError::Database` if any database operation fails.
/// - `AiError::Llm` if the LLM API call fails.
/// - `AiError::Embedding` if the embedding generation fails.
/// - `AiError::NotFound` if `conversation_id` is provided but does not exist.
///
/// # Side Effects
///
/// - Reads from and writes to `ai.conversations` and `ai.messages` (database I/O).
/// - Reads from `ai.knowledge` via RAG search (database I/O).
/// - Calls the `OpenAI` embeddings API (network I/O) for RAG query embedding.
/// - Reads/writes Redis for embedding cache (skipped if `redis_conn` is `None`).
/// - Calls the configured LLM provider API (network I/O).
/// - Logs at INFO level on conversation creation and completion.
#[allow(clippy::too_many_lines)]
pub async fn handle_chat(
    pool: &PgPool,
    redis_conn: Option<redis::aio::MultiplexedConnection>,
    http_client: &reqwest::Client,
    config: &Config,
    req: &ChatRequest,
) -> Result<ChatResponse, AiError> {
    let conversation_id = if let Some(id) = req.conversation_id {
        let exists: bool = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM ai.conversations WHERE id = $1)",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(AiError::NotFound(format!("Conversation {id} not found")));
        }
        id
    } else {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO ai.conversations (id, user_id, context, created_at, last_message_at) \
             VALUES ($1, $2, $3, NOW(), NOW())",
        )
        .bind(id)
        .bind(req.user_id)
        .bind(serde_json::json!({}))
        .execute(pool)
        .await?;

        log::info!("Created new conversation {id}");
        id
    };

    let user_message_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO ai.messages (id, conversation_id, role, content, sources, created_at) \
         VALUES ($1, $2, 'user', $3, '[]'::jsonb, NOW())",
    )
    .bind(user_message_id)
    .bind(conversation_id)
    .bind(&req.message)
    .execute(pool)
    .await?;

    let previous_messages = sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, role, content, sources, created_at \
         FROM ai.messages \
         WHERE conversation_id = $1 AND id != $2 \
         ORDER BY created_at ASC",
    )
    .bind(conversation_id)
    .bind(user_message_id)
    .fetch_all(pool)
    .await?;

    let query_embedding =
        embeddings::get_embedding(config, http_client, redis_conn, &req.message).await?;
    let search_results = rag::search(pool, config, &query_embedding).await?;

    let sources_json: Value = serde_json::to_value(
        search_results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "title": r.title,
                    "similarity": r.similarity,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or(serde_json::json!([]));

    let context_text = if search_results.is_empty() {
        "No relevant knowledge base entries found.".to_string()
    } else {
        search_results
            .iter()
            .map(|r| format!("[{}] {}: {}", r.category, r.title, r.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    let mut llm_messages: Vec<LlmMessage> = vec![LlmMessage {
        role: "system".to_string(),
        content: format!(
            "You are an AI assistant for a hackathon platform. Use the following knowledge base context to inform your responses. If the context is not relevant, answer based on your general knowledge.\n\nKnowledge Base:\n{context_text}"
        ),
    }];

    for msg in &previous_messages {
        llm_messages.push(LlmMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    llm_messages.push(LlmMessage {
        role: "user".to_string(),
        content: req.message.clone(),
    });

    let assistant_content = llm::llm_chat(config, http_client, llm_messages).await?;

    let assistant_message_id = Uuid::new_v4();
    let now: NaiveDateTime = chrono::Utc::now().naive_utc();

    sqlx::query(
        "INSERT INTO ai.messages (id, conversation_id, role, content, sources, created_at) \
         VALUES ($1, $2, 'assistant', $3, $4, NOW())",
    )
    .bind(assistant_message_id)
    .bind(conversation_id)
    .bind(&assistant_content)
    .bind(&sources_json)
    .execute(pool)
    .await?;

    sqlx::query("UPDATE ai.conversations SET last_message_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await?;

    Ok(ChatResponse {
        conversation_id,
        message: assistant_content,
        sources: sources_json,
        created_at: now,
    })
}

/// Get a conversation with all its messages.
///
/// # Expected Behavior
///
/// Loads the conversation from `ai.conversations` and all its messages
/// from `ai.messages` ordered by `created_at` ascending. Returns a
/// `ConversationDetail` combining both.
///
/// # Errors
///
/// - `AiError::NotFound` if the conversation does not exist.
/// - `AiError::Database` if the database query fails.
///
/// # Side Effects
///
/// - Reads from `ai.conversations` and `ai.messages` (database I/O).
pub async fn get_conversation_history(
    pool: &PgPool,
    conversation_id: Uuid,
) -> Result<ConversationDetail, AiError> {
    let conversation = sqlx::query_as::<_, crate::models::conversation::Conversation>(
        "SELECT id, user_id, context, created_at, last_message_at \
         FROM ai.conversations WHERE id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AiError::NotFound(format!("Conversation {conversation_id} not found")))?;

    let messages = sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, role, content, sources, created_at \
         FROM ai.messages WHERE conversation_id = $1 ORDER BY created_at ASC",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;

    Ok(ConversationDetail {
        id: conversation.id,
        user_id: conversation.user_id,
        context: conversation.context,
        created_at: conversation.created_at,
        last_message_at: conversation.last_message_at,
        messages,
    })
}

/// Delete a conversation and all its messages.
///
/// # Expected Behavior
///
/// Deletes all messages in `ai.messages` for the given `conversation_id`,
/// then deletes the conversation from `ai.conversations`. Returns `Ok(())`
/// even if the conversation does not exist (idempotent deletion).
///
/// # Errors
///
/// - `AiError::Database` if the database deletion fails.
///
/// # Side Effects
///
/// - Deletes rows from `ai.messages` and `ai.conversations` (database write).
/// - Logs at INFO level on successful deletion.
pub async fn delete_conversation(pool: &PgPool, conversation_id: Uuid) -> Result<(), AiError> {
    sqlx::query("DELETE FROM ai.messages WHERE conversation_id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM ai.conversations WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await?;

    log::info!("Deleted conversation {conversation_id}");
    Ok(())
}

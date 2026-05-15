use crate::config::Config;
use crate::errors::AiError;
use md5::{Digest, Md5};
use redis::AsyncCommands;

/// Get the embedding vector for a text string, using Redis cache when available.
///
/// # Expected Behavior
///
/// Computes an MD5 hash of the input text and checks Redis for a cached embedding
/// under the key `embedding:{md5_hex}` if `redis_conn` is `Some`. If found, deserializes
/// the JSON array into a `Vec<f32>`. If not found or `redis_conn` is `None`, calls the
/// `OpenAI` embeddings API at `https://api.openai.com/v1/embeddings` with the configured
/// embeddings model, extracts the embedding from `data[0].embedding`, caches it in Redis
/// with a 7-day TTL (604800 seconds) if available, and returns it. Always uses `OpenAI`'s
/// embeddings API regardless of the `LLM_PROVIDER` setting.
///
/// # Errors
///
/// - `AiError::Embedding` if the `OpenAI` API response does not contain the
///   expected embedding data.
/// - `AiError::Http` if the HTTP request to `OpenAI` fails.
/// - `AiError::Redis` is propagated if Redis cache read/write fails (but
///   the function falls back to the API call on Redis errors).
///
/// # Side Effects
///
/// - Reads from Redis key `embedding:{md5_hex}` (cache lookup; skipped if `redis_conn` is `None`).
/// - Makes an HTTP POST request to `OpenAI` embeddings API on cache miss (network I/O).
/// - Writes to Redis key `embedding:{md5_hex}` with 604800-second TTL on API success
///   (skipped if `redis_conn` is `None`; logs warning on write failure).
/// - Logs at INFO level on cache hit or API call.
/// - Logs at WARN level on Redis cache errors (non-fatal, falls back to API).
pub async fn get_embedding(
    config: &Config,
    http_client: &reqwest::Client,
    mut redis_conn: Option<redis::aio::MultiplexedConnection>,
    text: &str,
) -> Result<Vec<f32>, AiError> {
    let hash = compute_md5(text);
    let cache_key = format!("embedding:{hash}");

    if let Some(ref mut conn) = redis_conn {
        match get_cached_embedding(conn, &cache_key).await {
            Ok(Some(embedding)) => {
                log::info!("Embedding cache hit for key {cache_key}");
                return Ok(embedding);
            }
            Ok(None) => {}
            Err(e) => {
                log::warn!("Redis cache read error: {e}. Falling back to API.");
            }
        }
    }

    log::info!("Calling OpenAI embeddings API for text (hash: {hash})");

    let body = serde_json::json!({
        "model": config.embeddings_model,
        "input": text,
    });

    let response = http_client
        .post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", config.openai_api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let resp_body: serde_json::Value = response.json().await?;

    let embedding_values = resp_body["data"][0]["embedding"]
        .as_array()
        .ok_or_else(|| AiError::Embedding("No embedding array in OpenAI response".to_string()))?;

    #[allow(clippy::cast_possible_truncation)]
    let embedding: Vec<f32> = embedding_values
        .iter()
        .filter_map(|v| v.as_f64().map(|f| f as f32))
        .collect();

    if embedding.is_empty() {
        return Err(AiError::Embedding(
            "Empty embedding vector from OpenAI".to_string(),
        ));
    }

    if let Some(ref mut conn) = redis_conn {
        if let Err(e) = cache_embedding(conn, &cache_key, &embedding).await {
            log::warn!("Failed to cache embedding: {e}");
        }
    }

    Ok(embedding)
}

/// Read a cached embedding from Redis.
///
/// # Expected Behavior
///
/// Attempts to read the key from Redis and deserialize the JSON value
/// as a `Vec<f32>`. Returns `Ok(None)` if the key does not exist.
/// Returns `Ok(Some(vec))` if the key exists and can be deserialized.
///
/// # Errors
///
/// - `AiError::Redis` if the Redis command fails.
/// - Deserialization errors are mapped to `AiError::Embedding`.
///
/// # Side Effects
///
/// - Reads from Redis (read-only).
async fn get_cached_embedding(
    redis_conn: &mut redis::aio::MultiplexedConnection,
    cache_key: &str,
) -> Result<Option<Vec<f32>>, AiError> {
    let cached: Option<String> = redis_conn.get(cache_key).await?;
    match cached {
        Some(json_str) => {
            let embedding: Vec<f32> =
                serde_json::from_str(&json_str).map_err(|e| AiError::Embedding(e.to_string()))?;
            Ok(Some(embedding))
        }
        None => Ok(None),
    }
}

/// Write an embedding to Redis cache with a 7-day TTL.
///
/// # Expected Behavior
///
/// Serializes the embedding as a JSON array string and writes it to Redis
/// with the given key. Sets a TTL of 604800 seconds (7 days).
///
/// # Errors
///
/// - `AiError::Redis` if the Redis command fails.
///
/// # Side Effects
///
/// - Writes to Redis key (write).
/// - Sets 604800-second TTL on the key (write).
async fn cache_embedding(
    redis_conn: &mut redis::aio::MultiplexedConnection,
    cache_key: &str,
    embedding: &[f32],
) -> Result<(), AiError> {
    let json_str =
        serde_json::to_string(embedding).map_err(|e| AiError::Embedding(e.to_string()))?;
    redis_conn
        .set_ex::<_, _, ()>(cache_key, json_str, 604_800)
        .await?;
    Ok(())
}

/// Compute the MD5 hex digest of a string.
///
/// # Expected Behavior
///
/// Returns a lowercase hex string of the MD5 hash of the input text.
///
/// # Errors
///
/// None. MD5 computation is infallible.
///
/// # Side Effects
///
/// None. Pure computation.
pub(crate) fn compute_md5(text: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    result
        .iter()
        .fold(String::with_capacity(result.len() * 2), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        })
}

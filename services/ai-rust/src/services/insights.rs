use crate::config::Config;
use crate::errors::AiError;
use crate::models::insights::{Insight, InsightGenerateRequest, InsightResponse};
use crate::services::llm::{self, LlmMessage};
use chrono::Utc;
use redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

/// Generate AI-powered insights about the hackathon knowledge base.
///
/// # Expected Behavior
///
/// Queries the knowledge base for a count of entries by category and a sample
/// of recent entries. Builds a prompt describing the knowledge base state and
/// sends it to the configured LLM provider. The system prompt instructs the LLM
/// to return a JSON array of insight objects. The response is parsed and cached
/// in Redis under `ai:insights:{period}:{date}` with a TTL of 86400 seconds (1 day)
/// if a Redis connection is available. If `redis_conn` is `None`, the cache write
/// is skipped and insights are still returned. Returns the generated insights.
///
/// # Errors
///
/// - `AiError::Database` if the knowledge base query fails.
/// - `AiError::Llm` if the LLM API call fails.
/// - `AiError::Http` if the HTTP request fails.
///
/// Returns an empty insights list (not an error) if the LLM response cannot
/// be parsed as JSON.
///
/// # Side Effects
///
/// - Reads from `ai.knowledge` (database I/O).
/// - Calls the configured LLM provider API (network I/O).
/// - Writes to Redis key `ai:insights:{period}:{date}` with 86400-second TTL
///   (only if `redis_conn` is `Some`; logs warning on write failure).
/// - Logs at INFO level on successful generation.
/// - Logs at WARN level on JSON parse failure or Redis write failure.
pub async fn generate_insights(
    pool: &PgPool,
    mut redis_conn: Option<redis::aio::MultiplexedConnection>,
    http_client: &reqwest::Client,
    config: &Config,
    req: &InsightGenerateRequest,
) -> Result<InsightResponse, AiError> {
    let category_counts: Vec<(String, i64)> = sqlx::query_as::<_, (String, i64)>(
        "SELECT category, COUNT(*) FROM ai.knowledge GROUP BY category ORDER BY COUNT(*) DESC",
    )
    .fetch_all(pool)
    .await?;

    let recent_titles: Vec<(String,)> = sqlx::query_as::<_, (String,)>(
        "SELECT title FROM ai.knowledge ORDER BY updated_at DESC LIMIT 10",
    )
    .fetch_all(pool)
    .await?;

    let categories_summary = category_counts
        .iter()
        .map(|(cat, count)| format!("{cat}: {count} entries"))
        .collect::<Vec<_>>()
        .join(", ");

    let recent_summary = recent_titles
        .iter()
        .map(|(title,)| format!("- {title}"))
        .collect::<Vec<_>>()
        .join("\n");

    let topic_str = req
        .topic
        .as_deref()
        .unwrap_or("general trends and patterns");

    let user_prompt = format!(
        "Period: {}\nTopic: {}\nKnowledge base categories: {}\nRecent entries:\n{}",
        req.period, topic_str, categories_summary, recent_summary
    );

    let messages = vec![
        LlmMessage {
            role: "system".to_string(),
            content: "You are an AI insights generator for a hackathon platform. Analyze the knowledge base state and generate actionable insights. Return a JSON array of objects, each with: summary (string), details (object with any relevant structured data). Return ONLY the JSON array, no other text.".to_string(),
        },
        LlmMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let response_text = llm::llm_chat(config, http_client, messages).await?;

    let raw_insights: Vec<serde_json::Value> =
        if let Ok(parsed) = serde_json::from_str(&response_text) {
            parsed
        } else {
            let cleaned = response_text
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            match serde_json::from_str(cleaned) {
                Ok(parsed) => parsed,
                Err(e) => {
                    log::warn!("Failed to parse LLM insights response as JSON: {e}");
                    Vec::new()
                }
            }
        };

    let now = Utc::now();
    let insights: Vec<Insight> = raw_insights
        .into_iter()
        .map(|raw| Insight {
            id: Uuid::new_v4().to_string(),
            period: req.period.clone(),
            summary: raw["summary"].as_str().unwrap_or("No summary").to_string(),
            details: raw["details"].clone(),
            generated_at: now.to_rfc3339(),
        })
        .collect();

    let cache_key = format!("ai:insights:{}:{}", req.period, now.format("%Y-%m-%d"));

    if let Some(ref mut conn) = redis_conn {
        if let Ok(json) = serde_json::to_string(&insights) {
            if let Err(e) = conn.set_ex::<_, _, ()>(&cache_key, json, 86400).await {
                log::warn!("Failed to cache insights: {e}");
            }
        }
    }

    log::info!(
        "Generated {} insights for period {}",
        insights.len(),
        req.period
    );

    Ok(InsightResponse { insights })
}

/// Get cached insights for a given period.
///
/// # Expected Behavior
///
/// Searches Redis for keys matching `ai:insights:{period}:*` and returns
/// the most recently cached insights for that period. If no cached insights
/// are found, or if `redis_conn` is `None`, returns an empty list.
///
/// # Errors
///
/// - `AiError::Redis` if the Redis command fails (but returns empty on
///   non-critical errors).
///
/// # Side Effects
///
/// - Reads from Redis keys matching `ai:insights:{period}:*` (read-only;
///   skipped if `redis_conn` is `None`).
/// - Logs at WARN level on Redis read errors.
pub async fn get_insights(
    mut redis_conn: Option<redis::aio::MultiplexedConnection>,
    period: &str,
) -> Result<InsightResponse, AiError> {
    let Some(ref mut conn) = redis_conn else {
        return Ok(InsightResponse {
            insights: Vec::new(),
        });
    };

    let pattern = format!("ai:insights:{period}:*");
    let keys: Vec<String> = match redis::cmd("KEYS").arg(&pattern).query_async(conn).await {
        Ok(k) => k,
        Err(e) => {
            log::warn!("Redis KEYS command failed: {e}");
            return Ok(InsightResponse {
                insights: Vec::new(),
            });
        }
    };

    if keys.is_empty() {
        return Ok(InsightResponse {
            insights: Vec::new(),
        });
    }

    let mut all_insights: Vec<Insight> = Vec::new();

    for key in &keys {
        if let Ok(Some(json_str)) = conn.get::<_, Option<String>>(key).await {
            if let Ok(insights) = serde_json::from_str::<Vec<Insight>>(&json_str) {
                all_insights.extend(insights);
            }
        }
    }

    Ok(InsightResponse {
        insights: all_insights,
    })
}

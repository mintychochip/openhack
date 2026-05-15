use crate::config::Config;
use crate::errors::AiError;
use crate::models::knowledge::KnowledgeSearchResult;
use sqlx::PgPool;

/// Perform a vector similarity search against the knowledge base using pgvector.
///
/// # Expected Behavior
///
/// Takes a query embedding vector and searches `ai.knowledge` using pgvector's
/// cosine distance operator `<=>`. Returns rows where cosine similarity
/// (1 - distance) is greater than or equal to `config.rag_similarity_threshold`,
/// ordered by closest match first, limited to `config.rag_top_k` results.
/// The embedding parameter is formatted as a string `[0.1,0.2,...]` and cast
/// to the `vector` type in the SQL query.
///
/// # Errors
///
/// - `AiError::Database` if the SQL query fails (e.g., pgvector not installed,
///   invalid embedding dimension, database unreachable).
///
/// # Side Effects
///
/// - Reads from `ai.knowledge` table (database I/O).
/// - Logs at DEBUG level with the number of results found.
pub async fn search(
    pool: &PgPool,
    config: &Config,
    query_embedding: &[f32],
) -> Result<Vec<KnowledgeSearchResult>, AiError> {
    let embedding_str = format!(
        "[{}]",
        query_embedding
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    let results = sqlx::query_as::<_, KnowledgeSearchResult>(
        "SELECT id, title, content, category, tags, \
         1 - (embedding <=> $1::vector) AS similarity \
         FROM ai.knowledge \
         WHERE 1 - (embedding <=> $1::vector) >= $2 \
         ORDER BY embedding <=> $1::vector \
         LIMIT $3",
    )
    .bind(&embedding_str)
    .bind(config.rag_similarity_threshold)
    .bind(config.rag_top_k)
    .fetch_all(pool)
    .await?;

    log::debug!("RAG search returned {} results", results.len());
    Ok(results)
}

use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::models::knowledge::{
    CreateKnowledgeRequest, Knowledge, KnowledgeQueryParams, UpdateKnowledgeRequest,
};
use crate::services::embeddings;

/// Create a new knowledge entry with auto-generated embedding.
///
/// # Expected Behavior
///
/// Accepts a JSON `CreateKnowledgeRequest` body. Generates an embedding from
/// the content using the `OpenAI` embeddings API (with Redis caching if available),
/// inserts the entry into `ai.knowledge` with the embedding as a pgvector, and returns
/// the created `Knowledge` object (without the embedding field).
///
/// # Errors
///
/// Returns 500 on database error or embedding generation failure.
///
/// # Side Effects
///
/// - Calls `OpenAI` embeddings API (network I/O) if not cached.
/// - Reads/writes Redis for embedding cache (skipped if Redis is unavailable).
/// - Inserts a row into `ai.knowledge` (database write).
pub async fn create_knowledge(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<redis::aio::MultiplexedConnection>>,
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    req: web::Json<CreateKnowledgeRequest>,
) -> HttpResponse {
    let req = req.into_inner();
    let redis = redis_conn.get_ref().clone();

    let embedding = match embeddings::get_embedding(
        config.get_ref(),
        http_client.get_ref(),
        redis,
        &req.content,
    )
    .await
    {
        Ok(e) => e,
        Err(e) => {
            log::error!("Embedding generation failed: {e}");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate embedding"
            }));
        }
    };

    let embedding_str = format!(
        "[{}]",
        embedding
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    let id = Uuid::new_v4();

    let result = sqlx::query_as::<_, Knowledge>(
        "INSERT INTO ai.knowledge (id, title, content, category, tags, embedding, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6::vector, NOW(), NOW()) \
         RETURNING id, title, content, category, tags, created_at, updated_at",
    )
    .bind(id)
    .bind(&req.title)
    .bind(&req.content)
    .bind(&req.category)
    .bind(&req.tags)
    .bind(&embedding_str)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(knowledge) => HttpResponse::Created().json(knowledge),
        Err(e) => {
            log::error!("Failed to create knowledge: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create knowledge entry"
            }))
        }
    }
}

/// List knowledge entries with optional filters and pagination.
///
/// # Expected Behavior
///
/// Accepts query parameters: category (exact match), search (ILIKE on title/content),
/// tags (comma-separated, array overlap), page (default 1), `page_size` (default 20, max 100).
/// Returns a paginated list of `Knowledge` entries ordered by `created_at` descending.
///
/// # Errors
///
/// Returns 500 on database error.
///
/// # Side Effects
///
/// - Reads from `ai.knowledge` (database I/O).
pub async fn list_knowledge(
    pool: web::Data<PgPool>,
    query: web::Query<KnowledgeQueryParams>,
) -> HttpResponse {
    let page = query.effective_page();
    let page_size = query.effective_page_size();
    let offset = (page - 1) * page_size;

    let parsed_tags: Option<Vec<String>> = query
        .tags
        .as_ref()
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|t: &Vec<String>| !t.is_empty());

    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        "SELECT id, title, content, category, tags, created_at, updated_at FROM ai.knowledge WHERE 1=1",
    );

    if let Some(ref category) = query.category {
        builder.push(" AND category = ");
        builder.push_bind(category.clone());
    }

    if let Some(ref search) = query.search {
        builder.push(" AND (title ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(" OR content ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(")");
    }

    if let Some(ref tags) = parsed_tags {
        builder.push(" AND tags && ");
        builder.push_bind(tags.clone());
    }

    builder.push(" ORDER BY created_at DESC LIMIT ");
    builder.push_bind(page_size);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let result = builder
        .build_query_as::<Knowledge>()
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(entries) => HttpResponse::Ok().json(entries),
        Err(e) => {
            log::error!("Failed to list knowledge: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list knowledge entries"
            }))
        }
    }
}

/// Get a single knowledge entry by ID.
///
/// # Expected Behavior
///
/// Returns the `Knowledge` entry if found. Returns 404 if the ID does not exist.
///
/// # Errors
///
/// Returns 404 if not found. Returns 500 on database error.
///
/// # Side Effects
///
/// - Reads from `ai.knowledge` (database I/O).
pub async fn get_knowledge(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> HttpResponse {
    let id = id.into_inner();

    let result = sqlx::query_as::<_, Knowledge>(
        "SELECT id, title, content, category, tags, created_at, updated_at \
         FROM ai.knowledge WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(knowledge)) => HttpResponse::Ok().json(knowledge),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Knowledge entry not found"
        })),
        Err(e) => {
            log::error!("Failed to get knowledge: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get knowledge entry"
            }))
        }
    }
}

/// Update a knowledge entry, re-embedding if content changed.
///
/// # Expected Behavior
///
/// Accepts a JSON `UpdateKnowledgeRequest` body with optional fields. If content
/// is provided, regenerates the embedding from the new content. Updates only the
/// provided fields in `ai.knowledge`. Returns the updated `Knowledge` entry.
///
/// # Errors
///
/// Returns 404 if the ID does not exist. Returns 500 on database or embedding error.
///
/// # Side Effects
///
/// - May call `OpenAI` embeddings API (network I/O) if content changed.
/// - May read/write Redis for embedding cache (skipped if Redis is unavailable).
/// - Updates a row in `ai.knowledge` (database write).
#[allow(clippy::too_many_lines)]
pub async fn update_knowledge(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<redis::aio::MultiplexedConnection>>,
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    id: web::Path<Uuid>,
    req: web::Json<UpdateKnowledgeRequest>,
) -> HttpResponse {
    let id = id.into_inner();
    let req = req.into_inner();

    let existing = sqlx::query_as::<_, Knowledge>(
        "SELECT id, title, content, category, tags, created_at, updated_at \
         FROM ai.knowledge WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;

    let existing = match existing {
        Ok(Some(k)) => k,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Knowledge entry not found"
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch knowledge for update: {e}");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch knowledge entry"
            }));
        }
    };

    let content_changed = req.content.is_some();
    let new_title = req.title.unwrap_or(existing.title);
    let new_content = req.content.unwrap_or(existing.content);
    let new_category = req.category.unwrap_or(existing.category);
    let new_tags = req.tags.unwrap_or(existing.tags);

    if content_changed {
        let redis = redis_conn.get_ref().clone();
        let embedding = match embeddings::get_embedding(
            config.get_ref(),
            http_client.get_ref(),
            redis,
            &new_content,
        )
        .await
        {
            Ok(e) => e,
            Err(e) => {
                log::error!("Embedding regeneration failed: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to regenerate embedding"
                }));
            }
        };

        let embedding_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );

        let result = sqlx::query_as::<_, Knowledge>(
            "UPDATE ai.knowledge SET title = $1, content = $2, category = $3, tags = $4, \
             embedding = $5::vector, updated_at = NOW() \
             WHERE id = $6 \
             RETURNING id, title, content, category, tags, created_at, updated_at",
        )
        .bind(&new_title)
        .bind(&new_content)
        .bind(&new_category)
        .bind(&new_tags)
        .bind(&embedding_str)
        .bind(id)
        .fetch_one(pool.get_ref())
        .await;

        match result {
            Ok(knowledge) => HttpResponse::Ok().json(knowledge),
            Err(e) => {
                log::error!("Failed to update knowledge with embedding: {e}");
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to update knowledge entry"
                }))
            }
        }
    } else {
        let result = sqlx::query_as::<_, Knowledge>(
            "UPDATE ai.knowledge SET title = $1, content = $2, category = $3, tags = $4, \
             updated_at = NOW() \
             WHERE id = $5 \
             RETURNING id, title, content, category, tags, created_at, updated_at",
        )
        .bind(&new_title)
        .bind(&new_content)
        .bind(&new_category)
        .bind(&new_tags)
        .bind(id)
        .fetch_one(pool.get_ref())
        .await;

        match result {
            Ok(knowledge) => HttpResponse::Ok().json(knowledge),
            Err(e) => {
                log::error!("Failed to update knowledge: {e}");
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to update knowledge entry"
                }))
            }
        }
    }
}

/// Delete a knowledge entry.
///
/// # Expected Behavior
///
/// Deletes the knowledge entry by ID. Returns 204 on success regardless of whether
/// the entry existed (idempotent deletion).
///
/// # Errors
///
/// Returns 500 on database error.
///
/// # Side Effects
///
/// - Deletes a row from `ai.knowledge` (database write).
pub async fn delete_knowledge(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> HttpResponse {
    let id = id.into_inner();

    let result = sqlx::query("DELETE FROM ai.knowledge WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Failed to delete knowledge: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete knowledge entry"
            }))
        }
    }
}

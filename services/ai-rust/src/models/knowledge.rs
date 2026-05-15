use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Knowledge base entry retrieved from the database.
///
/// # Expected Behavior
///
/// Represents a single row from `ai.knowledge` without the embedding vector.
/// Used for API responses and list queries. The embedding column is excluded
/// because pgvector's vector type is not natively supported by sqlx's `FromRow`.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Knowledge {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Request body for creating a new knowledge entry.
///
/// # Expected Behavior
///
/// Title and content are required. Category defaults to "general" if not provided.
/// Tags default to an empty vector. The embedding is auto-generated from the content.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateKnowledgeRequest {
    pub title: String,
    pub content: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_category() -> String {
    "general".to_string()
}

/// Request body for updating an existing knowledge entry.
///
/// # Expected Behavior
///
/// All fields are optional. Only provided fields are updated.
/// If content is provided, the embedding is regenerated automatically.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateKnowledgeRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Knowledge search result from pgvector similarity search.
///
/// # Expected Behavior
///
/// Includes a similarity score (0.0 to 1.0) from cosine similarity.
/// Higher values indicate closer matches.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct KnowledgeSearchResult {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub similarity: f64,
}

/// Query parameters for listing knowledge entries.
///
/// # Expected Behavior
///
/// All fields are optional. Category filters by exact match. Search filters
/// by ILIKE on title and content. Tags is a comma-separated list that filters
/// using `PostgreSQL` array overlap. Page defaults to 1, `page_size` defaults to 20.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeQueryParams {
    pub category: Option<String>,
    pub search: Option<String>,
    pub tags: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl KnowledgeQueryParams {
    /// Return the effective page number (1-indexed, minimum 1).
    ///
    /// # Expected Behavior
    ///
    /// Returns the page value if set and >= 1, otherwise returns 1.
    pub fn effective_page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    /// Return the effective page size (minimum 1, maximum 100).
    ///
    /// # Expected Behavior
    ///
    /// Returns the `page_size` value if set, clamped to [1, 100].
    /// Defaults to 20 if not set.
    pub fn effective_page_size(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a project.
///
/// # Expected Behavior
///
/// Maps directly to the `core.projects` table. The `name` column in the
/// database is exposed as `title` in the API. `tags` is a `PostgreSQL` text[]
/// array represented as `Vec<String>`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub demo_url: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub video_url: Option<String>,
    pub demo_file_id: Option<Uuid>,
    pub demo_file_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub submission_number: Option<i32>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub featured: bool,
    pub showcase_order: i32,
    pub favorite_count: i32,
    pub screening_status: Option<String>,
    pub screened_by: Option<Uuid>,
    pub screened_at: Option<DateTime<Utc>>,
    pub screening_notes: Option<String>,
}

/// Request body for creating a new project.
///
/// # Expected Behavior
///
/// `title` is required and maps to the `name` database column. `team_id`
/// is required. All other fields are optional.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectCreate {
    pub team_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub demo_url: Option<String>,
    pub category: Option<String>,
    pub video_url: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Request body for updating a project.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated.
/// `title` maps to the `name` database column.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub demo_url: Option<String>,
    pub category: Option<String>,
    pub video_url: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Response body for a single project.
///
/// # Expected Behavior
///
/// Contains all project fields. `name` from the database is exposed as
/// `title`. `status` defaults to "draft".
#[derive(Debug, Clone, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub demo_url: Option<String>,
    pub status: String,
    pub category: Option<String>,
    pub video_url: Option<String>,
    pub demo_file_id: Option<Uuid>,
    pub demo_file_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub submission_number: Option<i32>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub featured: bool,
    pub showcase_order: i32,
    pub favorite_count: i32,
    pub screening_status: Option<String>,
    pub is_favorite: Option<bool>,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            team_id: p.team_id,
            title: p.name,
            description: p.description,
            repository_url: p.repository_url,
            demo_url: p.demo_url,
            status: p.status.unwrap_or_else(|| "draft".to_string()),
            category: p.category,
            video_url: p.video_url,
            demo_file_id: p.demo_file_id,
            demo_file_url: p.demo_file_url,
            tags: p.tags,
            submission_number: p.submission_number,
            submitted_at: p.submitted_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
            featured: p.featured,
            showcase_order: p.showcase_order,
            favorite_count: p.favorite_count,
            screening_status: p.screening_status,
            is_favorite: None,
        }
    }
}

/// Response body for a paginated list of projects.
///
/// # Expected Behavior
///
/// Contains the list of project responses and a total count for pagination.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectResponse>,
    pub total: i64,
}

/// Request body for updating a project's status (admin).
///
/// # Expected Behavior
///
/// `status` is required and must be a valid status string.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectStatusUpdate {
    pub status: String,
}

/// Query parameters for listing projects.
///
/// # Expected Behavior
///
/// All fields are optional. `page` defaults to 1, `page_size` defaults to 20.
/// `status` and `category` are filters.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub category: Option<String>,
}

/// Query parameters for showcase/gallery.
#[derive(Debug, Clone, Deserialize)]
pub struct ShowcaseQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub category: Option<String>,
    pub sort: Option<String>,
}

/// Request to feature/unfeature a project.
#[derive(Debug, Clone, Deserialize)]
pub struct FeatureRequest {
    pub featured: bool,
    pub showcase_order: Option<i32>,
}

/// Request to favorite/unfavorite a project.
#[derive(Debug, Clone, Deserialize)]
pub struct FavoriteRequest {
    pub favorite: bool,
}

/// Response for favorite count.
#[derive(Debug, Clone, Serialize)]
pub struct FavoriteResponse {
    pub project_id: Uuid,
    pub favorite_count: i32,
    pub is_favorite: bool,
}

/// Request for screening a project.
#[derive(Debug, Clone, Deserialize)]
pub struct ScreenRequest {
    pub status: String,
    pub notes: Option<String>,
}

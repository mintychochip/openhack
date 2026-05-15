use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::CoreError;
use crate::models::project::{
    Project, ProjectCreate, ProjectListResponse, ProjectResponse, ProjectUpdate,
};
use crate::services::publisher;

pub struct ProjectService;

impl ProjectService {
    /// Create a new project.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the user is a member of the specified team, then inserts a
    /// new row into `core.projects`. The `title` from the request maps to the
    /// `name` column in the database. Status defaults to "draft".
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if the user is not a team member.
    /// Returns `CoreError::Database` on insert failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify membership (database read).
    /// - Inserts a row into `core.projects` (database write).
    pub async fn create_project(
        pool: &PgPool,
        user_id: Uuid,
        data: &ProjectCreate,
    ) -> Result<ProjectResponse, CoreError> {
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(data.team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return Err(CoreError::Forbidden(
                "Must be a team member to create a project".into(),
            ));
        }

        let project = sqlx::query_as::<_, Project>(
            "INSERT INTO core.projects (team_id, name, description, repository_url, demo_url, status, category, video_url, tags) VALUES ($1, $2, $3, $4, $5, 'draft', $6, $7, $8) RETURNING id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at",
        )
        .bind(data.team_id)
        .bind(&data.title)
        .bind(&data.description)
        .bind(&data.repository_url)
        .bind(&data.demo_url)
        .bind(&data.category)
        .bind(&data.video_url)
        .bind(&data.tags)
        .fetch_one(pool)
        .await?;

        log::info!("Created project {} for team {}", project.id, data.team_id);
        Ok(project.into())
    }

    /// Get a project by ID.
    ///
    /// # Expected Behavior
    ///
    /// Fetches the project with explicit column list. Returns
    /// `CoreError::NotFound` if the project does not exist.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no project exists with the given ID.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.projects` (database read).
    pub async fn get_project(pool: &PgPool, id: Uuid) -> Result<ProjectResponse, CoreError> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        Ok(project.into())
    }

    /// List projects with pagination and optional filters.
    ///
    /// # Expected Behavior
    ///
    /// Returns a paginated list of projects. Filters by `status` and/or
    /// `category` if provided. `page` defaults to 1, `page_size` defaults
    /// to 20.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.projects` (database read).
    pub async fn list_projects(
        pool: &PgPool,
        page: i64,
        page_size: i64,
        status: Option<&str>,
        category: Option<&str>,
    ) -> Result<ProjectListResponse, CoreError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);
        let offset = (page - 1) * page_size;

        let mut where_clauses = Vec::new();
        let mut param_idx = 1;

        if status.is_some() {
            where_clauses.push(format!("status = ${param_idx}"));
            param_idx += 1;
        }
        if category.is_some() {
            where_clauses.push(format!("category = ${param_idx}"));
            param_idx += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM core.projects {where_sql}");
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        if let Some(s) = status {
            count_query = count_query.bind(s);
        }
        if let Some(c) = category {
            count_query = count_query.bind(c);
        }
        let total = count_query.fetch_one(pool).await.unwrap_or(0);

        let list_sql = format!(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_sql, param_idx, param_idx + 1
        );

        let mut list_query = sqlx::query_as::<_, Project>(&list_sql);
        if let Some(s) = status {
            list_query = list_query.bind(s);
        }
        if let Some(c) = category {
            list_query = list_query.bind(c);
        }
        list_query = list_query.bind(page_size).bind(offset);

        let projects = list_query.fetch_all(pool).await?;

        Ok(ProjectListResponse {
            projects: projects.into_iter().map(Into::into).collect(),
            total,
        })
    }

    /// Update a project. Only team members can update.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the user is a member of the project's team, then updates
    /// only the provided fields. `title` maps to the `name` database column.
    /// If Redis is available, publishes a `project.updated` event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if the user is not a team member.
    /// Returns `CoreError::NotFound` if the project does not exist.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify membership (database read).
    /// - Updates row in `core.projects` (database write).
    /// - Publishes `project.updated` event to Redis, if available.
    pub async fn update_project(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        id: Uuid,
        user_id: Uuid,
        data: &ProjectUpdate,
    ) -> Result<ProjectResponse, CoreError> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(project.team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return Err(CoreError::Forbidden(
                "Must be a team member to update".into(),
            ));
        }

        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.title.is_some() {
            updates.push(format!("name = ${param_idx}"));
            param_idx += 1;
        }
        if data.description.is_some() {
            updates.push(format!("description = ${param_idx}"));
            param_idx += 1;
        }
        if data.repository_url.is_some() {
            updates.push(format!("repository_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.demo_url.is_some() {
            updates.push(format!("demo_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.category.is_some() {
            updates.push(format!("category = ${param_idx}"));
            param_idx += 1;
        }
        if data.video_url.is_some() {
            updates.push(format!("video_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.tags.is_some() {
            updates.push(format!("tags = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Ok(project.into());
        }

        let sql = format!(
            "UPDATE core.projects SET {} WHERE id = ${} RETURNING id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Project>(&sql);

        if let Some(ref v) = data.title {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.repository_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.demo_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.category {
            query = query.bind(v);
        }
        if let Some(ref v) = data.video_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.tags {
            query = query.bind(v);
        }

        let updated_project = query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        publisher::project_updated(redis_conn, &id.to_string(), &project.team_id.to_string()).await;
        log::info!("Updated project {id}");

        Ok(updated_project.into())
    }

    /// Delete a project. Only team members can delete.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the user is a member of the project's team, then deletes.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if not a team member.
    /// Returns `CoreError::NotFound` if the project does not exist.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify membership (database read).
    /// - Deletes from `core.projects` (database write, cascades).
    pub async fn delete_project(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), CoreError> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(project.team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return Err(CoreError::Forbidden(
                "Must be a team member to delete".into(),
            ));
        }

        sqlx::query("DELETE FROM core.projects WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        log::info!("Deleted project {id}");
        Ok(())
    }

    /// Submit a project for judging.
    ///
    /// # Expected Behavior
    ///
    /// Sets the project status to "submitted", increments the
    /// `submission_number`, and sets `submitted_at` to now. Only team
    /// members can submit. If Redis is available, publishes a
    /// `project.submitted` event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if not a team member.
    /// Returns `CoreError::NotFound` if the project does not exist.
    /// Returns `CoreError::Conflict` if the project is already submitted.
    ///
    /// # Side Effects
    ///
    /// - Updates `core.projects` status and submission fields (database write).
    /// - Publishes `project.submitted` event to Redis, if available.
    pub async fn submit_project(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<ProjectResponse, CoreError> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(project.team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return Err(CoreError::Forbidden(
                "Must be a team member to submit".into(),
            ));
        }

        if project.status.as_deref() == Some("submitted") {
            return Err(CoreError::Conflict("Project already submitted".into()));
        }

        let updated = sqlx::query_as::<_, Project>(
            "UPDATE core.projects SET status = 'submitted', submission_number = COALESCE(submission_number, 0) + 1, submitted_at = NOW() WHERE id = $1 RETURNING id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        publisher::project_submitted(redis_conn, &id.to_string(), &project.team_id.to_string())
            .await;
        log::info!("Submitted project {id}");

        Ok(updated.into())
    }

    /// Upload a demo file for a project by proxying to the media service.
    ///
    /// # Expected Behavior
    ///
    /// Receives a multipart upload, forwards it to the media service at
    /// `MEDIA_SERVICE_URL/api/media/upload`, parses the response for
    /// `file_id` and `url`, and updates the project's `demo_file_id` and
    /// `demo_file_url`. Only team members can upload demos.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if not a team member.
    /// Returns `CoreError::NotFound` if the project does not exist.
    /// Returns `CoreError::Internal` if the media service upload fails.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify membership (database read).
    /// - Sends HTTP multipart request to media service (network write).
    /// - Updates `core.projects` `demo_file_id` and `demo_file_url` (database write).
    pub async fn upload_demo(
        pool: &PgPool,
        media_url: &str,
        id: Uuid,
        user_id: Uuid,
        file_name: &str,
        file_data: Vec<u8>,
        content_type: &str,
    ) -> Result<ProjectResponse, CoreError> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(project.team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return Err(CoreError::Forbidden(
                "Must be a team member to upload demo".into(),
            ));
        }

        let client = reqwest::Client::new();
        let part = reqwest::multipart::Part::bytes(file_data)
            .file_name(file_name.to_string())
            .mime_str(content_type)
            .unwrap_or_else(|_| reqwest::multipart::Part::bytes(Vec::new()).file_name("file"));

        let form = reqwest::multipart::Form::new().part("file", part);

        let upload_url = format!("{}/api/media/upload", media_url.trim_end_matches('/'));
        let response = client
            .post(&upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("Media upload failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CoreError::Internal(format!("Media response parse failed: {e}")))?;

        let file_id: Option<Uuid> = body
            .get("id")
            .or_else(|| body.get("file_id"))
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        let file_url: Option<String> = body
            .get("url")
            .or_else(|| body.get("file_url"))
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let updated = sqlx::query_as::<_, Project>(
            "UPDATE core.projects SET demo_file_id = $1, demo_file_url = $2 WHERE id = $3 RETURNING id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at",
        )
        .bind(file_id)
        .bind(&file_url)
        .bind(id)
        .fetch_one(pool)
        .await?;

        log::info!("Uploaded demo for project {id}");
        Ok(updated.into())
    }

    /// Delete a project's demo file.
    ///
    /// # Expected Behavior
    ///
    /// Sets `demo_file_id` and `demo_file_url` to NULL. Only team members
    /// can delete demos.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if not a team member.
    /// Returns `CoreError::NotFound` if the project does not exist.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify membership (database read).
    /// - Updates `core.projects` (database write).
    pub async fn delete_demo(
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<ProjectResponse, CoreError> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(project.team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return Err(CoreError::Forbidden(
                "Must be a team member to delete demo".into(),
            ));
        }

        let updated = sqlx::query_as::<_, Project>(
            "UPDATE core.projects SET demo_file_id = NULL, demo_file_url = NULL WHERE id = $1 RETURNING id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        log::info!("Deleted demo for project {id}");
        Ok(updated.into())
    }

    /// Update a project's status (admin only).
    ///
    /// # Expected Behavior
    ///
    /// Updates the status field directly without membership checks.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the project does not exist.
    ///
    /// # Side Effects
    ///
    /// - Updates `core.projects` status (database write).
    pub async fn update_project_status(
        pool: &PgPool,
        id: Uuid,
        status: &str,
    ) -> Result<ProjectResponse, CoreError> {
        let updated = sqlx::query_as::<_, Project>(
            "UPDATE core.projects SET status = $1 WHERE id = $2 RETURNING id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at",
        )
        .bind(status)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Project".into(), id.to_string()))?;

        log::info!("Admin updated project {id} status to {status}");
        Ok(updated.into())
    }

    /// List all projects (admin endpoint, no filters).
    ///
    /// # Expected Behavior
    ///
    /// Returns all projects ordered by `created_at` descending with pagination.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.projects` (database read).
    pub async fn list_projects_admin(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ProjectResponse>, CoreError> {
        let projects = sqlx::query_as::<_, Project>(
            "SELECT id, team_id, name, description, repository_url, demo_url, created_at, updated_at, status, category, video_url, demo_file_id, demo_file_url, tags, submission_number, submitted_at FROM core.projects ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(projects.into_iter().map(Into::into).collect())
    }
}

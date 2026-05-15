use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::NotifyError;
use crate::models::announcement::{
    Announcement, CreateAnnouncementRequest, ListAnnouncementsQuery,
};

/// Create a new announcement in the database.
///
/// # Expected Behavior
///
/// Inserts a new row into `notify.announcements` with a generated UUID.
/// The `status` defaults to "draft" if not provided. Returns the
/// newly created announcement with all fields populated from the database.
///
/// # Errors
///
/// Returns `NotifyError::Database` if the insert query fails (e.g., constraint
/// violation, connection error).
///
/// # Side Effects
///
/// - Writes one row to `notify.announcements` table.
/// - Logs at INFO level on success.
pub async fn create_announcement(
    pool: &PgPool,
    req: &CreateAnnouncementRequest,
) -> Result<Announcement, NotifyError> {
    let status = req.status.as_deref().unwrap_or("draft");
    let announcement = sqlx::query_as::<_, Announcement>(
        "INSERT INTO notify.announcements (id, title, content, audience, channels, status, scheduled_at, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW()) \
         RETURNING id, title, content, audience, channels, status, scheduled_at, sent_at, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(&req.title)
    .bind(&req.content)
    .bind(&req.audience)
    .bind(&req.channels)
    .bind(status)
    .bind(req.scheduled_at)
    .fetch_one(pool)
    .await?;

    log::info!(
        "Created announcement {} with status '{}'",
        announcement.id,
        status
    );
    Ok(announcement)
}

/// List announcements with optional status filter and pagination.
///
/// # Expected Behavior
///
/// Returns a paginated list of announcements ordered by `created_at` descending.
/// If `status` is provided in the query, filters by exact status match.
/// Limit defaults to 50 (max 200), offset defaults to 0.
///
/// # Errors
///
/// Returns `NotifyError::Database` if the query fails.
///
/// # Side Effects
///
/// - Reads from `notify.announcements` table.
pub async fn list_announcements(
    pool: &PgPool,
    query: &ListAnnouncementsQuery,
) -> Result<Vec<Announcement>, NotifyError> {
    let limit = query.effective_limit();
    let offset = query.effective_offset();

    let announcements = match &query.status {
        Some(status) => {
            sqlx::query_as::<_, Announcement>(
                "SELECT id, title, content, audience, channels, status, scheduled_at, sent_at, created_at \
                 FROM notify.announcements WHERE status = $1 \
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Announcement>(
                "SELECT id, title, content, audience, channels, status, scheduled_at, sent_at, created_at \
                 FROM notify.announcements \
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(announcements)
}

/// Get a single announcement by ID.
///
/// # Expected Behavior
///
/// Returns the announcement with the given ID. Returns
/// `NotifyError::AnnouncementNotFound` if no announcement exists.
///
/// # Errors
///
/// Returns `NotifyError::AnnouncementNotFound` if the ID does not exist.
/// Returns `NotifyError::Database` on database failure.
///
/// # Side Effects
///
/// - Reads from `notify.announcements` table.
pub async fn get_announcement(pool: &PgPool, id: Uuid) -> Result<Announcement, NotifyError> {
    let announcement = sqlx::query_as::<_, Announcement>(
        "SELECT id, title, content, audience, channels, status, scheduled_at, sent_at, created_at \
         FROM notify.announcements WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    announcement.ok_or_else(|| NotifyError::AnnouncementNotFound(id.to_string()))
}

/// Delete an announcement by ID.
///
/// # Expected Behavior
///
/// Deletes the announcement with the given ID. Returns `Ok(())` on success.
/// Returns `NotifyError::AnnouncementNotFound` if no announcement with that
/// ID exists.
///
/// # Errors
///
/// Returns `NotifyError::AnnouncementNotFound` if the ID does not exist.
/// Returns `NotifyError::Database` on database failure.
///
/// # Side Effects
///
/// - Deletes a row from `notify.announcements` table.
/// - Logs at INFO level on success.
pub async fn delete_announcement(pool: &PgPool, id: Uuid) -> Result<(), NotifyError> {
    let result = sqlx::query("DELETE FROM notify.announcements WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(NotifyError::AnnouncementNotFound(id.to_string()));
    }

    log::info!("Deleted announcement {id}");
    Ok(())
}

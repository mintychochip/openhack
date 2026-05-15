use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::NotifyError;
use crate::models::notification::{
    CreateNotificationRequest, ListNotificationsQuery, UnreadCount, UserNotification,
};

/// Create a new in-app notification for a user.
///
/// # Expected Behavior
///
/// Inserts a new row into `notify.user_notifications` with a generated UUID.
/// `is_read` defaults to false. Returns the newly created notification with
/// all fields populated from the database.
///
/// # Errors
///
/// Returns `NotifyError::Database` if the insert fails (e.g., constraint
/// violation, connection error).
///
/// # Side Effects
///
/// - Writes one row to `notify.user_notifications` table.
/// - Logs at INFO level on success.
#[allow(dead_code)]
pub async fn create_notification(
    pool: &PgPool,
    req: &CreateNotificationRequest,
) -> Result<UserNotification, NotifyError> {
    let notification = sqlx::query_as::<_, UserNotification>(
        "INSERT INTO notify.user_notifications (id, user_id, type, title, message, data, action_url, is_read, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, false, NOW()) \
         RETURNING id, user_id, type, title, message, data, action_url, is_read, read_at, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(req.user_id)
    .bind(&req.r#type)
    .bind(&req.title)
    .bind(&req.message)
    .bind(&req.data)
    .bind(&req.action_url)
    .fetch_one(pool)
    .await?;

    log::info!(
        "Created notification {} for user {}",
        notification.id,
        req.user_id
    );
    Ok(notification)
}

/// Get notifications for a specific user with pagination.
///
/// # Expected Behavior
///
/// Returns a paginated list of notifications for the given `user_id`,
/// ordered by `created_at` descending. Limit defaults to 50 (max 200),
/// offset defaults to 0.
///
/// # Errors
///
/// Returns `NotifyError::Database` if the query fails.
///
/// # Side Effects
///
/// - Reads from `notify.user_notifications` table.
pub async fn get_user_notifications(
    pool: &PgPool,
    user_id: Uuid,
    query: &ListNotificationsQuery,
) -> Result<Vec<UserNotification>, NotifyError> {
    let limit = query.effective_limit();
    let offset = query.effective_offset();

    let notifications = sqlx::query_as::<_, UserNotification>(
        "SELECT id, user_id, type, title, message, data, action_url, is_read, read_at, created_at \
         FROM notify.user_notifications WHERE user_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(notifications)
}

/// Mark a specific notification as read.
///
/// # Expected Behavior
///
/// Sets `is_read` to true and `read_at` to `NOW()` for the notification with
/// the given ID, but only if it belongs to the specified `user_id`.
/// Returns `NotifyError::NotificationNotFound` if the notification does not
/// exist or does not belong to the user.
///
/// # Errors
///
/// Returns `NotifyError::NotificationNotFound` if the notification ID does
/// not exist for the given user. Returns `NotifyError::Database` on query failure.
///
/// # Side Effects
///
/// - Updates a row in `notify.user_notifications` table.
/// - Logs at INFO level on success.
pub async fn mark_notification_read(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
) -> Result<(), NotifyError> {
    let result = sqlx::query(
        "UPDATE notify.user_notifications SET is_read = true, read_at = NOW() \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(NotifyError::NotificationNotFound(id.to_string()));
    }

    log::info!("Marked notification {id} as read for user {user_id}");
    Ok(())
}

/// Mark all notifications as read for a specific user.
///
/// # Expected Behavior
///
/// Sets `is_read` to true and `read_at` to `NOW()` for all unread notifications
/// belonging to the specified `user_id`. Returns the count of updated notifications.
///
/// # Errors
///
/// Returns `NotifyError::Database` on query failure.
///
/// # Side Effects
///
/// - Updates multiple rows in `notify.user_notifications` table.
/// - Logs at INFO level with the count of updated rows.
pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> Result<u64, NotifyError> {
    let result = sqlx::query(
        "UPDATE notify.user_notifications SET is_read = true, read_at = NOW() \
         WHERE user_id = $1 AND is_read = false",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    log::info!(
        "Marked {} notifications as read for user {}",
        result.rows_affected(),
        user_id
    );
    Ok(result.rows_affected())
}

/// Get the count of unread notifications for a specific user.
///
/// # Expected Behavior
///
/// Returns the count of notifications where `user_id` matches and `is_read`
/// is false. Returns an `UnreadCount` struct with the count.
///
/// # Errors
///
/// Returns `NotifyError::Database` on query failure.
///
/// # Side Effects
///
/// - Reads from `notify.user_notifications` table.
pub async fn get_unread_count(pool: &PgPool, user_id: Uuid) -> Result<UnreadCount, NotifyError> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notify.user_notifications WHERE user_id = $1 AND is_read = false",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(UnreadCount { count: count.0 })
}

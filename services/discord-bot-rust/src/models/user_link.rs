use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserLink {
    pub id: Uuid,
    pub discord_user_id: String,
    pub openhack_user_id: Uuid,
    pub linked_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserLinkRequest {
    pub discord_user_id: String,
    pub openhack_user_id: Uuid,
}

/// Find a user link by Discord user ID.
///
/// # Expected Behavior
///
/// Queries `discord_bot.user_links` for a row with the given
/// `discord_user_id`. Returns the link if found, or `None`.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn find_by_discord_id(
    pool: &sqlx::PgPool,
    discord_user_id: &str,
) -> Result<Option<UserLink>, sqlx::Error> {
    sqlx::query_as::<_, UserLink>(
        "SELECT id, discord_user_id, openhack_user_id, linked_at \
         FROM discord_bot.user_links WHERE discord_user_id = $1",
    )
    .bind(discord_user_id)
    .fetch_optional(pool)
    .await
}

/// Find a user link by OpenHack user ID.
///
/// # Expected Behavior
///
/// Queries `discord_bot.user_links` for a row with the given
/// `openhack_user_id`. Returns the link if found, or `None`.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn find_by_openhack_id(
    pool: &sqlx::PgPool,
    openhack_user_id: Uuid,
) -> Result<Option<UserLink>, sqlx::Error> {
    sqlx::query_as::<_, UserLink>(
        "SELECT id, discord_user_id, openhack_user_id, linked_at \
         FROM discord_bot.user_links WHERE openhack_user_id = $1",
    )
    .bind(openhack_user_id)
    .fetch_optional(pool)
    .await
}

/// Find a user link by OpenHack user ID (within a transaction).
///
/// # Expected Behavior
///
/// Same as `find_by_openhack_id` but operates within an existing transaction.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table within the transaction.
pub async fn find_by_openhack_id_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    openhack_user_id: Uuid,
) -> Result<Option<UserLink>, sqlx::Error> {
    sqlx::query_as::<_, UserLink>(
        "SELECT id, discord_user_id, openhack_user_id, linked_at \
         FROM discord_bot.user_links WHERE openhack_user_id = $1",
    )
    .bind(openhack_user_id)
    .fetch_optional(tx.as_mut())
    .await
}

/// Create a new user link (within a transaction).
///
/// # Expected Behavior
///
/// Same as `create` but operates within an existing transaction.
///
/// # Errors
///
/// Returns `sqlx::Error` on constraint violation or database failure.
///
/// # Side Effects
///
/// - Writes one row to `discord_bot.user_links` table within the transaction.
pub async fn create_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    req: &CreateUserLinkRequest,
) -> Result<UserLink, sqlx::Error> {
    sqlx::query_as::<_, UserLink>(
        "INSERT INTO discord_bot.user_links (id, discord_user_id, openhack_user_id, linked_at) \
         VALUES ($1, $2, $3, NOW()) \
         RETURNING id, discord_user_id, openhack_user_id, linked_at",
    )
    .bind(Uuid::new_v4())
    .bind(&req.discord_user_id)
    .bind(req.openhack_user_id)
    .fetch_one(tx.as_mut())
    .await
}

/// Delete a user link by Discord user ID.
///
/// # Expected Behavior
///
/// Deletes the link with the given Discord user ID. Returns true if a row
/// was deleted, false if no matching link was found.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Deletes a row from `discord_bot.user_links` table.
pub async fn delete_by_discord_id(
    pool: &sqlx::PgPool,
    discord_user_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM discord_bot.user_links WHERE discord_user_id = $1")
        .bind(discord_user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List all user links with pagination.
///
/// # Expected Behavior
///
/// Returns paginated user links ordered by `linked_at` descending.
/// Limit defaults to 50, offset defaults to 0.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn list(
    pool: &sqlx::PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<UserLink>, sqlx::Error> {
    let limit = limit.clamp(1, 200);
    let offset = offset.max(0);

    sqlx::query_as::<_, UserLink>(
        "SELECT id, discord_user_id, openhack_user_id, linked_at \
         FROM discord_bot.user_links \
         ORDER BY linked_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

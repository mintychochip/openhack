use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelConfig {
    pub id: Uuid,
    pub event_type: String,
    pub channel_id: String,
    pub guild_id: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelConfigRequest {
    pub event_type: String,
    pub channel_id: String,
    pub guild_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChannelConfigRequest {
    pub channel_id: Option<String>,
    pub event_type: Option<String>,
}

/// Find channel configuration for a given event type.
///
/// # Expected Behavior
///
/// Queries `discord_bot.channel_config` for a row matching the given
/// `event_type` and `guild_id`. Returns the first match or `None`.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.channel_config` table.
pub async fn find_by_event_type(
    pool: &sqlx::PgPool,
    event_type: &str,
    guild_id: &str,
) -> Result<Option<ChannelConfig>, sqlx::Error> {
    sqlx::query_as::<_, ChannelConfig>(
        "SELECT id, event_type, channel_id, guild_id, created_at, updated_at \
         FROM discord_bot.channel_config \
         WHERE event_type = $1 AND guild_id = $2",
    )
    .bind(event_type)
    .bind(guild_id)
    .fetch_optional(pool)
    .await
}

/// List all channel configurations for a guild.
///
/// # Expected Behavior
///
/// Returns all channel config entries for the given guild, ordered by event_type.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.channel_config` table.
pub async fn list_by_guild(
    pool: &sqlx::PgPool,
    guild_id: &str,
) -> Result<Vec<ChannelConfig>, sqlx::Error> {
    sqlx::query_as::<_, ChannelConfig>(
        "SELECT id, event_type, channel_id, guild_id, created_at, updated_at \
         FROM discord_bot.channel_config \
         WHERE guild_id = $1 \
         ORDER BY event_type",
    )
    .bind(guild_id)
    .fetch_all(pool)
    .await
}

/// Create a new channel configuration.
///
/// # Expected Behavior
///
/// Inserts a new row into `discord_bot.channel_config`. Returns the created entry.
/// If a row with the same `event_type` and `channel_id` already exists, the
/// insert will fail with a unique constraint violation.
///
/// # Errors
///
/// Returns `sqlx::Error` on constraint violation or database failure.
///
/// # Side Effects
///
/// - Writes one row to `discord_bot.channel_config` table.
pub async fn create(
    pool: &sqlx::PgPool,
    req: &CreateChannelConfigRequest,
) -> Result<ChannelConfig, sqlx::Error> {
    sqlx::query_as::<_, ChannelConfig>(
        "INSERT INTO discord_bot.channel_config (id, event_type, channel_id, guild_id, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, NOW(), NOW()) \
         RETURNING id, event_type, channel_id, guild_id, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(&req.event_type)
    .bind(&req.channel_id)
    .bind(&req.guild_id)
    .fetch_one(pool)
    .await
}

/// Update a channel configuration.
///
/// # Expected Behavior
///
/// Updates only the provided fields (partial update). Sets `updated_at` to NOW().
/// Returns the updated entry.
///
/// # Errors
///
/// Returns `sqlx::Error` if the ID doesn't exist or on database failure.
///
/// # Side Effects
///
/// - Writes updated fields to `discord_bot.channel_config` table.
pub async fn update(
    pool: &sqlx::PgPool,
    id: Uuid,
    req: &UpdateChannelConfigRequest,
) -> Result<ChannelConfig, sqlx::Error> {
    let current = sqlx::query_as::<_, ChannelConfig>(
        "SELECT id, event_type, channel_id, guild_id, created_at, updated_at \
         FROM discord_bot.channel_config WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(current) = current else {
        return Err(sqlx::Error::RowNotFound);
    };

    let event_type = req.event_type.as_deref().unwrap_or(&current.event_type);
    let channel_id = req.channel_id.as_deref().unwrap_or(&current.channel_id);

    sqlx::query_as::<_, ChannelConfig>(
        "UPDATE discord_bot.channel_config SET event_type = $1, channel_id = $2, updated_at = NOW() \
         WHERE id = $3 \
         RETURNING id, event_type, channel_id, guild_id, created_at, updated_at",
    )
    .bind(event_type)
    .bind(channel_id)
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Delete a channel configuration by ID.
///
/// # Expected Behavior
///
/// Deletes the row. Returns 0 rows affected if not found.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Deletes a row from `discord_bot.channel_config` table.
pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM discord_bot.channel_config WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

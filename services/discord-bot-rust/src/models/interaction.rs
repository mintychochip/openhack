use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InteractionLog {
    pub id: Uuid,
    pub discord_user_id: String,
    pub interaction_type: String,
    pub command_name: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub response_status: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

/// Log an interaction to the database.
///
/// # Expected Behavior
///
/// Inserts a new row into `discord_bot.interactions` with the given details.
/// Returns the logged interaction on success.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Writes one row to `discord_bot.interactions` table.
pub async fn log_interaction(
    pool: &sqlx::PgPool,
    discord_user_id: &str,
    interaction_type: &str,
    command_name: Option<&str>,
    channel_id: Option<&str>,
    guild_id: Option<&str>,
    payload: Option<&serde_json::Value>,
    response_status: &str,
) -> Result<InteractionLog, sqlx::Error> {
    sqlx::query_as::<_, InteractionLog>(
        "INSERT INTO discord_bot.interactions \
         (id, discord_user_id, interaction_type, command_name, channel_id, guild_id, payload, response_status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW()) \
         RETURNING id, discord_user_id, interaction_type, command_name, channel_id, guild_id, payload, response_status, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(discord_user_id)
    .bind(interaction_type)
    .bind(command_name)
    .bind(channel_id)
    .bind(guild_id)
    .bind(payload)
    .bind(response_status)
    .fetch_one(pool)
    .await
}

/// List interaction logs with pagination.
///
/// # Expected Behavior
///
/// Returns paginated interaction logs ordered by `created_at` descending.
/// Limit defaults to 50 (max 200), offset defaults to 0.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.interactions` table.
pub async fn list(
    pool: &sqlx::PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<InteractionLog>, sqlx::Error> {
    let limit = limit.clamp(1, 200);
    let offset = offset.max(0);

    sqlx::query_as::<_, InteractionLog>(
        "SELECT id, discord_user_id, interaction_type, command_name, channel_id, guild_id, payload, response_status, created_at \
         FROM discord_bot.interactions \
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

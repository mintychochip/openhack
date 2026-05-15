use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LinkToken {
    pub id: Uuid,
    pub discord_user_id: String,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub claimed: bool,
    pub created_at: Option<NaiveDateTime>,
}

/// Create a new link token for Discord-to-OpenHack account linking.
///
/// # Expected Behavior
///
/// Generates a cryptographically random 32-byte hex token, inserts it into
/// `discord_bot.link_tokens` with the given Discord user ID and a 15-minute
/// expiration. Returns the created link token including the plaintext token
/// value (only returned once, never stored in plaintext in logs).
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Writes one row to `discord_bot.link_tokens` table.
pub async fn create(pool: &sqlx::PgPool, discord_user_id: &str) -> Result<LinkToken, sqlx::Error> {
    let token = generate_token();
    let expires_at = chrono::Utc::now()
        .naive_utc()
        .checked_add_signed(chrono::Duration::minutes(15))
        .unwrap_or_else(|| chrono::Utc::now().naive_utc());

    sqlx::query_as::<_, LinkToken>(
        "INSERT INTO discord_bot.link_tokens (id, discord_user_id, token, expires_at, claimed, created_at) \
         VALUES ($1, $2, $3, $4, FALSE, NOW()) \
         RETURNING id, discord_user_id, token, expires_at, claimed, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(discord_user_id)
    .bind(&token)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// Validate and claim a link token atomically (within a transaction).
///
///
/// # Expected Behavior
///
/// Same as `claim` but operates within an existing transaction.
///
/// # Errors
///
/// Returns `sqlx::Error` on database failure.
///
/// # Side Effects
///
/// - Updates `claimed` to TRUE on the matching token row (if any), within the transaction.
pub async fn claim_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token: &str,
) -> Result<Option<LinkToken>, sqlx::Error> {
    sqlx::query_as::<_, LinkToken>(
        "UPDATE discord_bot.link_tokens SET claimed = TRUE \
         WHERE token = $1 AND claimed = FALSE AND expires_at > NOW() \
         RETURNING id, discord_user_id, token, expires_at, claimed, created_at",
    )
    .bind(token)
    .fetch_optional(tx.as_mut())
    .await
}

/// Generate a cryptographically random hex token.
///
/// # Expected Behavior
///
/// Generates 32 random bytes using `rand::thread_rng()` and returns
/// their hex encoding (64-character string).
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None. Pure function (aside from RNG state).
fn generate_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

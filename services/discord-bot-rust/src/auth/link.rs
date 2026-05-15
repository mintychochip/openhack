use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::BotError;

/// Initiate a Discord-to-OpenHack account link.
///
/// # Expected Behavior
///
/// Creates a one-time link token for the given Discord user ID.
/// The token expires in 15 minutes. Returns the plaintext token
/// value so the caller can construct the linking URL. If the
/// Discord user already has a linked account, returns
/// `BotError::Validation`.
///
/// # Errors
///
/// Returns `BotError::Validation` if the Discord user is already linked.
/// Returns `BotError::Database` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
/// - Writes to `discord_bot.link_tokens` table.
pub async fn initiate_link(pool: &PgPool, discord_user_id: &str) -> Result<String, BotError> {
    let existing = crate::models::user_link::find_by_discord_id(pool, discord_user_id).await?;
    if existing.is_some() {
        return Err(BotError::Validation(
            "This Discord account is already linked.".into(),
        ));
    }

    let link_token = crate::models::link_token::create(pool, discord_user_id).await?;
    Ok(link_token.token)
}

/// Confirm a Discord-to-OpenHack account link.
///
/// # Expected Behavior
///
/// Validates the link token (must be unclaimed and unexpired), checks that
/// the OpenHack user is not already linked, and creates a `user_links`
/// record mapping the Discord user ID to the OpenHack user ID. All three
/// operations run inside a single database transaction to prevent race
/// conditions and token waste. If the OpenHack user is already linked,
/// the token is NOT consumed (the transaction rolls back).
///
/// # Errors
///
/// Returns `BotError::LinkTokenInvalid` if the token is invalid/expired/claimed.
/// Returns `BotError::Validation` if the OpenHack user is already linked.
/// Returns `BotError::Database` on database failure.
///
/// # Side Effects
///
/// - Updates `claimed` to TRUE on the token row (on success).
/// - Writes to `discord_bot.user_links` table (on success).
pub async fn confirm_link(
    pool: &PgPool,
    token: &str,
    openhack_user_id: Uuid,
) -> Result<(), BotError> {
    let mut tx = pool.begin().await?;

    let existing =
        crate::models::user_link::find_by_openhack_id_tx(&mut tx, openhack_user_id).await?;
    if existing.is_some() {
        return Err(BotError::Validation(
            "This OpenHack account is already linked to a Discord account.".into(),
        ));
    }

    let link_token = crate::models::link_token::claim_tx(&mut tx, token)
        .await?
        .ok_or(BotError::LinkTokenInvalid)?;

    crate::models::user_link::create_tx(
        &mut tx,
        &crate::models::user_link::CreateUserLinkRequest {
            discord_user_id: link_token.discord_user_id,
            openhack_user_id,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Check if a Discord user has a linked OpenHack account.
///
/// # Expected Behavior
///
/// Returns `Some(openhack_user_id)` if linked, `None` if not.
///
/// # Errors
///
/// Returns `BotError::Database` on database failure.
///
/// # Side Effects
///
/// - Reads from `discord_bot.user_links` table.
pub async fn get_link_status(
    pool: &PgPool,
    discord_user_id: &str,
) -> Result<Option<Uuid>, BotError> {
    let link = crate::models::user_link::find_by_discord_id(pool, discord_user_id).await?;
    Ok(link.map(|l| l.openhack_user_id))
}

/// Unlink a Discord account from its OpenHack account.
///
/// # Expected Behavior
///
/// Deletes the user link for the given Discord user ID. Returns
/// `BotError::UserLinkNotFound` if no link exists.
///
/// # Errors
///
/// Returns `BotError::UserLinkNotFound` if no link exists.
/// Returns `BotError::Database` on database failure.
///
/// # Side Effects
///
/// - Deletes a row from `discord_bot.user_links` table.
pub async fn unlink(pool: &PgPool, discord_user_id: &str) -> Result<(), BotError> {
    let deleted = crate::models::user_link::delete_by_discord_id(pool, discord_user_id).await?;
    if !deleted {
        return Err(BotError::UserLinkNotFound(discord_user_id.to_string()));
    }
    Ok(())
}

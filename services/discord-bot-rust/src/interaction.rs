use actix_web::{web, HttpRequest, HttpResponse};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Parsed context from a Discord interaction POST request.
///
/// # Expected Behavior
///
/// Contains all fields needed by command/component handlers: the interaction
/// ID and token (for followup messages), guild and channel IDs, the Discord
/// user ID of the invoker, and the raw `data` JSON object from the interaction.
/// For guild interactions the user is at `member.user.id`; for DM interactions
/// it is at `user.id`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InteractionContext {
    pub id: String,
    pub token: String,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub discord_user_id: String,
    pub data: Option<serde_json::Value>,
}

/// Actix-web route handler for the Discord Interactions Endpoint URL.
///
/// # Expected Behavior
///
/// Receives POST requests from Discord at `/interactions`. Verifies the
/// ED25519 signature on every request using the `X-Signature-Ed25519` and
/// `X-Signature-Timestamp` headers plus the raw body. If verification fails,
/// returns 401. If the body cannot be parsed as JSON, returns 400.
///
/// Dispatches based on interaction type:
/// - **1 (PING)**: returns `{"type": 1}` (PONG).
/// - **2 (APPLICATION_COMMAND)**: delegates to `commands::handle_interaction`.
/// - **3 (MESSAGE_COMPONENT)**: delegates to `components::buttons::handle_component`.
/// - **5 (MODAL_SUBMIT)**: delegates to `components::buttons::handle_modal`.
/// - Any other type returns a PONG as a safe fallback.
///
/// All handler return values are returned as JSON in the 200 response body,
/// which is the interaction response Discord expects.
///
/// # Errors
///
/// Returns 401 if signature verification fails. Returns 400 if the body is
/// not valid JSON. Handler errors are converted to ephemeral user-facing
/// messages by the command/component dispatchers.
///
/// # Side Effects
///
/// - Reads `X-Signature-Ed25519` and `X-Signature-Timestamp` headers.
/// - Delegates to command/component handlers which may read/write the
///   database and call external APIs.
pub async fn handle_interaction_endpoint(
    req: HttpRequest,
    body: web::Bytes,
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<crate::config::Config>,
    openhack_client: web::Data<crate::openhack_client::OpenHackClient>,
) -> HttpResponse {
    if !verify_discord_signature(&req, &body, &config.discord_public_key) {
        return HttpResponse::Unauthorized().finish();
    }

    let interaction: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse interaction body: {e}");
            return HttpResponse::BadRequest().finish();
        }
    };

    let interaction_type = interaction
        .get("type")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    match interaction_type {
        1 => HttpResponse::Ok().json(serde_json::json!({"type": 1})),
        2 => {
            let ctx = parse_interaction_context(&interaction);
            let response = crate::commands::handle_interaction(
                pool.get_ref(),
                config.get_ref(),
                openhack_client.get_ref(),
                &ctx,
            )
            .await;
            HttpResponse::Ok().json(response)
        }
        3 => {
            let ctx = parse_interaction_context(&interaction);
            let response = crate::components::buttons::handle_component(
                pool.get_ref(),
                config.get_ref(),
                openhack_client.get_ref(),
                &ctx,
            )
            .await;
            HttpResponse::Ok().json(response)
        }
        5 => {
            let ctx = parse_interaction_context(&interaction);
            let response = crate::components::buttons::handle_modal(
                pool.get_ref(),
                config.get_ref(),
                openhack_client.get_ref(),
                &ctx,
            )
            .await;
            HttpResponse::Ok().json(response)
        }
        _ => HttpResponse::Ok().json(serde_json::json!({"type": 1})),
    }
}

/// Verify an ED25519 signature given the raw hex-encoded components.
///
/// # Expected Behavior
///
/// Decodes the public key, signature, and timestamp from hex/UTF-8,
/// constructs the message as `timestamp_bytes + body`, and verifies
/// the Ed25519 signature. Returns `true` if valid, `false` if any step
/// fails (invalid hex, invalid key, bad signature).
///
/// # Errors
///
/// None. Returns `false` on any failure instead of propagating errors.
///
/// # Side Effects
///
/// None. Pure verification function.
pub(crate) fn verify_signature(
    sig_hex: &str,
    timestamp: &str,
    body: &[u8],
    public_key_hex: &str,
) -> bool {
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let pk_bytes = match hex::decode(public_key_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let pk_array: [u8; 32] = match pk_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };

    let verifying_key = match VerifyingKey::from_bytes(&pk_array) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = match Signature::from_slice(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut message = Vec::with_capacity(timestamp.len().saturating_add(body.len()));
    message.extend_from_slice(timestamp.as_bytes());
    message.extend_from_slice(body);

    verifying_key.verify(&message, &signature).is_ok()
}

/// Verify the ED25519 signature on a Discord interaction request.
///
/// # Expected Behavior
///
/// Reads the `X-Signature-Ed25519` (hex-encoded 64-byte signature) and
/// `X-Signature-Timestamp` headers, then delegates to `verify_signature`
/// with the extracted values and the raw body.
///
/// # Errors
///
/// None. Returns `false` on any failure instead of propagating errors.
///
/// # Side Effects
///
/// None. Pure verification function.
fn verify_discord_signature(req: &HttpRequest, body: &[u8], public_key_hex: &str) -> bool {
    let sig_hex = match req
        .headers()
        .get("X-Signature-Ed25519")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s,
        None => return false,
    };

    let timestamp = match req
        .headers()
        .get("X-Signature-Timestamp")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s,
        None => return false,
    };

    verify_signature(sig_hex, timestamp, body, public_key_hex)
}

/// Parse a Discord interaction JSON payload into an `InteractionContext`.
///
/// # Expected Behavior
///
/// Extracts `id`, `token`, `guild_id`, `channel_id`, the Discord user ID
/// (from `member.user.id` for guild interactions or `user.id` for DMs), and
/// the `data` field. Missing optional fields become `None` or empty strings.
///
/// # Errors
///
/// None. Returns defaults for missing fields.
///
/// # Side Effects
///
/// None.
fn parse_interaction_context(interaction: &serde_json::Value) -> InteractionContext {
    InteractionContext {
        id: interaction
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        token: interaction
            .get("token")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        guild_id: interaction
            .get("guild_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        channel_id: interaction
            .get("channel_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        discord_user_id: interaction
            .pointer("/member/user/id")
            .or_else(|| interaction.pointer("/user/id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        data: interaction.get("data").cloned(),
    }
}

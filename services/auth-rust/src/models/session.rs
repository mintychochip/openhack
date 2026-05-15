use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a session in the auth.sessions table.
///
/// # Expected Behavior
///
/// Maps 1:1 to the `auth.sessions` table. Each session is tied to a user
/// and stores a SHA-256 hash of the refresh token. Sessions can be
/// revoked individually or in bulk. The `expires_at` field determines
/// session validity; `revoked_at` indicates manual revocation.
///
/// # Errors
///
/// sqlx may return errors if the database row types do not match the struct
/// field types.
///
/// # Side Effects
///
/// None. This is a plain data struct.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing)]
    #[sqlx(rename = "refresh_token_hash")]
    pub _refresh_token_hash: String,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

/// JWT claims embedded in access tokens.
///
/// # Expected Behavior
///
/// Contains the subject (user ID), email, roles, issued-at timestamp (iat),
/// and expiration timestamp (exp). The `sub` field is the UUID of the user
/// as a string. Used for both encoding and decoding JWTs.
///
/// # Errors
///
/// None at construction time. Decoding may fail if the JWT is invalid or
/// expired, which is handled by the jsonwebtoken crate.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub roles: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

/// Request body for token refresh.
///
/// # Expected Behavior
///
/// Deserialized from JSON POST body. The `refresh_token` must be a valid UUID
/// string that was previously issued to the client.
///
/// # Errors
///
/// Returns 400 Bad Request if `refresh_token` is missing or invalid.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// Response containing access and refresh tokens.
///
/// # Expected Behavior
///
/// Returned on successful login or token refresh. `accessToken` is a JWT
/// with a short expiry (default 15 minutes). `refreshToken` is a UUID
/// string that the client stores securely. `expiresIn` is the number of
/// seconds until the access token expires.
///
/// # Errors
///
/// None.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
}

/// Response indicating MFA is required before completing login.
///
/// # Expected Behavior
///
/// Returned when a user with MFA enabled attempts to log in without
/// providing an MFA code. The client must submit the mfaToken along
/// with the MFA code in a subsequent login request.
///
/// # Errors
///
/// None.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize)]
pub struct MfaRequiredResponse {
    #[serde(rename = "mfaRequired")]
    pub mfa_required: bool,
    #[serde(rename = "mfaToken")]
    pub mfa_token: String,
}

/// Response returned after successful logout.
///
/// # Expected Behavior
///
/// Indicates the logout was successful and how many sessions were revoked.
///
/// # Errors
///
/// None.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize)]
pub struct LogoutResponse {
    #[serde(rename = "loggedOut")]
    pub logged_out: bool,
    #[serde(rename = "sessionsRevoked")]
    pub sessions_revoked: i64,
}

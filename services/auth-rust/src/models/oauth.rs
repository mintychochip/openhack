use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents an OAuth account link in the `auth.oauth_accounts` table.
///
/// # Expected Behavior
///
/// Maps 1:1 to the `auth.oauth_accounts` table. Each record links a user
/// to an external OAuth provider (GitHub, Google, Discord). The
/// (provider, `provider_user_id`) pair is unique, ensuring a single
/// provider account maps to at most one local user.
///
/// # Errors
///
/// sqlx may return errors if the database row types do not match.
///
/// # Side Effects
///
/// None. This is a plain data struct.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    #[serde(skip_serializing)]
    #[sqlx(rename = "access_token")]
    pub _access_token: Option<String>,
    #[serde(skip_serializing)]
    #[sqlx(rename = "refresh_token")]
    pub _refresh_token: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// Supported OAuth providers.
///
/// # Expected Behavior
///
/// Enum representing the three supported OAuth providers. Each variant
/// maps to the corresponding provider name stored in the `provider` column
/// of `auth.oauth_accounts`. Case-insensitive matching is used when
/// parsing from URL path parameters.
///
/// # Errors
///
/// Returns `AuthError::BadRequest` if an unsupported provider string is
/// provided.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthProvider {
    GitHub,
    Google,
    Discord,
}

impl OAuthProvider {
    /// Return the string identifier used in the database provider column.
    ///
    /// # Expected Behavior
    ///
    /// Returns "github", "google", or "discord" depending on the variant.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::GitHub => "github",
            OAuthProvider::Google => "google",
            OAuthProvider::Discord => "discord",
        }
    }

    /// Parse a provider string into an `OAuthProvider`.
    ///
    /// # Expected Behavior
    ///
    /// Case-insensitive parsing of "github", "google", or "discord".
    /// Returns None for unrecognized strings.
    ///
    /// # Errors
    ///
    /// None. Returns None for invalid providers.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "github" => Some(OAuthProvider::GitHub),
            "google" => Some(OAuthProvider::Google),
            "discord" => Some(OAuthProvider::Discord),
            _ => None,
        }
    }

    /// Return the OAuth authorization URL for this provider.
    ///
    /// # Expected Behavior
    ///
    /// Returns the provider's authorization endpoint URL used to redirect
    /// users for authentication.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn auth_url(&self) -> &'static str {
        match self {
            OAuthProvider::GitHub => "https://github.com/login/oauth/authorize",
            OAuthProvider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            OAuthProvider::Discord => "https://discord.com/api/oauth2/authorize",
        }
    }

    /// Return the OAuth token exchange URL for this provider.
    ///
    /// # Expected Behavior
    ///
    /// Returns the provider's token endpoint URL used to exchange an
    /// authorization code for access tokens.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn token_url(&self) -> &'static str {
        match self {
            OAuthProvider::GitHub => "https://github.com/login/oauth/access_token",
            OAuthProvider::Google => "https://oauth2.googleapis.com/token",
            OAuthProvider::Discord => "https://discord.com/api/oauth2/token",
        }
    }

    /// Return the user info API URL for this provider.
    ///
    /// # Expected Behavior
    ///
    /// Returns the provider's API endpoint for fetching the authenticated
    /// user's profile information.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn user_info_url(&self) -> &'static str {
        match self {
            OAuthProvider::GitHub => "https://api.github.com/user",
            OAuthProvider::Google => "https://www.googleapis.com/oauth2/v2/userinfo",
            OAuthProvider::Discord => "https://discord.com/api/v10/users/@me",
        }
    }

    /// Return the OAuth scope string for this provider.
    ///
    /// # Expected Behavior
    ///
    /// Returns the scope string required to access the user's email and
    /// basic profile information from the provider.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn scope(&self) -> &'static str {
        match self {
            OAuthProvider::GitHub => "user:email",
            OAuthProvider::Google => "openid email profile",
            OAuthProvider::Discord => "identify email",
        }
    }
}

/// User profile information extracted from an OAuth provider.
///
/// # Expected Behavior
///
/// Contains the provider-specific user ID, email, display name, and
/// avatar URL. Fields are Optional because not all providers return
/// all fields. Used during the find-or-create user flow.
///
/// # Errors
///
/// None.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUserInfo {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request body for password reset.
///
/// # Expected Behavior
///
/// Deserialized from JSON POST body. Contains the reset token (from the
/// email link) and the new password. The token must not be expired or
/// previously used.
///
/// # Errors
///
/// Returns 400 Bad Request if the token is invalid, expired, or already
/// used, or if the new password is too short.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

/// Represents a password reset token in `auth.password_reset_tokens`.
///
/// # Expected Behavior
///
/// Maps 1:1 to the `auth.password_reset_tokens` table. Each token is
/// tied to a user and can be used exactly once before it expires or
/// is marked as used.
///
/// # Errors
///
/// sqlx may return errors if the database row types do not match.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    #[sqlx(rename = "token")]
    pub _token: String,
    pub expires_at: Option<NaiveDateTime>,
    #[sqlx(rename = "used_at")]
    pub _used_at: Option<NaiveDateTime>,
    #[sqlx(rename = "created_at")]
    pub _created_at: Option<NaiveDateTime>,
}

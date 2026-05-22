use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a user in the auth.users table.
///
/// # Expected Behavior
///
/// Maps 1:1 to the `auth.users` table. Contains all user fields including
/// password hash, MFA settings, OAuth identifiers, and role assignments.
/// The `roles` field is a `PostgreSQL` text[] mapped to Vec<String>.
///
/// # Errors
///
/// sqlx may return errors if the database row types do not match the struct
/// field types (e.g., if the `roles` column is not `text[]`).
///
/// # Side Effects
///
/// None. This is a plain data struct.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub github_username: Option<String>,
    pub discord_id: Option<String>,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    pub sms_mfa_enabled: bool,
    pub sms_phone_number: Option<String>,
    pub roles: Vec<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub last_login_at: Option<NaiveDateTime>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<NaiveDateTime>,
    pub email_verification_token: Option<Uuid>,
    pub email_verification_token_expires_at: Option<NaiveDateTime>,
    pub deletion_requested_at: Option<NaiveDateTime>,
    pub deletion_scheduled_at: Option<NaiveDateTime>,
}

/// Public user profile returned in API responses.
///
/// # Expected Behavior
///
/// Contains only fields safe for API exposure. Excludes `password_hash`,
/// `mfa_secret`, and other sensitive fields. Used in /me, /users, and
/// registration responses.
///
/// # Errors
///
/// None. This is a plain data struct.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub github_username: Option<String>,
    pub discord_id: Option<String>,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub sms_mfa_enabled: bool,
    pub roles: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(rename = "lastLoginAt")]
    pub last_login_at: Option<NaiveDateTime>,
}

impl From<User> for UserProfile {
    /// Convert a full User record into a public `UserProfile`, stripping
    /// sensitive fields.
    ///
    /// # Expected Behavior
    ///
    /// Copies all non-sensitive fields from User to `UserProfile`. The
    /// `password_hash` and `mfa_secret` fields are excluded.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            github_username: user.github_username,
            discord_id: user.discord_id,
            email_verified: user.email_verified,
            mfa_enabled: user.mfa_enabled,
            sms_mfa_enabled: user.sms_mfa_enabled,
            roles: user.roles,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        }
    }
}

/// Request body for user registration.
///
/// # Expected Behavior
///
/// Deserialized from JSON POST body. Email must be a valid email format
/// (validated in the handler). Password must be at least 8 characters.
/// Name must not be empty.
///
/// # Errors
///
/// Returns 400 Bad Request if email is invalid, password is too short,
/// or name is empty.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

/// Response returned after successful registration.
///
/// # Expected Behavior
///
/// Contains the new user's ID, email, name, and creation timestamp.
/// Does not include sensitive fields.
///
/// # Errors
///
/// None.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(rename = "verificationToken", skip_serializing_if = "Option::is_none")]
    pub verification_token: Option<Uuid>,
}

/// Request body for user login.
///
/// # Expected Behavior
///
/// Deserialized from JSON POST body. Email and password are required.
/// `mfa_code` and `mfa_token` are provided when completing a second-factor
/// authentication step. `device_info` is optional metadata about the client.
///
/// # Errors
///
/// Returns 400 Bad Request if email or password is missing.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "mfaCode")]
    pub mfa_code: Option<String>,
    #[serde(rename = "mfaToken")]
    pub mfa_token: Option<String>,
    #[serde(rename = "deviceInfo")]
    pub device_info: Option<serde_json::Value>,
}

/// Request body for updating user profile.
///
/// # Expected Behavior
///
/// Deserialized from JSON PUT body. All fields are optional; only provided
/// fields will be updated. Email changes are not supported through this
/// endpoint (require a separate verification flow).
///
/// # Errors
///
/// None at deserialization time. Validation occurs in the handler.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub github_username: Option<String>,
    pub discord_id: Option<String>,
}

/// Request body for updating a user's role (admin only).
///
/// # Expected Behavior
///
/// Deserialized from JSON POST body. The `roles` field replaces the user's
/// entire role list. Valid roles include "participant", "judge", "admin",
/// "organizer".
///
/// # Errors
///
/// Returns 400 Bad Request if roles is empty or contains invalid values.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRoleRequest {
    pub roles: Vec<String>,
}

/// Paginated user list query parameters (admin only).
///
/// # Expected Behavior
///
/// Deserialized from query string. `limit` defaults to 20, max 100.
/// `offset` defaults to 0. `search` optionally filters by email or name.
///
/// # Errors
///
/// None at deserialization time. Limits are enforced in the handler.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Deserialize)]
pub struct UserListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

impl UserListQuery {
    /// Return the effective limit, clamped between 1 and 100, defaulting to 20.
    ///
    /// # Expected Behavior
    ///
    /// If limit is None, returns 20. If less than 1, returns 1.
    /// If greater than 100, returns 100. Otherwise returns the provided value.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    /// Return the effective offset, defaulting to 0 if None.
    ///
    /// # Expected Behavior
    ///
    /// If offset is None, returns 0. If negative, returns 0.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn effective_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

/// Paginated user list response (admin only).
///
/// # Expected Behavior
///
/// Contains a list of user profiles and total count for pagination.
///
/// # Errors
///
/// None.
///
/// # Side Effects
///
/// None.
#[derive(Debug, Clone, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserProfile>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

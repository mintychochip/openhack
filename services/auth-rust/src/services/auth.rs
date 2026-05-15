use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AuthError;
use crate::models::session::Claims;
use crate::models::user::{
    RegisterRequest, RegisterResponse, UpdateProfileRequest, User, UserProfile,
};

/// Hash a plaintext password using Argon2.
///
/// # Expected Behavior
///
/// Generates a random salt using the operating system's secure RNG,
/// then hashes the password with Argon2id (default parameters).
/// Returns the PHC string format hash which includes the algorithm
/// identifier, version, parameters, salt, and hash.
///
/// # Errors
///
/// Returns `AuthError::HashError` if the hashing operation fails
/// (e.g., password exceeds maximum length or contains null bytes).
///
/// # Side Effects
///
/// - Calls the operating system's CSPRNG for salt generation (read from /dev/urandom or equivalent).
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::HashError(e.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against a stored Argon2 hash.
///
/// # Expected Behavior
///
/// Parses the PHC-format hash string, then verifies the password
/// against it using Argon2id. Returns Ok(()) if the password matches,
/// or an error if it does not or if the hash string is malformed.
///
/// # Errors
///
/// Returns `AuthError::HashError` if the hash string cannot be parsed
/// or if the password does not match.
///
/// # Side Effects
///
/// None. Pure computation, no I/O.
pub fn verify_password(password: &str, hash: &str) -> Result<(), AuthError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| AuthError::HashError(e.to_string()))?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|e| AuthError::HashError(e.to_string()))
}

/// Create a JWT access token for a user.
///
/// # Expected Behavior
///
/// Encodes a JWT with HS256 signing containing the user's ID (sub),
/// email, roles, issued-at timestamp (iat), and expiration (exp).
/// The expiration is calculated from the current time plus the
/// configured `jwt_expiry_secs`. Returns the signed token string.
///
/// # Errors
///
/// Returns `AuthError::JwtError` if the JWT encoding fails (e.g.,
/// invalid secret, serialization failure).
///
/// # Side Effects
///
/// None. Pure computation, no I/O or state mutation.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn create_access_token(user: &User, config: &Config) -> Result<String, AuthError> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        roles: user.roles.clone(),
        exp: now + config.jwt_expiry_secs as usize,
        iat: now,
    };
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(AuthError::JwtError)
}

/// Decode and validate a JWT access token.
///
/// # Expected Behavior
///
/// Decodes the JWT using HS256 with the configured secret. Validates
/// the token's expiration and algorithm. Returns the Claims if valid,
/// or an error if the token is invalid, expired, or signed with a
/// different key.
///
/// # Errors
///
/// Returns `AuthError::JwtError` if the token is invalid, expired,
/// or has an invalid signature.
///
/// # Side Effects
///
/// None. Pure computation, no I/O.
pub fn validate_access_token(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(AuthError::JwtError)?;
    Ok(data.claims)
}

/// Register a new user.
///
/// # Expected Behavior
///
/// Validates the registration request (email format, password length >= 8,
/// name non-empty). Checks that the email is not already registered. Hashes
/// the password with Argon2. Inserts a new row into `auth.users` with
/// `email_verified = false`, `mfa_enabled = false`, and default roles
/// `ARRAY['participant']`. Returns the new user's ID, email, name, and
/// creation timestamp.
///
/// # Errors
///
/// Returns `AuthError::BadRequest` if the email is invalid, password is
/// too short, or name is empty. Returns `AuthError::Conflict` if the email
/// is already registered. Returns `AuthError::DatabaseError` on database
/// failures.
///
/// # Side Effects
///
/// - Reads from `auth.users` table (email uniqueness check).
/// - Writes to `auth.users` table (insert new user).
/// - Calls Argon2 for password hashing (CPU-intensive, uses OS RNG for salt).
pub async fn register_user(
    pool: &PgPool,
    req: &RegisterRequest,
) -> Result<RegisterResponse, AuthError> {
    if req.email.is_empty() || !req.email.contains('@') {
        return Err(AuthError::BadRequest("Invalid email address".to_string()));
    }
    if req.password.len() < 8 {
        return Err(AuthError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    if req.name.trim().is_empty() {
        return Err(AuthError::BadRequest("Name is required".to_string()));
    }

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM auth.users WHERE email = $1")
        .bind(&req.email)
        .fetch_one(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

    if existing > 0 {
        return Err(AuthError::Conflict("Email already registered".to_string()));
    }

    let password_hash = hash_password(&req.password)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO auth.users (id, email, password_hash, name, email_verified, mfa_enabled, roles, created_at, updated_at)
         VALUES ($1, $2, $3, $4, false, false, ARRAY['participant'], NOW(), NOW())",
    )
    .bind(user_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(req.name.trim())
    .execute(pool)
    .await
    .map_err(AuthError::DatabaseError)?;

    let created_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT created_at FROM auth.users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(AuthError::DatabaseError)?
            .flatten();

    Ok(RegisterResponse {
        id: user_id,
        email: req.email.clone(),
        name: req.name.trim().to_string(),
        created_at,
    })
}

/// Look up a user by email address.
///
/// # Expected Behavior
///
/// Queries `auth.users` by email (case-sensitive). Returns the full User
/// record if found, or None if no user exists with that email.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AuthError> {
    sqlx::query_as::<_, User>("SELECT id, email, password_hash, name, avatar_url, github_username, discord_id, email_verified, mfa_enabled, mfa_secret, sms_mfa_enabled, sms_phone_number, roles, created_at, updated_at, last_login_at FROM auth.users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(AuthError::DatabaseError)
}

/// Look up a user by ID.
///
/// # Expected Behavior
///
/// Queries `auth.users` by primary key. Returns the full User record
/// if found, or None if no user exists with that ID.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AuthError> {
    sqlx::query_as::<_, User>("SELECT id, email, password_hash, name, avatar_url, github_username, discord_id, email_verified, mfa_enabled, mfa_secret, sms_mfa_enabled, sms_phone_number, roles, created_at, updated_at, last_login_at FROM auth.users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AuthError::DatabaseError)
}

/// Update a user's profile fields.
///
/// # Expected Behavior
///
/// Updates only the fields provided in `req` (non-None fields). Builds
/// a dynamic UPDATE query that sets only the provided fields and always
/// updates `updated_at` to `NOW()`. Returns the updated `UserProfile`.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
/// Returns `AuthError::NotFound` if the user does not exist.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (update fields).
pub async fn update_user_profile(
    pool: &PgPool,
    user_id: Uuid,
    req: &UpdateProfileRequest,
) -> Result<UserProfile, AuthError> {
    let mut set_clauses = Vec::new();

    if req.name.is_some() {
        set_clauses.push("name = $N".to_string());
    }
    if req.avatar_url.is_some() {
        set_clauses.push("avatar_url = $N".to_string());
    }
    if req.github_username.is_some() {
        set_clauses.push("github_username = $N".to_string());
    }
    if req.discord_id.is_some() {
        set_clauses.push("discord_id = $N".to_string());
    }

    if set_clauses.is_empty() {
        let user = find_user_by_id(pool, user_id)
            .await?
            .ok_or_else(|| AuthError::NotFound("User not found".to_string()))?;
        return Ok(user.into());
    }

    let user = find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AuthError::NotFound("User not found".to_string()))?;

    let name = req.name.clone().unwrap_or(user.name);
    let avatar_url = req.avatar_url.clone().or(user.avatar_url);
    let github_username = req.github_username.clone().or(user.github_username);
    let discord_id = req.discord_id.clone().or(user.discord_id);

    sqlx::query(
        "UPDATE auth.users SET name = $1, avatar_url = $2, github_username = $3, discord_id = $4, updated_at = NOW() WHERE id = $5",
    )
    .bind(&name)
    .bind(&avatar_url)
    .bind(&github_username)
    .bind(&discord_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AuthError::DatabaseError)?;

    let updated = find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AuthError::NotFound("User not found".to_string()))?;

    Ok(updated.into())
}

/// Update a user's last login timestamp.
///
/// # Expected Behavior
///
/// Sets `last_login_at` to `NOW()` for the given user ID.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Writes to `auth.users` table.
pub async fn update_last_login(pool: &PgPool, user_id: Uuid) -> Result<(), AuthError> {
    sqlx::query("UPDATE auth.users SET last_login_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;
    Ok(())
}

/// Delete a user and all associated data.
///
/// # Expected Behavior
///
/// Deletes the user from `auth.users`. Cascading deletes handle
/// `auth.sessions`, `auth.oauth_accounts`, and `auth.password_reset_tokens`
/// (requires ON DELETE CASCADE in the schema). Returns Ok(()) on success.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Deletes from `auth.users`, `auth.sessions`, `auth.oauth_accounts`,
///   and `auth.password_reset_tokens` tables (cascade).
pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<(), AuthError> {
    sqlx::query("DELETE FROM auth.users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;
    Ok(())
}

/// Update a user's roles.
///
/// # Expected Behavior
///
/// Replaces the user's `roles` array with the provided list. Also
/// updates `updated_at` to `NOW()`. Returns the updated `UserProfile`.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
/// Returns `AuthError::NotFound` if the user does not exist.
///
/// # Side Effects
///
/// - Writes to `auth.users` table.
pub async fn update_user_roles(
    pool: &PgPool,
    user_id: Uuid,
    roles: &[String],
) -> Result<UserProfile, AuthError> {
    sqlx::query("UPDATE auth.users SET roles = $1, updated_at = NOW() WHERE id = $2")
        .bind(roles)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

    let user = find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AuthError::NotFound("User not found".to_string()))?;
    Ok(user.into())
}

/// List users with pagination and optional search.
///
/// # Expected Behavior
///
/// Returns paginated `UserProfile` records from `auth.users`. If `search`
/// is provided, filters by email or name using ILIKE. Returns total count
/// for pagination. Results are ordered by `created_at` descending.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Reads from `auth.users` table.
pub async fn list_users(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    search: Option<&str>,
) -> Result<(Vec<User>, i64), AuthError> {
    let (users, total) = if let Some(search) = search {
        let pattern = format!("%{}%", search.replace('%', "\\%").replace('_', "\\_"));
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM auth.users WHERE email ILIKE $1 OR name ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        let users = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, avatar_url, github_username, discord_id, email_verified, mfa_enabled, mfa_secret, sms_mfa_enabled, sms_phone_number, roles, created_at, updated_at, last_login_at FROM auth.users WHERE email ILIKE $1 OR name ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        (users, total)
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth.users")
            .fetch_one(pool)
            .await
            .map_err(AuthError::DatabaseError)?;

        let users = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, avatar_url, github_username, discord_id, email_verified, mfa_enabled, mfa_secret, sms_mfa_enabled, sms_phone_number, roles, created_at, updated_at, last_login_at FROM auth.users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        (users, total)
    };

    Ok((users, total))
}

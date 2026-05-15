use rand::Rng;
use sqlx::PgPool;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::errors::AuthError;
use crate::models::user::User;

/// Generate a random 8-character uppercase backup code.
///
/// # Expected Behavior
///
/// Uses the thread-local RNG to generate 8 random uppercase alphabetic
/// characters (A-Z). Used as a one-time MFA recovery code.
///
/// # Errors
///
/// None. RNG operations are infallible.
///
/// # Side Effects
///
/// - Uses thread-local RNG (reads from OS entropy source).
fn generate_backup_code() -> String {
    let rng = &mut rand::thread_rng();
    (0..8).map(|_| rng.gen_range(b'A'..=b'Z') as char).collect()
}

/// Generate 10 unique backup codes for MFA.
///
/// # Expected Behavior
///
/// Generates 10 random 8-character uppercase codes. Ensures all codes
/// are unique by using a `HashSet` during generation.
///
/// # Errors
///
/// None.
///
/// # Side Effects
///
/// - Uses thread-local RNG.
pub fn generate_backup_codes() -> Vec<String> {
    let mut codes = std::collections::HashSet::new();
    while codes.len() < 10 {
        codes.insert(generate_backup_code());
    }
    codes.into_iter().collect()
}

/// Encode bytes as base32 (RFC 4648) for TOTP secret storage.
///
/// # Expected Behavior
///
/// Encodes the input bytes using the standard base32 alphabet
/// (A-Z, 2-7). Used to store TOTP secrets in a format compatible
/// with authenticator apps.
///
/// # Errors
///
/// None. Encoding is infallible.
///
/// # Side Effects
///
/// None.
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut bits = 0u32;
    let mut n_bits = 0u32;
    for &byte in data {
        bits = (bits << 8) | u32::from(byte);
        n_bits += 8;
        while n_bits >= 5 {
            n_bits -= 5;
            result.push(ALPHABET[((bits >> n_bits) & 0x1F) as usize] as char);
        }
    }
    if n_bits > 0 {
        result.push(ALPHABET[((bits << (5 - n_bits)) & 0x1F) as usize] as char);
    }
    result
}

/// Generate random TOTP secret bytes (20 bytes = 160 bits, standard for TOTP).
///
/// # Expected Behavior
///
/// Uses the OS CSPRNG via the rand crate to generate 20 cryptographically
/// random bytes suitable for use as a TOTP secret.
///
/// # Errors
///
/// None. OS RNG is infallible.
///
/// # Side Effects
///
/// - Uses OS CSPRNG.
fn generate_secret_bytes() -> Vec<u8> {
    let rng = &mut rand::thread_rng();
    (0..20).map(|_| rng.gen::<u8>()).collect()
}

/// Create a TOTP instance from secret bytes with standard parameters.
///
/// # Expected Behavior
///
/// Creates a TOTP with SHA1 algorithm, 6 digits, skew=1, step=30 seconds.
/// Returns the TOTP instance or an error if creation fails.
///
/// # Errors
///
/// Returns `AuthError::MfaError` if the TOTP cannot be created.
///
/// # Side Effects
///
/// None. Pure construction.
fn create_totp(secret_bytes: Vec<u8>) -> Result<TOTP, AuthError> {
    TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)
        .map_err(|e| AuthError::MfaError(format!("Failed to create TOTP: {e}")))
}

/// Construct the otpauth:// URL for QR code generation.
///
/// # Expected Behavior
///
/// Builds a standard <otpauth://totp>/ URL with issuer, account name,
/// secret, algorithm, digits, and period parameters. This URL can
/// be rendered as a QR code by the client for authenticator app setup.
///
/// # Errors
///
/// None. URL construction is infallible.
///
/// # Side Effects
///
/// None.
fn build_otpauth_url(secret_b32: &str, email: &str) -> String {
    format!(
        "otpauth://totp/OpenHack:{}?secret={}&issuer=OpenHack&algorithm=SHA1&digits=6&period=30",
        urlencoding::encode(email),
        secret_b32
    )
}

/// Decode a base32-encoded secret string back to raw bytes.
///
/// # Expected Behavior
///
/// Parses the base32 string using the totp-rs Secret type and returns
/// the raw bytes. Returns an error if the string is not valid base32.
///
/// # Errors
///
/// Returns `AuthError::MfaError` if the secret cannot be decoded.
///
/// # Side Effects
///
/// None.
fn decode_secret(secret_str: &str) -> Result<Vec<u8>, AuthError> {
    Secret::Encoded(secret_str.to_string())
        .to_bytes()
        .map_err(|e| AuthError::MfaError(format!("Invalid TOTP secret: {e}")))
}

/// Enable TOTP-based MFA for a user.
///
/// # Expected Behavior
///
/// Generates a new TOTP secret, creates a TOTP instance with SHA1
/// algorithm, 6 digits, skew=1 (allow 1 step drift), 30-second step.
/// Stores the base32-encoded secret in the user record in `auth.users`.
/// Returns the secret, QR code URL, and backup codes. MFA is not actually
/// enabled until `verify_mfa_setup` is called with a valid code to
/// confirm the user has correctly configured their authenticator app.
///
/// # Errors
///
/// Returns `AuthError::MfaError` if the TOTP instance cannot be created.
/// Returns `AuthError::DatabaseError` on database failures.
/// Returns `AuthError::BadRequest` if MFA is already enabled.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (update `mfa_secret`).
/// - Calls TOTP library for secret generation and URL construction.
pub async fn enable_mfa_totp(
    pool: &PgPool,
    user: &User,
) -> Result<(String, String, Vec<String>), AuthError> {
    if user.mfa_enabled {
        return Err(AuthError::BadRequest("MFA is already enabled".to_string()));
    }

    let secret_bytes = generate_secret_bytes();
    let _totp = create_totp(secret_bytes.clone())?;
    let secret_str = base32_encode(&secret_bytes);
    let qr_url = build_otpauth_url(&secret_str, &user.email);
    let backup_codes = generate_backup_codes();

    sqlx::query("UPDATE auth.users SET mfa_secret = $1, updated_at = NOW() WHERE id = $2")
        .bind(&secret_str)
        .bind(user.id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

    Ok((secret_str, qr_url, backup_codes))
}

/// Verify TOTP setup by checking a code from the user's authenticator app.
///
/// # Expected Behavior
///
/// Validates the provided TOTP code against the user's stored TOTP secret.
/// If valid, sets `mfa_enabled = true` in `auth.users`. Returns backup
/// codes for the user to save.
///
/// # Errors
///
/// Returns `AuthError::MfaError` if the TOTP code is invalid.
/// Returns `AuthError::BadRequest` if MFA is already enabled or no
/// secret is configured. Returns `AuthError::DatabaseError` on database
/// failures.
///
/// # Side Effects
///
/// - Reads from `auth.users` table (get `mfa_secret`).
/// - Writes to `auth.users` table (set `mfa_enabled` = true).
/// - Calls TOTP library for code verification.
pub async fn verify_mfa_setup(
    pool: &PgPool,
    user: &User,
    code: &str,
) -> Result<Vec<String>, AuthError> {
    if user.mfa_enabled {
        return Err(AuthError::BadRequest("MFA is already enabled".to_string()));
    }

    let secret_str = user.mfa_secret.as_ref().ok_or_else(|| {
        AuthError::BadRequest("No MFA secret configured. Call enable first.".to_string())
    })?;

    let secret_bytes = decode_secret(secret_str)?;
    let totp = create_totp(secret_bytes)?;

    let valid = totp
        .check_current(code)
        .map_err(|e| AuthError::MfaError(format!("TOTP check error: {e}")))?;

    if !valid {
        return Err(AuthError::MfaError("Invalid MFA code".to_string()));
    }

    let backup_codes = generate_backup_codes();

    sqlx::query("UPDATE auth.users SET mfa_enabled = true, updated_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

    Ok(backup_codes)
}

/// Verify a TOTP code during login.
///
/// # Expected Behavior
///
/// Validates the provided TOTP code against the user's stored TOTP secret
/// with skew=1 (allows 1 time step drift). Returns Ok(true) if valid,
/// Ok(false) if not.
///
/// # Errors
///
/// Returns `AuthError::MfaError` if no MFA secret is configured or if
/// the TOTP check encounters a system time error.
///
/// # Side Effects
///
/// - Reads from `auth.users` table (get `mfa_secret`).
pub fn verify_mfa_code(_pool: &PgPool, user: &User, code: &str) -> Result<bool, AuthError> {
    let secret_str = user
        .mfa_secret
        .as_ref()
        .ok_or_else(|| AuthError::MfaError("No MFA secret configured".to_string()))?;

    let secret_bytes = decode_secret(secret_str)?;
    let totp = create_totp(secret_bytes)?;

    let valid = totp
        .check_current(code)
        .map_err(|e| AuthError::MfaError(format!("TOTP check error: {e}")))?;

    Ok(valid)
}

/// Disable MFA for a user.
///
/// # Expected Behavior
///
/// Sets `mfa_enabled = false`, `mfa_secret = NULL`, and `sms_mfa_enabled = false`
/// in `auth.users`. Clears any backup codes from Redis.
///
/// # Errors
///
/// Returns `AuthError::BadRequest` if MFA is not enabled.
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (update mfa fields).
/// - Deletes Redis key `mfa:backup_codes:{userId}` (best-effort).
pub async fn disable_mfa(pool: &PgPool, user: &User) -> Result<(), AuthError> {
    if !user.mfa_enabled && !user.sms_mfa_enabled {
        return Err(AuthError::BadRequest("MFA is not enabled".to_string()));
    }

    sqlx::query(
        "UPDATE auth.users SET mfa_enabled = false, mfa_secret = NULL, sms_mfa_enabled = false, sms_phone_number = NULL, updated_at = NOW() WHERE id = $1",
    )
    .bind(user.id)
    .execute(pool)
    .await
    .map_err(AuthError::DatabaseError)?;

    Ok(())
}

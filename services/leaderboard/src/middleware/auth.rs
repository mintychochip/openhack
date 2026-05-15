pub use openhack_common::auth::{AuthUser, JwtSecret};

use actix_web::web;

/// Extract and validate a Bearer token from the Authorization header,
/// then decode the JWT and return the authenticated user.
///
/// # Expected Behavior
///
/// Reads the `Authorization` header from the request. Expects the format
/// `Bearer <token>`. Decodes the JWT using HS256 with the service's
/// `JWT_SECRET`. Returns `AuthUser` on success or a `LeaderboardError::Unauthorized`
/// on failure (missing header, malformed token, invalid signature, expired token).
///
/// # Errors
///
/// Returns `LeaderboardError::Unauthorized` if:
/// - The `Authorization` header is missing
/// - The header does not start with "Bearer "
/// - The JWT cannot be decoded (invalid signature, expired, malformed)
///
/// # Side Effects
///
/// - Reads the `Authorization` header from the incoming request (read-only).
/// - Reads the `JWT_SECRET` from app data (read-only).
pub fn extract_auth_user(
    req: &actix_web::HttpRequest,
    jwt_secret: &str,
) -> Result<AuthUser, crate::errors::LeaderboardError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            crate::errors::LeaderboardError::Unauthorized("Missing Authorization header".into())
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        crate::errors::LeaderboardError::Unauthorized("Invalid Authorization header format".into())
    })?;

    let token_data = jsonwebtoken::decode::<openhack_common::auth::Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|e| crate::errors::LeaderboardError::Unauthorized(format!("Invalid token: {e}")))?;

    let Ok(user_id) = uuid::Uuid::parse_str(&token_data.claims.sub) else {
        return Err(crate::errors::LeaderboardError::Unauthorized(
            "Invalid token subject".into(),
        ));
    };

    Ok(AuthUser {
        user_id,
        email: token_data.claims.email,
        roles: token_data.claims.roles,
    })
}

/// Check if the authenticated user has the "admin" or "organizer" role.
///
/// # Expected Behavior
///
/// Returns `Ok(())` if the user's roles contain "admin" or "organizer".
/// Returns `LeaderboardError::Forbidden` if none of these roles are present.
///
/// # Errors
///
/// Returns `LeaderboardError::Forbidden` if the user lacks the required role.
///
/// # Side Effects
///
/// None. Pure role check with no I/O.
pub fn require_admin_role(user: &AuthUser) -> Result<(), crate::errors::LeaderboardError> {
    if user.is_admin_or_organizer() {
        Ok(())
    } else {
        Err(crate::errors::LeaderboardError::Forbidden(
            "Admin or organizer role required".into(),
        ))
    }
}

/// Helper to extract auth user from a request using app data JWT secret.
///
/// # Expected Behavior
///
/// Retrieves the `JWT_SECRET` from the app's `web::Data<JwtSecret>`, then delegates to
/// `extract_auth_user` for token parsing and validation.
///
/// # Errors
///
/// Returns `LeaderboardError::Unauthorized` if the JWT secret is unavailable
/// or if token extraction/validation fails.
///
/// # Side Effects
///
/// - Reads the `JWT_SECRET` from app data (read-only).
/// - Reads the `Authorization` header from the incoming request (read-only).
pub fn get_auth_user(
    req: &actix_web::HttpRequest,
) -> Result<AuthUser, crate::errors::LeaderboardError> {
    if let Some(internal_token) = check_lambda_internal_token(req) {
        log::info!("Authenticated via Lambda internal token");
        return Ok(internal_token);
    }

    let jwt_secret = req.app_data::<web::Data<JwtSecret>>().ok_or_else(|| {
        crate::errors::LeaderboardError::Internal("JWT secret not configured".into())
    })?;

    extract_auth_user(req, &jwt_secret.0)
}

/// Validate a Lambda internal token against the expected environment variable.
///
/// # Expected Behavior
///
/// Reads `LAMBDA_INTERNAL_TOKEN` from the environment. If unset or empty,
/// returns `None`. Compares `provided_token` against the expected value.
/// If they match, returns `AuthUser` with nil UUID `user_id` and admin role.
/// If they do not match, returns `None` and logs a warning.
///
/// # Errors
///
/// Returns `None` (not an error) if the env var is missing/empty, the
/// provided token is `None`, or the tokens do not match.
///
/// # Side Effects
///
/// - Reads `LAMBDA_INTERNAL_TOKEN` environment variable (read).
/// - Logs at WARN level if a token is provided but does not match.
pub(crate) fn validate_lambda_internal_token(provided_token: Option<&str>) -> Option<AuthUser> {
    let expected = std::env::var("LAMBDA_INTERNAL_TOKEN").ok()?;
    if expected.is_empty() {
        return None;
    }
    let provided = provided_token?;
    if provided == expected {
        Some(AuthUser {
            user_id: uuid::Uuid::nil(),
            email: "internal@lambda".into(),
            roles: vec!["admin".into()],
        })
    } else {
        log::warn!("Invalid Lambda internal token provided");
        None
    }
}

fn check_lambda_internal_token(req: &actix_web::HttpRequest) -> Option<AuthUser> {
    let provided = req
        .headers()
        .get("X-Lambda-Internal-Token")
        .and_then(|v| v.to_str().ok());
    validate_lambda_internal_token(provided)
}

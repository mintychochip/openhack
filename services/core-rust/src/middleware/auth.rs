pub use openhack_common::auth::{AuthUser, JwtSecret};

use actix_web::web;

/// Extract and validate a Bearer token from the Authorization header,
/// then decode the JWT and return the authenticated user.
///
/// # Expected Behavior
///
/// Reads the `Authorization` header from the request. Expects the format
/// `Bearer <token>`. Decodes the JWT using HS256 with the service's
/// `JWT_SECRET`. Returns `AuthUser` on success or a `CoreError::Unauthorized`
/// on failure (missing header, malformed token, invalid signature, expired token).
///
/// # Errors
///
/// Returns `CoreError::Unauthorized` if:
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
) -> Result<AuthUser, crate::errors::CoreError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            crate::errors::CoreError::Unauthorized("Missing Authorization header".into())
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        crate::errors::CoreError::Unauthorized("Invalid Authorization header format".into())
    })?;

    let token_data = jsonwebtoken::decode::<openhack_common::auth::Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|e| crate::errors::CoreError::Unauthorized(format!("Invalid token: {e}")))?;

    let Ok(user_id) = uuid::Uuid::parse_str(&token_data.claims.sub) else {
        return Err(crate::errors::CoreError::Unauthorized(
            "Invalid token subject".into(),
        ));
    };

    Ok(AuthUser {
        user_id,
        email: token_data.claims.email,
        roles: token_data.claims.roles,
    })
}

/// Extract the authenticated user from a request, returning 401 if auth fails.
///
/// # Expected Behavior
///
/// First checks for `X-Lambda-Internal-Token` header. If present and matches
/// the `LAMBDA_INTERNAL_TOKEN` environment variable, returns a synthetic admin
/// `AuthUser` with `user_id` set to nil UUID. Otherwise, retrieves `JWT_SECRET`
/// from app data and delegates to `extract_auth_user`.
///
/// # Errors
///
/// Returns `CoreError::Unauthorized` if neither internal token nor valid JWT
/// is provided. Returns `CoreError::Internal` if JWT secret is unavailable.
///
/// # Side Effects
///
/// - Reads the `LAMBDA_INTERNAL_TOKEN` environment variable (read-only).
/// - Reads the `X-Lambda-Internal-Token` header from the request (read-only).
/// - Reads the `JWT_SECRET` from app data (read-only).
pub fn get_auth_user(req: &actix_web::HttpRequest) -> Result<AuthUser, crate::errors::CoreError> {
    if let Some(internal_token) = check_lambda_internal_token(req) {
        log::info!("Authenticated via Lambda internal token");
        return Ok(internal_token);
    }

    let jwt_secret = req
        .app_data::<web::Data<JwtSecret>>()
        .ok_or_else(|| crate::errors::CoreError::Internal("JWT secret not configured".into()))?;

    extract_auth_user(req, &jwt_secret.0)
}

/// Check if the request carries a valid Lambda internal token.
///
/// # Expected Behavior
///
/// Reads the `X-Lambda-Internal-Token` header and the `LAMBDA_INTERNAL_TOKEN`
/// environment variable. If both are present and match, returns `Some(AuthUser)`
/// with admin role and nil UUID `user_id`. Otherwise returns `None`.
///
/// # Errors
///
/// None. Returns `Option<AuthUser>`.
///
/// # Side Effects
///
/// - Reads the `LAMBDA_INTERNAL_TOKEN` environment variable (read-only).
/// - Reads the `X-Lambda-Internal-Token` header (read-only).
pub(crate) fn check_lambda_internal_token(req: &actix_web::HttpRequest) -> Option<AuthUser> {
    let expected = std::env::var("LAMBDA_INTERNAL_TOKEN").ok()?;
    if expected.is_empty() {
        return None;
    }
    let provided = req
        .headers()
        .get("X-Lambda-Internal-Token")
        .and_then(|v| v.to_str().ok())?;
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

/// Extract the authenticated user from a request, returning None if auth fails.
///
/// # Expected Behavior
///
/// Attempts to extract the JWT and decode it. Returns `Some(AuthUser)` on
/// success, `None` on any failure (missing header, invalid token, etc.).
/// Used for optional-auth endpoints.
///
/// # Errors
///
/// None. All errors are silently ignored and None is returned.
///
/// # Side Effects
///
/// - Reads the `JWT_SECRET` from app data (read-only).
/// - Reads the `Authorization` header from the incoming request (read-only).
pub fn get_optional_user(req: &actix_web::HttpRequest) -> Option<AuthUser> {
    get_auth_user(req).ok()
}

/// Check if the authenticated user has the "admin" role.
///
/// # Expected Behavior
///
/// Returns `Ok(())` if the user's roles contain "admin".
/// Returns `CoreError::Forbidden` if "admin" is not present.
///
/// # Errors
///
/// Returns `CoreError::Forbidden` if the user lacks the required role.
///
/// # Side Effects
///
/// None. Pure role check with no I/O.
pub fn require_admin(user: &AuthUser) -> Result<(), crate::errors::CoreError> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(crate::errors::CoreError::Forbidden(
            "Admin role required".into(),
        ))
    }
}

/// Check if the authenticated user has "admin" or "organizer" role.
///
/// # Expected Behavior
///
/// Returns `Ok(())` if the user's roles contain "admin" or "organizer".
/// Returns `CoreError::Forbidden` if neither role is present.
///
/// # Errors
///
/// Returns `CoreError::Forbidden` if the user lacks the required role.
///
/// # Side Effects
///
/// None. Pure role check with no I/O.
pub fn require_admin_or_organizer(user: &AuthUser) -> Result<(), crate::errors::CoreError> {
    if user.is_admin_or_organizer() {
        Ok(())
    } else {
        Err(crate::errors::CoreError::Forbidden(
            "Admin or organizer role required".into(),
        ))
    }
}

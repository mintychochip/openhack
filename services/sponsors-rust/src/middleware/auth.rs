pub use openhack_common::auth::{AuthUser, JwtSecret};

use actix_web::web;

/// Extract and validate a Bearer token from the Authorization header,
/// then decode the JWT and return the authenticated user.
///
/// # Expected Behavior
///
/// Reads the `Authorization` header from the request. Expects the format
/// `Bearer <token>`. Decodes the JWT using HS256 with the service's
/// `JWT_SECRET`. Returns `AuthUser` on success or an `SponsorError::Unauthorized`
/// on failure (missing header, malformed token, invalid signature, expired token).
///
/// # Errors
///
/// Returns `SponsorError::Unauthorized` if:
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
) -> Result<AuthUser, crate::errors::SponsorError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            crate::errors::SponsorError::Unauthorized("Missing Authorization header".into())
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        crate::errors::SponsorError::Unauthorized("Invalid Authorization header format".into())
    })?;

    let token_data = jsonwebtoken::decode::<openhack_common::auth::Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|e| crate::errors::SponsorError::Unauthorized(format!("Invalid token: {e}")))?;

    let Ok(user_id) = uuid::Uuid::parse_str(&token_data.claims.sub) else {
        return Err(crate::errors::SponsorError::Unauthorized(
            "Invalid token subject".into(),
        ));
    };

    Ok(AuthUser {
        user_id,
        email: token_data.claims.email,
        roles: token_data.claims.roles,
    })
}

/// Check if the authenticated user has the "sponsor" role (or "organizer"
/// or "admin" which are implicitly granted sponsor access).
///
/// # Expected Behavior
///
/// Returns `Ok(())` if the user's roles contain "sponsor", "organizer",
/// or "admin". Returns `SponsorError::Forbidden` if none of these roles
/// are present.
///
/// # Errors
///
/// Returns `SponsorError::Forbidden` if the user lacks the required role.
///
/// # Side Effects
///
/// None. Pure role check with no I/O.
pub fn require_sponsor_role(user: &AuthUser) -> Result<(), crate::errors::SponsorError> {
    if user.has_role("sponsor") || user.is_admin_or_organizer() {
        Ok(())
    } else {
        Err(crate::errors::SponsorError::Forbidden(
            "Sponsor, organizer, or admin role required".into(),
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
/// Returns `SponsorError::Unauthorized` if the JWT secret is unavailable
/// or if token extraction/validation fails.
///
/// # Side Effects
///
/// - Reads the `JWT_SECRET` from app data (read-only).
/// - Reads the `Authorization` header from the incoming request (read-only).
pub fn get_auth_user(
    req: &actix_web::HttpRequest,
) -> Result<AuthUser, crate::errors::SponsorError> {
    let jwt_secret = req
        .app_data::<web::Data<JwtSecret>>()
        .ok_or_else(|| crate::errors::SponsorError::Internal("JWT secret not configured".into()))?;

    extract_auth_user(req, &jwt_secret.0)
}

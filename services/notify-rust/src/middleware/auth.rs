pub use openhack_common::auth::{AuthUser, JwtSecret};

use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;

/// Actix-web validator callback for bearer token authentication.
///
/// # Expected Behavior
///
/// Called by actix-web-httpauth middleware for each incoming request.
/// Extracts the bearer token, validates it against `JWT_SECRET` from
/// the application config (stored in request `app_data`), and inserts
/// the `AuthUser` into request extensions on success. Returns Ok(()) on
/// valid auth, or an error on failure.
///
/// # Errors
///
/// Returns an `actix_web::Error` if the `JWT_SECRET` config is missing,
/// the token is missing/malformed, the token is expired, or the
/// `sub` claim is not a valid UUID.
///
/// # Side Effects
///
/// - Inserts `AuthUser` into request extensions (mutation).
/// - Logs at WARN level on authentication failure.
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if let Some(user) = check_lambda_internal_token(&req) {
        req.extensions_mut().insert(user);
        return Ok(req);
    }

    let secret = if let Some(s) = req.app_data::<actix_web::web::Data<JwtSecret>>() {
        s.0.clone()
    } else {
        log::error!("JWT_SECRET not configured in app data");
        return Err((
            actix_web::error::ErrorInternalServerError("Server misconfigured"),
            req,
        ));
    };

    match jsonwebtoken::decode::<openhack_common::auth::Claims>(
        credentials.token(),
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    ) {
        Ok(token_data) => {
            let Ok(user_id) = uuid::Uuid::parse_str(&token_data.claims.sub) else {
                log::warn!("Invalid sub claim in JWT: {}", token_data.claims.sub);
                return Err((actix_web::error::ErrorUnauthorized("Invalid token"), req));
            };
            req.extensions_mut().insert(AuthUser {
                user_id,
                email: token_data.claims.email,
                roles: token_data.claims.roles,
            });
            Ok(req)
        }
        Err(e) => {
            log::warn!("JWT validation failed: {e}");
            Err((actix_web::error::ErrorUnauthorized("Invalid token"), req))
        }
    }
}

pub async fn admin_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if let Some(user) = check_lambda_internal_token(&req) {
        req.extensions_mut().insert(user);
        return Ok(req);
    }

    let secret = if let Some(s) = req.app_data::<actix_web::web::Data<JwtSecret>>() {
        s.0.clone()
    } else {
        log::error!("JWT_SECRET not configured in app data");
        return Err((
            actix_web::error::ErrorInternalServerError("Server misconfigured"),
            req,
        ));
    };

    match jsonwebtoken::decode::<openhack_common::auth::Claims>(
        credentials.token(),
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    ) {
        Ok(token_data) => {
            let Ok(user_id) = uuid::Uuid::parse_str(&token_data.claims.sub) else {
                log::warn!("Invalid sub claim in JWT: {}", token_data.claims.sub);
                return Err((actix_web::error::ErrorUnauthorized("Invalid token"), req));
            };
            if !token_data
                .claims
                .roles
                .iter()
                .any(|r| r.eq_ignore_ascii_case("admin"))
            {
                log::warn!("User {user_id} lacks admin role");
                return Err((
                    actix_web::error::ErrorForbidden("Admin access required"),
                    req,
                ));
            }
            req.extensions_mut().insert(AuthUser {
                user_id,
                email: token_data.claims.email,
                roles: token_data.claims.roles,
            });
            Ok(req)
        }
        Err(e) => {
            log::warn!("JWT validation failed: {e}");
            Err((actix_web::error::ErrorUnauthorized("Invalid token"), req))
        }
    }
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

fn check_lambda_internal_token(req: &ServiceRequest) -> Option<AuthUser> {
    let provided = req
        .headers()
        .get("X-Lambda-Internal-Token")
        .and_then(|v| v.to_str().ok());
    validate_lambda_internal_token(provided)
}

pub use openhack_common::auth::{AuthUser, JwtSecret};

use actix_web::web;

pub fn extract_auth_user(
    req: &actix_web::HttpRequest,
    jwt_secret: &str,
) -> Result<AuthUser, crate::errors::AiError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            crate::errors::AiError::Unauthorized("Missing Authorization header".into())
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        crate::errors::AiError::Unauthorized("Invalid Authorization header format".into())
    })?;

    let token_data = jsonwebtoken::decode::<openhack_common::auth::Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|e| crate::errors::AiError::Unauthorized(format!("Invalid token: {e}")))?;

    let Ok(user_id) = uuid::Uuid::parse_str(&token_data.claims.sub) else {
        return Err(crate::errors::AiError::Unauthorized(
            "Invalid token subject".into(),
        ));
    };

    Ok(AuthUser {
        user_id,
        email: token_data.claims.email,
        roles: token_data.claims.roles,
    })
}

pub fn get_auth_user(req: &actix_web::HttpRequest) -> Result<AuthUser, crate::errors::AiError> {
    if let Some(internal_token) = check_lambda_internal_token(req) {
        log::info!("Authenticated via Lambda internal token");
        return Ok(internal_token);
    }

    let jwt_secret = req
        .app_data::<web::Data<JwtSecret>>()
        .ok_or_else(|| crate::errors::AiError::BadRequest("JWT secret not configured".into()))?;

    extract_auth_user(req, &jwt_secret.0)
}

fn check_lambda_internal_token(req: &actix_web::HttpRequest) -> Option<AuthUser> {
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

#[must_use]
pub fn get_optional_user(req: &actix_web::HttpRequest) -> Option<AuthUser> {
    get_auth_user(req).ok()
}

pub fn require_admin(user: &AuthUser) -> Result<(), crate::errors::AiError> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(crate::errors::AiError::Forbidden(
            "Admin role required".into(),
        ))
    }
}

pub fn require_admin_or_organizer(user: &AuthUser) -> Result<(), crate::errors::AiError> {
    if user.is_admin_or_organizer() {
        Ok(())
    } else {
        Err(crate::errors::AiError::Forbidden(
            "Admin or organizer role required".into(),
        ))
    }
}

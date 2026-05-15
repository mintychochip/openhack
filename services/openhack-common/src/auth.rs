use actix_web::{dev::Payload, error, web, Error, FromRequest, HttpRequest};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct JwtSecret(pub String);

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
}

impl AuthUser {
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    #[must_use]
    pub fn is_admin_or_organizer(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "organizer")
    }

    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(user) = check_lambda_internal_token(req) {
            return std::future::ready(Ok(user));
        }

        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        match auth_header {
            Some(token) => {
                let jwt_secret = req.app_data::<web::Data<JwtSecret>>();
                match jwt_secret {
                    Some(secret) => match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(secret.0.as_bytes()),
                        &Validation::new(Algorithm::HS256),
                    ) {
                        Ok(token_data) => {
                            let Ok(user_id) = Uuid::parse_str(&token_data.claims.sub) else {
                                return std::future::ready(Err(error::ErrorUnauthorized(
                                    "Invalid token subject",
                                )));
                            };
                            std::future::ready(Ok(AuthUser {
                                user_id,
                                email: token_data.claims.email,
                                roles: token_data.claims.roles,
                            }))
                        }
                        Err(_) => std::future::ready(Err(error::ErrorUnauthorized(
                            "Invalid or expired token",
                        ))),
                    },
                    None => std::future::ready(Err(error::ErrorInternalServerError(
                        "JWT secret not configured",
                    ))),
                }
            }
            None => std::future::ready(Err(error::ErrorUnauthorized(
                "Missing authorization header",
            ))),
        }
    }
}

fn check_lambda_internal_token(req: &HttpRequest) -> Option<AuthUser> {
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
            user_id: Uuid::nil(),
            email: "internal@lambda".to_string(),
            roles: vec!["admin".to_string()],
        })
    } else {
        log::warn!("Invalid Lambda internal token provided");
        None
    }
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn get_optional_user(req: &HttpRequest) -> Option<AuthUser> {
    AuthUser::extract(req).into_inner().ok()
}

#[must_use]
pub fn get_client_ip(req: &HttpRequest) -> String {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            req.connection_info()
                .peer_addr()
                .map(std::string::ToString::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

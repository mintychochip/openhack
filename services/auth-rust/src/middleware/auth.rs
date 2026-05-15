use actix_web::{dev::Payload, error, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::config::Config;
use crate::services::auth::validate_access_token;

#[derive(Debug, Clone)]
pub struct AuthedUser {
    pub user_id: Uuid,
    pub _email: String,
    pub roles: Vec<String>,
}

impl FromRequest for AuthedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        match auth_header {
            Some(token) => {
                let config = req.app_data::<actix_web::web::Data<Config>>();
                match config {
                    Some(cfg) => match validate_access_token(token, &cfg.jwt_secret) {
                        Ok(claims) => {
                            let Ok(user_id) = Uuid::parse_str(&claims.sub) else {
                                return ready(Err(error::ErrorUnauthorized(
                                    "Invalid token subject",
                                )));
                            };
                            ready(Ok(AuthedUser {
                                user_id,
                                _email: claims.email,
                                roles: claims.roles,
                            }))
                        }
                        Err(_) => ready(Err(error::ErrorUnauthorized("Invalid or expired token"))),
                    },
                    None => ready(Err(error::ErrorInternalServerError(
                        "Server configuration not available",
                    ))),
                }
            }
            None => ready(Err(error::ErrorUnauthorized(
                "Missing authorization header",
            ))),
        }
    }
}

pub fn is_admin(user: &AuthedUser) -> bool {
    user.is_admin()
}

impl AuthedUser {
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }
}

pub fn get_client_ip(req: &HttpRequest) -> String {
    openhack_common::auth::get_client_ip(req)
}

use actix_web::{web, HttpResponse};

use crate::config::Config;
use crate::errors::AiError;
use crate::middleware::auth::{get_auth_user, require_admin};
use crate::models::brand::BrandNormalizeRequest;
use crate::services::brand;

/// Normalize scraped brand data into a DaisyUI-compatible theme.
///
/// # Expected Behavior
///
/// Accepts a JSON `BrandNormalizeRequest` body. Requires an authenticated
/// admin user. Checks if the brand extraction feature is enabled via
/// `FEATURE_BRAND_EXTRACTION`. If disabled, returns 403. Otherwise,
/// delegates to `brand::normalize_brand` which calls the LLM and returns
/// a `BrandNormalizeResponse` with semantic colors, suggested preset,
/// and font recommendations.
///
/// # Errors
///
/// Returns 401 if no valid JWT is provided. Returns 403 if the user is not
/// an admin or if the feature is disabled. Returns 500 on LLM error.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API via service layer (network I/O).
/// - Validates JWT from the Authorization header.
pub async fn brand_normalize(
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    req: web::Json<BrandNormalizeRequest>,
    http_req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user = match get_auth_user(&http_req) {
        Ok(u) => u,
        Err(e) => {
            log::warn!("Brand normalize auth failed: {e}");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": e.to_string()
            }));
        }
    };

    if let Err(e) = require_admin(&auth_user) {
        log::warn!("Brand normalize forbidden for user {}: {e}", auth_user.user_id);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden",
            "message": e.to_string()
        }));
    }

    if !config.feature_brand_extraction {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Brand extraction feature is disabled"
        }));
    }

    match brand::normalize_brand(config.get_ref(), http_client.get_ref(), &req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AiError::Llm(msg)) => {
            log::error!("Brand normalization LLM error: {msg}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "LLM error",
                "message": msg
            }))
        }
        Err(AiError::Http(e)) => {
            log::error!("Brand normalization HTTP error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "HTTP error",
                "message": e.to_string()
            }))
        }
        Err(e) => {
            log::error!("Brand normalization error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to normalize brand",
                "message": e.to_string()
            }))
        }
    }
}

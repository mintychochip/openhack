use actix_web::{web, HttpResponse};

use crate::config::Config;
use crate::errors::AiError;
use crate::services::code_review;

/// Review code using the LLM.
///
/// # Expected Behavior
///
/// Accepts a JSON `CodeReviewRequest` body. Checks if the code review feature
/// is enabled via `FEATURE_CODE_REVIEW`. If disabled, returns 403. Otherwise,
/// delegates to `code_review::review_code` which calls the LLM and returns
/// a `CodeReviewResponse` with findings and a score.
///
/// # Errors
///
/// Returns 403 if feature is disabled. Returns 500 on LLM error.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API via service layer (network I/O).
pub async fn review_code(
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    req: web::Json<code_review::CodeReviewRequest>,
) -> HttpResponse {
    if !config.feature_code_review {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Code review feature is disabled"
        }));
    }

    match code_review::review_code(config.get_ref(), http_client.get_ref(), &req.into_inner()).await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AiError::FeatureDisabled(msg)) => HttpResponse::Forbidden().json(serde_json::json!({
            "error": msg
        })),
        Err(e) => {
            log::error!("Code review error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to review code"
            }))
        }
    }
}

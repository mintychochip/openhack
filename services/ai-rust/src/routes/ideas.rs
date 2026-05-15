use actix_web::{web, HttpResponse};

use crate::config::Config;
use crate::errors::AiError;
use crate::models::ideas::IdeaRequest;
use crate::services::ideas;

/// Generate hackathon project ideas.
///
/// # Expected Behavior
///
/// Accepts a JSON `IdeaRequest` body. Checks if the idea generator feature is
/// enabled via `FEATURE_IDEA_GENERATOR`. If disabled, returns 403. Otherwise,
/// delegates to `ideas::generate_ideas` which calls the LLM and returns an
/// `IdeaResponse` with a list of generated ideas.
///
/// # Errors
///
/// Returns 403 if feature is disabled. Returns 500 on LLM error.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API via service layer (network I/O).
pub async fn generate_ideas(
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    req: web::Json<IdeaRequest>,
) -> HttpResponse {
    if !config.feature_idea_generator {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Idea generator feature is disabled"
        }));
    }

    match ideas::generate_ideas(config.get_ref(), http_client.get_ref(), &req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AiError::FeatureDisabled(msg)) => HttpResponse::Forbidden().json(serde_json::json!({
            "error": msg
        })),
        Err(e) => {
            log::error!("Idea generation error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate ideas"
            }))
        }
    }
}

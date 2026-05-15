use actix_web::{web, HttpResponse};

use crate::config::Config;
use crate::errors::AiError;
use crate::services::team_matcher;

/// Match teammates based on user profile.
///
/// # Expected Behavior
///
/// Accepts a JSON `TeamMatchRequest` body. Checks if the team matcher feature
/// is enabled via `FEATURE_TEAM_MATCHER`. If disabled, returns 403. Otherwise,
/// delegates to `team_matcher::match_teammates` which calls the LLM and returns
/// a `TeamMatchResponse` with suggested teammate profiles.
///
/// # Errors
///
/// Returns 403 if feature is disabled. Returns 500 on LLM error.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API via service layer (network I/O).
pub async fn match_teammates(
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    req: web::Json<team_matcher::TeamMatchRequest>,
) -> HttpResponse {
    if !config.feature_team_matcher {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Team matcher feature is disabled"
        }));
    }

    match team_matcher::match_teammates(config.get_ref(), http_client.get_ref(), &req.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AiError::FeatureDisabled(msg)) => HttpResponse::Forbidden().json(serde_json::json!({
            "error": msg
        })),
        Err(e) => {
            log::error!("Team matcher error: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to match teammates"
            }))
        }
    }
}

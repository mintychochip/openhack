use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

use crate::middleware::auth::get_auth_user;
use crate::services::phase::PhaseService;

#[derive(Deserialize)]
struct LambdaEventInput {
    path: Option<String>,
    #[allow(dead_code)]
    method: Option<String>,
}

/// Lambda event dispatcher endpoint for serverless routing.
///
/// # Expected Behavior
///
/// Receives Lambda-routed events from AWS Lambda Web Adapter (`EventBridge`,
/// SQS, SNS). Parses the JSON body for a `path` field that specifies which
/// internal handler to invoke. Supported paths:
/// - `/api/core/admin/trigger-phase-check` — triggers phase transition check
///
/// Validates the `X-Lambda-Internal-Token` header for authentication.
/// If the `path` field is missing or unrecognized, returns 400.
///
/// # Errors
///
/// Returns 400 if the event body cannot be parsed or the path is unrecognized.
/// Returns 401 if the internal token is invalid.
///
/// # Side Effects
///
/// - May trigger a phase transition check (database writes, event publishing).
/// - Logs at INFO on dispatch, WARN on unknown path or auth failure.
pub async fn dispatch(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if !auth_user.roles.iter().any(|r| r == "admin") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "Admin role required"
        }));
    }

    let input: LambdaEventInput = match serde_json::from_value(body.into_inner()) {
        Ok(i) => i,
        Err(e) => {
            log::warn!("Failed to parse Lambda event input: {e}");
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_input",
                "message": format!("Failed to parse Lambda event: {e}")
            }));
        }
    };

    let Some(path) = input.path else {
        log::warn!("Lambda event missing 'path' field");
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "missing_path",
            "message": "Lambda event must include a 'path' field"
        }));
    };

    if path.as_str() == "/api/core/admin/trigger-phase-check" {
        log::info!("Lambda dispatch: triggering phase check");
        match PhaseService::check_and_transition(pool.get_ref()).await {
            Ok(()) => HttpResponse::Ok().json(serde_json::json!({
                "status": "completed",
                "message": "Phase check and transition executed"
            })),
            Err(e) => {
                log::error!("Phase check failed: {e}");
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "phase_check_failed",
                    "message": format!("{e}")
                }))
            }
        }
    } else {
        log::warn!("Lambda event with unknown path: {path}");
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "unknown_path",
            "message": format!("No handler for path: {path}")
        }))
    }
}

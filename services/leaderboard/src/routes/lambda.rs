use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use sqlx::PgPool;

use crate::services::ranking::RankingService;

#[derive(Deserialize)]
struct LambdaEventInput {
    path: Option<String>,
    #[allow(dead_code)]
    method: Option<String>,
    formula_id: Option<uuid::Uuid>,
}

/// Lambda event dispatcher endpoint for serverless routing.
///
/// # Expected Behavior
///
/// Receives Lambda-routed events from AWS Lambda Web Adapter. Parses the
/// JSON body for a `path` field. Supported paths:
/// - `/api/leaderboard/admin/recalculate` — triggers ranking recalculation
///
/// Auth is handled by `X-Lambda-Internal-Token` header (bypasses JWT).
///
/// # Errors
///
/// Returns 400 if the event body cannot be parsed or the path is unrecognized.
///
/// # Side Effects
///
/// - May trigger ranking recalculation (database writes, cache invalidation).
/// - Logs at INFO on dispatch, WARN on unknown path.
pub async fn dispatch(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
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

    if path.as_str() == "/api/leaderboard/admin/recalculate" {
        log::info!("Lambda dispatch: recalculating rankings");
        match RankingService::recalculate_rankings(
            pool.get_ref(),
            redis_conn.get_ref().as_ref(),
            input.formula_id,
        )
        .await
        {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => {
                log::error!("Recalculation failed: {e}");
                e.to_http_response()
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

use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::middleware::auth::{get_auth_user, get_optional_user};
use crate::models::hackathon::HackathonConfigUpdate;
use crate::services::hackathon::HackathonService;

/// GET /api/core/info — Get hackathon configuration.
///
/// # Expected Behavior
///
/// Returns the hackathon configuration. Authentication is optional; if
/// present, the `join_code` may be included in the response for members.
///
/// # Side Effects
///
/// - Reads from `core.hackathon_config` (database read).
pub async fn get_info(pool: web::Data<PgPool>, req: actix_web::HttpRequest) -> HttpResponse {
    let _auth_user = get_optional_user(&req);
    match HackathonService::get_config(pool.get_ref()).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => e.to_http_response(),
    }
}

/// PUT /api/core/info — Update hackathon configuration.
///
/// # Expected Behavior
///
/// Requires authentication. Updates the singleton hackathon configuration
/// with the provided fields.
///
/// # Side Effects
///
/// - Reads the Authorization header and validates JWT (read).
/// - Updates `core.hackathon_config` (database write).
pub async fn update_info(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<HackathonConfigUpdate>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    match HackathonService::update_config(pool.get_ref(), &body.into_inner()).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => e.to_http_response(),
    }
}

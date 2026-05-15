use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::{get_auth_user, require_admin};
use crate::models::checkin::CheckinScanRequest;
use crate::services::checkin::CheckinService;

/// POST /api/core/checkin/events/{id}/qr — Create a QR code for an event.
///
/// # Expected Behavior
///
/// Requires authentication and admin role. Generates a unique QR code
/// for the specified event.
///
/// # Side Effects
///
/// - Inserts into `core.event_checkins` (database write).
pub async fn create_qr(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let event_id = path.into_inner();
    match CheckinService::create_qr_code(pool.get_ref(), event_id).await {
        Ok(qr) => HttpResponse::Created().json(qr),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/checkin/events/{id}/qr — Get QR codes for an event.
///
/// # Expected Behavior
///
/// Requires authentication and admin role. Returns all QR codes for the event.
///
/// # Side Effects
///
/// - Reads from `core.event_checkins` (database read).
pub async fn get_qr(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let event_id = path.into_inner();
    match CheckinService::get_qr_codes(pool.get_ref(), event_id).await {
        Ok(qr_codes) => HttpResponse::Ok().json(qr_codes),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/checkin/scan — Scan a QR code and check in.
///
/// # Expected Behavior
///
/// Requires authentication. Scans the provided QR code, marks the user
/// as checked in for the associated event.
///
/// # Side Effects
///
/// - Updates `core.event_checkins` (database write).
/// - Updates `core.event_rsvps` (database write).
pub async fn scan_checkin(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<CheckinScanRequest>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    match CheckinService::scan_checkin(pool.get_ref(), &body.qr_code, user_id).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/checkin/events/{id}/stats — Get check-in statistics.
///
/// # Expected Behavior
///
/// Requires authentication and admin role. Returns RSVP count, checked-in
/// count, and total QR codes for the event.
///
/// # Side Effects
///
/// - Reads from `core.event_rsvps` and `core.event_checkins` (database reads).
pub async fn get_stats(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let event_id = path.into_inner();
    match CheckinService::get_checkin_stats(pool.get_ref(), event_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/checkin/events/{id}/attendees — Get check-in attendees.
///
/// # Expected Behavior
///
/// Requires authentication and admin role. Returns all check-in records
/// for the event.
///
/// # Side Effects
///
/// - Reads from `core.event_checkins` (database read).
pub async fn get_attendees(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };
    if let Err(e) = require_admin(&auth_user) {
        return e.to_http_response();
    }

    let event_id = path.into_inner();
    match CheckinService::get_checkin_attendees(pool.get_ref(), event_id).await {
        Ok(attendees) => HttpResponse::Ok().json(attendees),
        Err(e) => e.to_http_response(),
    }
}

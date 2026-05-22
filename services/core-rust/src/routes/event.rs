use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::get_auth_user;
use crate::models::event::{AttendeeListQuery, EventCreate, EventListQuery, EventUpdate};
use crate::services::event::EventService;

/// POST /api/core/events — Create a new event.
///
/// # Expected Behavior
///
/// Requires authentication. Creates a new event.
///
/// # Side Effects
///
/// - Inserts into `core.events` (database write).
pub async fn create_event(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<EventCreate>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    match EventService::create_event(pool.get_ref(), &body.into_inner()).await {
        Ok(event) => HttpResponse::Created().json(event),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/events — List events with optional type filter.
///
/// # Expected Behavior
///
/// Optional auth. Supports type, limit, offset query params.
///
/// # Side Effects
///
/// - Reads from `core.events` (database read).
pub async fn list_events(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    query: web::Query<EventListQuery>,
) -> HttpResponse {
    let type_filter = query.r#type.as_deref();
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    match EventService::list_events(pool.get_ref(), type_filter, limit, offset).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/events/{id} — Get an event by ID.
///
/// # Expected Behavior
///
/// Optional auth. Returns event details with RSVP count.
///
/// # Side Effects
///
/// - Reads from `core.events` and `core.event_rsvps` (database reads).
pub async fn get_event(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    match EventService::get_event(pool.get_ref(), id).await {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => e.to_http_response(),
    }
}

/// PUT /api/core/events/{id} — Update an event.
///
/// # Expected Behavior
///
/// Requires authentication. Updates only provided fields.
///
/// # Side Effects
///
/// - Updates `core.events` (database write).
pub async fn update_event(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<EventUpdate>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let id = path.into_inner();
    match EventService::update_event(pool.get_ref(), id, &body.into_inner()).await {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/events/{id} — Delete an event.
///
/// # Expected Behavior
///
/// Requires authentication. Deletes the event and its RSVPs (cascade).
///
/// # Side Effects
///
/// - Deletes from `core.events` (database write, cascades).
pub async fn delete_event(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let id = path.into_inner();
    match EventService::delete_event(pool.get_ref(), id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/events/{id}/rsvp — RSVP to an event.
///
/// # Expected Behavior
///
/// Requires authentication. Creates an RSVP for the user.
///
/// # Side Effects
///
/// - Inserts into `core.event_rsvps` (database write).
/// - Publishes `event.rsvp` event to Redis.
pub async fn rsvp_event(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let event_id = path.into_inner();
    let mut redis_local = redis_conn.get_ref().clone();
    match EventService::rsvp_event(pool.get_ref(), redis_local.as_mut(), event_id, user_id).await {
        Ok(()) => {
            openhack_common::metrics::inc_business_counter("core_event_rsvps_total");
            HttpResponse::Ok().json(serde_json::json!({"message": "RSVP successful"}))
        }
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/events/{id}/rsvp — Cancel an RSVP.
///
/// # Expected Behavior
///
/// Requires authentication. Removes the user's RSVP.
///
/// # Side Effects
///
/// - Deletes from `core.event_rsvps` (database write).
pub async fn cancel_rsvp(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let event_id = path.into_inner();
    match EventService::cancel_rsvp(pool.get_ref(), event_id, user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/events/{id}/attendees — Get event attendees.
///
/// # Expected Behavior
///
/// Optional auth. Supports `attended_only`, limit, offset query params.
///
/// # Side Effects
///
/// - Reads from `core.event_rsvps` (database read).
pub async fn get_attendees(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    query: web::Query<AttendeeListQuery>,
) -> HttpResponse {
    let event_id = path.into_inner();
    let attended_only = query.attended_only.unwrap_or(false);
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    match EventService::get_attendees(pool.get_ref(), event_id, attended_only, limit, offset).await
    {
        Ok(attendees) => HttpResponse::Ok().json(attendees),
        Err(e) => e.to_http_response(),
    }
}

/// POST /`api/core/events/{id}/attendees/{user_id`} — Mark attendee as attended.
///
/// # Expected Behavior
///
/// Requires authentication. Marks the specified user as having attended.
///
/// # Side Effects
///
/// - Updates `core.event_rsvps` (database write).
pub async fn mark_attended(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let _auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let (event_id, user_id) = path.into_inner();
    match EventService::mark_attended(pool.get_ref(), event_id, user_id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"message": "Marked as attended"})),
        Err(e) => e.to_http_response(),
    }
}

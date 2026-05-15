use actix_web::{web, HttpResponse, ResponseError};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::events::publisher;
use crate::middleware::auth::AuthedUser;
use crate::models::session::LogoutResponse;
use crate::services::session;

/// POST /api/auth/logout — Revoke all sessions for the authenticated user.
///
/// # Expected Behavior
///
/// Requires a valid JWT access token. Revokes all sessions for the
/// authenticated user by setting `revoked_at = NOW()` on all non-revoked
/// sessions in `auth.sessions`. Also deletes all session cache entries
/// from Redis. Returns `{loggedOut: true, sessionsRevoked: N}` where N
/// is the number of sessions that were revoked. Publishes a
/// `user.logged_out` event.
///
/// # Errors
///
/// Returns 401 if the access token is missing or invalid.
/// Returns 500 on database or internal errors.
///
/// # Side Effects
///
/// - Writes to `auth.sessions` table (bulk update `revoked_at`).
/// - Deletes Redis keys `session:*` for the user's sessions.
/// - Publishes `user.logged_out` event to Redis.
pub async fn logout(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    user: AuthedUser,
) -> HttpResponse {
    match session::revoke_all_user_sessions(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        user.user_id,
    )
    .await
    {
        Ok(count) => {
            publisher::user_logged_out(redis_conn.get_ref().as_ref(), &user.user_id.to_string())
                .await;
            HttpResponse::Ok().json(LogoutResponse {
                logged_out: true,
                sessions_revoked: count,
            })
        }
        Err(e) => e.error_response(),
    }
}

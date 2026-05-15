use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::config::Config;
use crate::events::publisher;
use crate::middleware::auth::get_client_ip;
use crate::middleware::rate_limit::check_rate_limit;
use crate::models::user::RegisterRequest;
use crate::services::auth;

/// POST /api/auth/register — Register a new user.
///
/// # Expected Behavior
///
/// Rate-limited to 10 requests per minute per IP. Validates the request
/// (email format, password length >= 8, name non-empty). Checks email
/// uniqueness. Hashes the password with Argon2. Inserts a new user into
/// `auth.users` with default roles `['participant']`. Publishes a
/// `user.registered` event to Redis. Returns the new user's ID, email,
/// name, and createdAt.
///
/// # Errors
///
/// Returns 400 if email/password/name validation fails.
/// Returns 409 if email is already registered.
/// Returns 429 if rate limit is exceeded.
/// Returns 500 on database or internal errors.
///
/// # Side Effects
///
/// - Writes to `auth.users` table (insert).
/// - Publishes `user.registered` event to Redis.
/// - Increments Redis rate limit counter.
/// - CPU-intensive Argon2 password hashing.
pub async fn register(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    _config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    let ip = get_client_ip(&req);

    if let Err(e) = check_rate_limit(redis_conn.get_ref().as_ref(), "register", &ip, 10, 60).await {
        return e.error_response();
    }

    match auth::register_user(pool.get_ref(), &body.into_inner()).await {
        Ok(response) => {
            publisher::user_registered(
                redis_conn.get_ref().as_ref(),
                &response.id.to_string(),
                &response.email,
                &response.name,
            )
            .await;
            HttpResponse::Created().json(response)
        }
        Err(e) => e.error_response(),
    }
}

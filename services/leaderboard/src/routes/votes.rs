use actix_web::{web, HttpResponse};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::vote::VoteCreate;
use crate::services::voting::VotingService;

/// Cast a vote for a project.
///
/// # Expected Behavior
///
/// Public endpoint. Validates voting is open, checks rate limits
/// and max votes per user from the active voting config, then
/// inserts the vote. Rejects duplicate votes. Returns 201 Created
/// with the vote data on success.
///
/// # Errors
///
/// Returns 400 if voting is closed, rate limit exceeded, max votes
/// exceeded, or duplicate vote.
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from `leaderboard.voting_config` (database read).
/// - Reads from `leaderboard.votes` for rate limit checks (database read).
/// - Inserts a row into `leaderboard.votes` (database write).
/// - Updates `leaderboard.ranks` `public_votes/weighted_votes` (database write).
/// - Invalidates leaderboard cache (Redis write).
pub async fn cast_vote(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    body: web::Json<VoteCreate>,
) -> HttpResponse {
    match VotingService::cast_vote(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        &body.into_inner(),
    )
    .await
    {
        Ok(response) => {
            openhack_common::metrics::inc_business_counter("leaderboard_votes_total");
            HttpResponse::Created().json(response)
        }
        Err(e) => e.to_http_response(),
    }
}

/// Get vote count for a project.
///
/// # Expected Behavior
///
/// Public endpoint. Returns the count of valid votes and the sum
/// of weighted votes for the given `project_id`. Checks Redis cache
/// first with a 30-second TTL.
///
/// # Errors
///
/// Returns 500 on database/Redis errors.
///
/// # Side Effects
///
/// - Reads from Redis cache (read).
/// - Queries `leaderboard.votes` on cache miss (database read).
/// - Writes to Redis cache on cache miss (write).
pub async fn get_vote_count(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let project_id = path.into_inner();
    match VotingService::get_vote_count(pool.get_ref(), redis_conn.get_ref().as_ref(), project_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.to_http_response(),
    }
}

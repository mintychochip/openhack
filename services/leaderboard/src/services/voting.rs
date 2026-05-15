use chrono::{Duration, Utc};
use redis::aio::MultiplexedConnection;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::LeaderboardError;
use crate::models::vote::{Vote, VoteCreate, VoteModerate, VoteResponse};
use crate::models::voting_config::VotingConfig;
use crate::services::cache;

pub struct VotingService;

impl VotingService {
    /// Cast a vote for a project with rate limiting and validation.
    ///
    /// # Expected Behavior
    ///
    /// Checks if voting is open using the `leaderboard.is_voting_open()`
    /// `PostgreSQL` function. If closed, returns a validation error. Loads
    /// the active voting config and checks: (1) max votes per user, and
    /// (2) rate limit (votes per minute window). Uses the default vote
    /// weight from the config. Inserts the vote and updates the team's
    /// `public_votes` and `weighted_votes` in `leaderboard.ranks`. Rejects
    /// duplicate votes with a unique constraint error.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Validation` if voting is closed, if the
    /// user has exceeded the max votes limit, or if the rate limit is
    /// exceeded. Returns `LeaderboardError::Validation` for duplicate
    /// votes (unique constraint violation on `voter_token_hash`, `project_id`).
    /// Returns `LeaderboardError::Database` for other database failures.
    ///
    /// # Side Effects
    ///
    /// - Reads from `leaderboard.voting_config` (database read).
    /// - Reads from `leaderboard.votes` for rate limit checks (database read).
    /// - Inserts a row into `leaderboard.votes` (database write).
    /// - Updates `public_votes` and `weighted_votes` in `leaderboard.ranks`
    ///   (database write).
    /// - Invalidates leaderboard cache (Redis write).
    /// - Logs at INFO level on successful vote.
    pub async fn cast_vote(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        data: &VoteCreate,
    ) -> Result<VoteResponse, LeaderboardError> {
        let voting_open: bool = sqlx::query_scalar("SELECT leaderboard.is_voting_open()")
            .fetch_one(pool)
            .await
            .unwrap_or(false);

        if !voting_open {
            return Err(LeaderboardError::Validation(
                "Voting is currently closed".into(),
            ));
        }

        let voting_config = sqlx::query_as::<_, VotingConfig>(
            "SELECT * FROM leaderboard.voting_config WHERE is_active = true LIMIT 1",
        )
        .fetch_optional(pool)
        .await?;

        let weight: Decimal = voting_config
            .as_ref()
            .and_then(|c| c.vote_weight_default)
            .unwrap_or(Decimal::ONE);

        if let Some(ref vc) = voting_config {
            let user_vote_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM leaderboard.votes WHERE voter_token_hash = $1 AND is_valid = true",
            )
            .bind(&data.voter_token_hash)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            let max_votes = i64::from(vc.max_votes_per_user.unwrap_or(10));
            if user_vote_count >= max_votes {
                return Err(LeaderboardError::Validation(format!(
                    "Maximum votes per user exceeded ({user_vote_count}/{max_votes})"
                )));
            }

            let window_seconds = vc.rate_limit_window_seconds.unwrap_or(60);
            let cutoff = Utc::now() - Duration::seconds(i64::from(window_seconds));
            let recent_votes: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM leaderboard.votes WHERE voter_token_hash = $1 AND created_at > $2",
            )
            .bind(&data.voter_token_hash)
            .bind(cutoff)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            let max_per_minute = i64::from(vc.max_votes_per_minute.unwrap_or(5));
            if recent_votes >= max_per_minute {
                return Err(LeaderboardError::Validation(
                    "Rate limit exceeded, too many votes".into(),
                ));
            }
        }

        let row = sqlx::query_as::<_, Vote>(
            "INSERT INTO leaderboard.votes (project_id, voter_token_hash, voter_ip, weight_applied)
             VALUES ($1, $2, $3, $4)
             RETURNING *",
        )
        .bind(data.project_id)
        .bind(&data.voter_token_hash)
        .bind(&data.voter_ip)
        .bind(weight)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.is_unique_violation() {
                    return LeaderboardError::Validation("Already voted for this project".into());
                }
            }
            LeaderboardError::Database(e)
        })?;

        sqlx::query(
            "UPDATE leaderboard.ranks SET
                public_votes = COALESCE(public_votes, 0) + 1,
                weighted_votes = COALESCE(weighted_votes, 0) + $1,
                last_updated = NOW()
             WHERE team_id = $2",
        )
        .bind(weight)
        .bind(data.project_id)
        .execute(pool)
        .await
        .ok();

        cache::invalidate_leaderboard_cache(conn.cloned())
            .await
            .ok();

        log::info!("Vote cast for project {} by token hash", data.project_id);

        Ok(row.into())
    }

    /// Get the vote count and weighted votes for a project.
    ///
    /// # Expected Behavior
    ///
    /// Queries the count of valid votes and the sum of their weights
    /// for the given `project_id`. Checks Redis cache first; on cache
    /// miss, queries the database and caches the result with a
    /// 30-second TTL.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `leaderboard:votes:{project_id}` (read).
    /// - Queries `leaderboard.votes` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn get_vote_count(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        project_id: Uuid,
    ) -> Result<serde_json::Value, LeaderboardError> {
        if let Ok(Some(cached)) = cache::get_vote_count_from_cache(conn.cloned(), &project_id).await
        {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&cached) {
                return Ok(parsed);
            }
        }

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM leaderboard.votes WHERE project_id = $1 AND is_valid = true",
        )
        .bind(project_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let weighted: Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(weight_applied), 0) FROM leaderboard.votes WHERE project_id = $1 AND is_valid = true",
        )
        .bind(project_id)
        .fetch_one(pool)
        .await
        .unwrap_or(Decimal::ZERO);

        let response = serde_json::json!({
            "project_id": project_id,
            "vote_count": count,
            "weighted_votes": weighted.to_string()
        });

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_vote_count_cache(conn.cloned(), &project_id, &json)
                .await
                .ok();
        }

        Ok(response)
    }

    /// Moderate a vote by setting its validity status.
    ///
    /// # Expected Behavior
    ///
    /// Updates the vote's `is_valid`, `moderated_at`, `moderated_by`,
    /// and `moderation_reason` fields. If the validity changed, adjusts
    /// the team's `public_votes` and `weighted_votes` in `leaderboard.ranks`
    /// accordingly (incrementing or decrementing). Invalidates the
    /// leaderboard cache after the update.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::NotFound` if no vote exists with the
    /// given ID.
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `leaderboard.votes` to get the existing vote (database read).
    /// - Updates `leaderboard.votes` (database write).
    /// - Updates `public_votes` and `weighted_votes` in `leaderboard.ranks`
    ///   if validity changed (database write).
    /// - Invalidates leaderboard cache (Redis write).
    /// - Logs at INFO level on successful moderation.
    pub async fn moderate_vote(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        vote_id: Uuid,
        moderator_id: Uuid,
        data: &VoteModerate,
    ) -> Result<VoteResponse, LeaderboardError> {
        let existing = sqlx::query_as::<_, Vote>("SELECT * FROM leaderboard.votes WHERE id = $1")
            .bind(vote_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| LeaderboardError::NotFound("Vote".into(), vote_id.to_string()))?;

        let was_valid = existing.is_valid.unwrap_or(true);

        let row = sqlx::query_as::<_, Vote>(
            "UPDATE leaderboard.votes SET is_valid = $1, moderated_at = NOW(), moderated_by = $2, moderation_reason = $3 WHERE id = $4 RETURNING *",
        )
        .bind(data.is_valid)
        .bind(moderator_id)
        .bind(&data.moderation_reason)
        .bind(vote_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| LeaderboardError::NotFound("Vote".into(), vote_id.to_string()))?;

        if was_valid != data.is_valid {
            let weight = existing.weight_applied.unwrap_or(Decimal::ONE);
            if data.is_valid {
                sqlx::query(
                    "UPDATE leaderboard.ranks SET public_votes = COALESCE(public_votes, 0) + 1, weighted_votes = COALESCE(weighted_votes, 0) + $1, last_updated = NOW() WHERE team_id = $2",
                )
                .bind(weight)
                .bind(existing.project_id)
                .execute(pool)
                .await
                .ok();
            } else {
                sqlx::query(
                    "UPDATE leaderboard.ranks SET public_votes = GREATEST(COALESCE(public_votes, 0) - 1, 0), weighted_votes = GREATEST(COALESCE(weighted_votes, 0) - $1, 0), last_updated = NOW() WHERE team_id = $2",
                )
                .bind(weight)
                .bind(existing.project_id)
                .execute(pool)
                .await
                .ok();
            }
            cache::invalidate_leaderboard_cache(conn.cloned())
                .await
                .ok();
        }

        log::info!("Moderated vote {} (valid={})", vote_id, data.is_valid);

        Ok(row.into())
    }
}

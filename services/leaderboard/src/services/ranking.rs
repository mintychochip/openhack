use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::LeaderboardError;
use crate::models::formula::RankingFormula;
use crate::models::rank::{LeaderboardResponse, Rank};
use crate::models::score_history::{ScoreHistory, ScoreHistoryListResponse};
use crate::services::cache;

/// Substitute variable names in a ranking formula with SQL column references.
///
/// # Expected Behavior
///
/// Replaces all occurrences of `judge_score` with `total_score` and all
/// occurrences of `public_votes` with `COALESCE(weighted_votes, 0)` in
/// the provided formula string. Uses Rust's `str::replace`, which performs
/// simple substring replacement (not regex). The substitution does not
/// sanitize or validate the formula for SQL safety — it is a pure string
/// mapping of logical variable names to their physical column references.
///
/// # Arguments
///
/// * `formula` - The formula string containing logical variable names like
///   `judge_score` and `public_votes`.
///
/// # Returns
///
/// The formula string with variable names replaced by SQL column references.
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None. Pure function with no I/O.
pub(crate) fn substitute_formula_variables(formula: &str) -> String {
    formula
        .replace("judge_score", "total_score")
        .replace("public_votes", "COALESCE(weighted_votes, 0)")
}

pub struct RankingService;

impl RankingService {
    /// Get the current leaderboard sorted by rank.
    ///
    /// # Expected Behavior
    ///
    /// Queries `leaderboard.ranks` ordered by rank ascending (NULLS LAST),
    /// then by `combined_score` and `total_score` descending. Checks Redis
    /// cache first; on cache miss, queries the database and caches the
    /// result with a 1-minute TTL. Supports pagination via limit/offset.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` if the database query fails.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `leaderboard:current` (read).
    /// - Queries `leaderboard.ranks` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn get_leaderboard(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        limit: i64,
        offset: i64,
    ) -> Result<LeaderboardResponse, LeaderboardError> {
        if let Ok(Some(cached)) = cache::get_leaderboard_from_cache(conn.cloned()).await {
            if let Ok(parsed) = serde_json::from_str::<LeaderboardResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.ranks")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let ranks = sqlx::query_as::<_, Rank>(
            "SELECT * FROM leaderboard.ranks ORDER BY rank ASC NULLS LAST, combined_score DESC NULLS LAST, total_score DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let response = LeaderboardResponse {
            leaderboard: ranks.into_iter().map(Into::into).collect(),
            total,
        };

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_leaderboard_cache(conn.cloned(), &json)
                .await
                .ok();
        }

        Ok(response)
    }

    /// Get score history for a specific team.
    ///
    /// # Expected Behavior
    ///
    /// Queries `leaderboard.score_history` for the given `team_id`,
    /// ordered by `recorded_at` descending with pagination. Checks Redis
    /// cache first; on cache miss, queries the database and caches the
    /// result with a 2-minute TTL.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` if the database query fails.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `leaderboard:history:{team_id}` (read).
    /// - Queries `leaderboard.score_history` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn get_score_history(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        team_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<ScoreHistoryListResponse, LeaderboardError> {
        if let Ok(Some(cached)) = cache::get_score_history_from_cache(conn.cloned(), &team_id).await
        {
            if let Ok(parsed) = serde_json::from_str::<ScoreHistoryListResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.score_history WHERE team_id = $1")
                .bind(team_id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let history = sqlx::query_as::<_, ScoreHistory>(
            "SELECT * FROM leaderboard.score_history WHERE team_id = $1 ORDER BY recorded_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(team_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let response = ScoreHistoryListResponse {
            history: history.into_iter().map(Into::into).collect(),
            total,
        };

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_score_history_cache(conn.cloned(), &team_id, &json)
                .await
                .ok();
        }

        Ok(response)
    }

    /// Recalculate rankings using the active or specified formula.
    ///
    /// # Expected Behavior
    ///
    /// Loads the specified formula (or the active one if none specified).
    /// Substitutes variable names in the formula to column references
    /// (`judge_score` → `total_score`, `public_votes` → `weighted_votes`) and
    /// updates `combined_score` for all non-frozen teams. Recalculates
    /// rank using `ROW_NUMBER()` window function, preserving `previous_rank`.
    /// Records score history for all updated teams. Invalidates all
    /// leaderboard caches.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::NotFound` if no formula exists with the
    /// given ID or if no active formula exists.
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `leaderboard.ranking_formulas` (database read).
    /// - Updates `combined_score`, `formula_id`, `previous_rank`, `rank`,
    ///   and `last_updated` in `leaderboard.ranks` (database writes).
    /// - Inserts rows into `leaderboard.score_history` (database write).
    /// - Invalidates all leaderboard-related Redis caches (writes).
    /// - Logs at INFO level on successful recalculation.
    pub async fn recalculate_rankings(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        formula_id: Option<Uuid>,
    ) -> Result<LeaderboardResponse, LeaderboardError> {
        let formula = if let Some(fid) = formula_id {
            sqlx::query_as::<_, RankingFormula>(
                "SELECT * FROM leaderboard.ranking_formulas WHERE id = $1",
            )
            .bind(fid)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| LeaderboardError::NotFound("Formula".into(), fid.to_string()))?
        } else {
            sqlx::query_as::<_, RankingFormula>(
                "SELECT * FROM leaderboard.ranking_formulas WHERE is_active = true LIMIT 1",
            )
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                LeaderboardError::NotFound("Formula".into(), "no active formula".into())
            })?
        };

        let sql_formula = substitute_formula_variables(&formula.formula);

        let update_sql = format!(
            "UPDATE leaderboard.ranks SET combined_score = ({sql_formula}), formula_id = $1, last_updated = NOW() WHERE is_frozen = false"
        );

        sqlx::query(&update_sql)
            .bind(formula.id)
            .execute(pool)
            .await?;

        sqlx::query(
            "UPDATE leaderboard.ranks r SET
                previous_rank = r.rank,
                rank = sub.new_rank,
                last_updated = NOW()
             FROM (
                SELECT team_id, ROW_NUMBER() OVER (ORDER BY combined_score DESC NULLS LAST, total_score DESC) as new_rank
                FROM leaderboard.ranks
                WHERE is_frozen = false
             ) sub
             WHERE r.team_id = sub.team_id AND r.is_frozen = false",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT INTO leaderboard.score_history (team_id, score, rank)
             SELECT team_id, total_score, rank FROM leaderboard.ranks WHERE is_frozen = false",
        )
        .execute(pool)
        .await?;

        cache::invalidate_all_leaderboard_caches(conn.cloned()).await;

        log::info!("Recalculated rankings using formula {}", formula.id);

        Self::get_leaderboard(pool, conn, 100, 0).await
    }

    /// Freeze the leaderboard, locking all non-frozen team rankings.
    ///
    /// # Expected Behavior
    ///
    /// Sets `is_frozen = true` and `frozen_at = NOW()` for all teams
    /// where `is_frozen = false`. Optionally sets `phase_id` if provided.
    /// Invalidates all leaderboard caches after the update.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates `is_frozen`, `frozen_at`, and `phase_id` in
    ///   `leaderboard.ranks` (database write).
    /// - Invalidates all leaderboard-related Redis caches (writes).
    /// - Logs at INFO level on successful freeze.
    pub async fn freeze_leaderboard(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        phase_id: Option<Uuid>,
    ) -> Result<LeaderboardResponse, LeaderboardError> {
        sqlx::query(
            "UPDATE leaderboard.ranks SET is_frozen = true, frozen_at = NOW(), phase_id = COALESCE($1, phase_id) WHERE is_frozen = false",
        )
        .bind(phase_id)
        .execute(pool)
        .await?;

        cache::invalidate_all_leaderboard_caches(conn.cloned()).await;

        log::info!("Leaderboard frozen");

        Self::get_leaderboard(pool, conn, 100, 0).await
    }

    /// Unfreeze the leaderboard, unlocking all frozen team rankings.
    ///
    /// # Expected Behavior
    ///
    /// Sets `is_frozen = false` and `frozen_at = NULL` for all teams
    /// where `is_frozen = true`. Invalidates all leaderboard caches
    /// after the update.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates `is_frozen` and `frozen_at` in `leaderboard.ranks`
    ///   (database write).
    /// - Invalidates all leaderboard-related Redis caches (writes).
    /// - Logs at INFO level on successful unfreeze.
    pub async fn unfreeze_leaderboard(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
    ) -> Result<LeaderboardResponse, LeaderboardError> {
        sqlx::query(
            "UPDATE leaderboard.ranks SET is_frozen = false, frozen_at = NULL WHERE is_frozen = true",
        )
        .execute(pool)
        .await?;

        cache::invalidate_all_leaderboard_caches(conn.cloned()).await;

        log::info!("Leaderboard unfrozen");

        Self::get_leaderboard(pool, conn, 100, 0).await
    }

    /// Get leaderboard statistics.
    ///
    /// # Expected Behavior
    ///
    /// Queries counts of teams, valid votes, frozen teams, formulas, and
    /// snapshots. Returns a JSON object with these statistics.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `leaderboard.ranks`, `leaderboard.votes`,
    ///   `leaderboard.ranking_formulas`, and `leaderboard.snapshots`
    ///   (database reads).
    pub async fn get_stats(pool: &PgPool) -> Result<serde_json::Value, LeaderboardError> {
        let team_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.ranks")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let total_votes: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.votes WHERE is_valid = true")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let frozen_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.ranks WHERE is_frozen = true")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let formula_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.ranking_formulas")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let snapshot_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.snapshots")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        Ok(serde_json::json!({
            "team_count": team_count,
            "total_votes": total_votes,
            "frozen_teams": frozen_count,
            "formula_count": formula_count,
            "snapshot_count": snapshot_count
        }))
    }
}

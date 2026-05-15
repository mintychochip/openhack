use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::LeaderboardError;
use crate::models::formula::{FormulaCreate, FormulaListResponse, FormulaResponse, RankingFormula};
use crate::models::rank::Rank;
use crate::models::snapshot::{Snapshot, SnapshotCreate, SnapshotListResponse, SnapshotResponse};
use crate::models::voting_config::{VotingConfig, VotingConfigResponse, VotingConfigUpdate};
use crate::services::cache;

pub struct FormulaService;

impl FormulaService {
    /// List all ranking formulas.
    ///
    /// # Expected Behavior
    ///
    /// Queries `leaderboard.ranking_formulas` ordered by `created_at`
    /// descending. Checks Redis cache first; on cache miss, queries
    /// the database and caches the result with a 5-minute TTL.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `leaderboard:formulas` (read).
    /// - Queries `leaderboard.ranking_formulas` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn list_formulas(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
    ) -> Result<FormulaListResponse, LeaderboardError> {
        if let Ok(Some(cached)) = cache::get_formulas_from_cache(conn.cloned()).await {
            if let Ok(parsed) = serde_json::from_str::<FormulaListResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.ranking_formulas")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let formulas = sqlx::query_as::<_, RankingFormula>(
            "SELECT * FROM leaderboard.ranking_formulas ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;

        let response = FormulaListResponse {
            formulas: formulas.into_iter().map(Into::into).collect(),
            total,
        };

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_formulas_cache(conn.cloned(), &json).await.ok();
        }

        Ok(response)
    }

    /// Get the currently active ranking formula.
    ///
    /// # Expected Behavior
    ///
    /// Queries `leaderboard.ranking_formulas` where `is_active = true`.
    /// Checks Redis cache first; on cache miss, queries the database
    /// and caches the result with a 5-minute TTL.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::NotFound` if no active formula exists.
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `leaderboard:formulas:active` (read).
    /// - Queries `leaderboard.ranking_formulas` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn get_active_formula(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
    ) -> Result<FormulaResponse, LeaderboardError> {
        if let Ok(Some(cached)) = cache::get_active_formula_from_cache(conn.cloned()).await {
            if let Ok(parsed) = serde_json::from_str::<FormulaResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let formula = sqlx::query_as::<_, RankingFormula>(
            "SELECT * FROM leaderboard.ranking_formulas WHERE is_active = true LIMIT 1",
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| LeaderboardError::NotFound("Formula".into(), "no active formula".into()))?;

        let response: FormulaResponse = formula.into();

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_active_formula_cache(conn.cloned(), &json)
                .await
                .ok();
        }

        Ok(response)
    }

    /// Create a new ranking formula.
    ///
    /// # Expected Behavior
    ///
    /// Inserts a new row into `leaderboard.ranking_formulas` with the
    /// provided data. `is_active` and `is_preset` default to `false`.
    /// Returns the created formula as a `FormulaResponse`. Invalidates
    /// the formulas cache after creation.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` if the insert fails.
    ///
    /// # Side Effects
    ///
    /// - Inserts a row into `leaderboard.ranking_formulas` (database write).
    /// - Invalidates `leaderboard:formulas` Redis cache (write).
    /// - Logs at INFO level on successful creation.
    pub async fn create_formula(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        data: &FormulaCreate,
    ) -> Result<FormulaResponse, LeaderboardError> {
        let row = sqlx::query_as::<_, RankingFormula>(
            "INSERT INTO leaderboard.ranking_formulas (name, description, formula, variables)
             VALUES ($1, $2, $3, $4)
             RETURNING *",
        )
        .bind(&data.name)
        .bind(&data.description)
        .bind(&data.formula)
        .bind(&data.variables)
        .fetch_one(pool)
        .await?;

        let formula_id = row.id;
        let response = row.into();

        cache::invalidate_formulas_cache(conn.cloned()).await.ok();

        log::info!("Created formula {formula_id}");

        Ok(response)
    }

    /// Activate a ranking formula by ID.
    ///
    /// # Expected Behavior
    ///
    /// Deactivates all formulas first (sets `is_active = false`), then
    /// activates the specified formula (sets `is_active = true`). Returns
    /// the activated formula as a `FormulaResponse`. Invalidates both
    /// the formulas list and active formula caches.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::NotFound` if no formula exists with the
    /// given ID.
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates all rows in `leaderboard.ranking_formulas` to set
    ///   `is_active = false` (database write).
    /// - Updates the specified formula to set `is_active = true`
    ///   (database write).
    /// - Invalidates `leaderboard:formulas` and `leaderboard:formulas:active`
    ///   Redis caches (writes).
    /// - Logs at INFO level on successful activation.
    pub async fn activate_formula(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
    ) -> Result<FormulaResponse, LeaderboardError> {
        sqlx::query(
            "UPDATE leaderboard.ranking_formulas SET is_active = false, updated_at = NOW()",
        )
        .execute(pool)
        .await?;

        let row = sqlx::query_as::<_, RankingFormula>(
            "UPDATE leaderboard.ranking_formulas SET is_active = true, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| LeaderboardError::NotFound("Formula".into(), id.to_string()))?;

        cache::invalidate_formulas_cache(conn.cloned()).await.ok();
        cache::invalidate_active_formula_cache(conn.cloned())
            .await
            .ok();

        log::info!("Activated formula {id}");

        Ok(row.into())
    }

    /// Update the active voting configuration.
    ///
    /// # Expected Behavior
    ///
    /// Loads the active voting config. Updates only the fields provided
    /// in `VotingConfigUpdate` using dynamic SQL construction. Returns
    /// the updated config as a `VotingConfigResponse`. Invalidates the
    /// voting config cache after the update.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::NotFound` if no active voting config exists.
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates row in `leaderboard.voting_config` (database write).
    /// - Invalidates `leaderboard:voting_config` Redis cache (write).
    /// - Logs at INFO level on successful update.
    pub async fn update_voting_config(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        data: &VotingConfigUpdate,
    ) -> Result<VotingConfigResponse, LeaderboardError> {
        let existing = sqlx::query_as::<_, VotingConfig>(
            "SELECT * FROM leaderboard.voting_config WHERE is_active = true LIMIT 1",
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            LeaderboardError::NotFound("VotingConfig".into(), "no active config".into())
        })?;

        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.opens_at.is_some() {
            updates.push(format!("opens_at = ${param_idx}"));
            param_idx += 1;
        }
        if data.closes_at.is_some() {
            updates.push(format!("closes_at = ${param_idx}"));
            param_idx += 1;
        }
        if data.max_votes_per_user.is_some() {
            updates.push(format!("max_votes_per_user = ${param_idx}"));
            param_idx += 1;
        }
        if data.max_votes_per_minute.is_some() {
            updates.push(format!("max_votes_per_minute = ${param_idx}"));
            param_idx += 1;
        }
        if data.rate_limit_window_seconds.is_some() {
            updates.push(format!("rate_limit_window_seconds = ${param_idx}"));
            param_idx += 1;
        }
        if data.vote_weight_default.is_some() {
            updates.push(format!("vote_weight_default = ${param_idx}"));
            param_idx += 1;
        }
        if data.is_active.is_some() {
            updates.push(format!("is_active = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Ok(existing.into());
        }

        let sql = format!(
            "UPDATE leaderboard.voting_config SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, VotingConfig>(&sql);

        if let Some(ref v) = data.opens_at {
            query = query.bind(v);
        }
        if let Some(ref v) = data.closes_at {
            query = query.bind(v);
        }
        if let Some(ref v) = data.max_votes_per_user {
            query = query.bind(v);
        }
        if let Some(ref v) = data.max_votes_per_minute {
            query = query.bind(v);
        }
        if let Some(ref v) = data.rate_limit_window_seconds {
            query = query.bind(v);
        }
        if let Some(ref v) = data.vote_weight_default {
            query = query.bind(v);
        }
        if let Some(ref v) = data.is_active {
            query = query.bind(v);
        }

        let row = query
            .bind(existing.id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                LeaderboardError::NotFound("VotingConfig".into(), existing.id.to_string())
            })?;

        cache::invalidate_voting_config_cache(conn.cloned())
            .await
            .ok();

        log::info!("Updated voting config {}", existing.id);

        Ok(row.into())
    }

    /// Get the active voting configuration.
    ///
    /// # Expected Behavior
    ///
    /// Queries `leaderboard.voting_config` where `is_active = true`.
    /// Checks Redis cache first; on cache miss, queries the database
    /// and caches the result with a 5-minute TTL.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::NotFound` if no active voting config exists.
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `leaderboard:voting_config` (read).
    /// - Queries `leaderboard.voting_config` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn get_active_voting_config(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
    ) -> Result<VotingConfigResponse, LeaderboardError> {
        if let Ok(Some(cached)) = cache::get_voting_config_from_cache(conn.cloned()).await {
            if let Ok(parsed) = serde_json::from_str::<VotingConfigResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let config = sqlx::query_as::<_, VotingConfig>(
            "SELECT * FROM leaderboard.voting_config WHERE is_active = true LIMIT 1",
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            LeaderboardError::NotFound("VotingConfig".into(), "no active config".into())
        })?;

        let response: VotingConfigResponse = config.into();

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_voting_config_cache(conn.cloned(), &json)
                .await
                .ok();
        }

        Ok(response)
    }

    /// Create a leaderboard snapshot.
    ///
    /// # Expected Behavior
    ///
    /// Reads the current leaderboard state from `leaderboard.ranks`,
    /// serializes it as JSONB, and inserts a snapshot row. Returns the
    /// created snapshot as a `SnapshotResponse`. Invalidates the
    /// snapshots cache after creation.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `leaderboard.ranks` (database read).
    /// - Inserts a row into `leaderboard.snapshots` (database write).
    /// - Invalidates `leaderboard:snapshots` Redis cache (write).
    /// - Logs at INFO level on successful creation.
    pub async fn create_snapshot(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        data: &SnapshotCreate,
    ) -> Result<SnapshotResponse, LeaderboardError> {
        let ranks = sqlx::query_as::<_, Rank>(
            "SELECT * FROM leaderboard.ranks ORDER BY rank ASC NULLS LAST, combined_score DESC NULLS LAST, total_score DESC",
        )
        .fetch_all(pool)
        .await?;

        let team_count = i32::try_from(ranks.len()).unwrap_or(0);
        let snapshot_data = serde_json::to_value(&ranks).unwrap_or(serde_json::Value::Null);

        let row = sqlx::query_as::<_, Snapshot>(
            "INSERT INTO leaderboard.snapshots (name, phase_id, formula_id, snapshot_data, team_count)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
        )
        .bind(&data.name)
        .bind(data.phase_id)
        .bind(data.formula_id)
        .bind(&snapshot_data)
        .bind(team_count)
        .fetch_one(pool)
        .await?;

        let snapshot_id = row.id;
        let response = row.into();

        cache::invalidate_snapshots_cache(conn.cloned()).await.ok();

        log::info!("Created snapshot {snapshot_id}");

        Ok(response)
    }

    /// List all leaderboard snapshots.
    ///
    /// # Expected Behavior
    ///
    /// Queries `leaderboard.snapshots` ordered by `created_at` descending.
    /// Checks Redis cache first; on cache miss, queries the database
    /// and caches the result with a 5-minute TTL.
    ///
    /// # Errors
    ///
    /// Returns `LeaderboardError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `leaderboard:snapshots` (read).
    /// - Queries `leaderboard.snapshots` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn list_snapshots(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
    ) -> Result<SnapshotListResponse, LeaderboardError> {
        if let Ok(Some(cached)) = cache::get_snapshots_from_cache(conn.cloned()).await {
            if let Ok(parsed) = serde_json::from_str::<SnapshotListResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard.snapshots")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let snapshots = sqlx::query_as::<_, Snapshot>(
            "SELECT * FROM leaderboard.snapshots ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;

        let response = SnapshotListResponse {
            snapshots: snapshots.into_iter().map(Into::into).collect(),
            total,
        };

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_snapshots_cache(conn.cloned(), &json).await.ok();
        }

        Ok(response)
    }
}

use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::SponsorError;
use crate::models::booth::Booth;
use crate::models::prize::{
    Prize, PrizeAnnounce, PrizeCreate, PrizeListResponse, PrizeResponse, PrizeUpdate,
};
use crate::services::cache;

pub struct PrizeService;

impl PrizeService {
    /// Create a new prize for a booth.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the booth exists, then inserts a new row into
    /// `sponsor.prizes` with the provided data. `announced` defaults to
    /// `false`. Returns the created prize as a `PrizeResponse`. Invalidates
    /// the prizes cache for the booth after creation.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists with the given
    /// `booth_id`.
    /// Returns `SponsorError::Database` if the insert fails (e.g., foreign
    /// key constraint violation).
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.booths` to verify existence (database read).
    /// - Inserts a row into `sponsor.prizes` (database write).
    /// - Invalidates `sponsor:booth:{boothId}:prizes` Redis cache (write).
    pub async fn create_prize(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        booth_id: Uuid,
        data: &PrizeCreate,
    ) -> Result<PrizeResponse, SponsorError> {
        let booth_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sponsor.booths WHERE id = $1)")
                .bind(booth_id)
                .fetch_one(pool)
                .await
                .unwrap_or(false);

        if !booth_exists {
            return Err(SponsorError::NotFound("Booth".into(), booth_id.to_string()));
        }

        let row = sqlx::query_as::<_, Prize>(
            "INSERT INTO sponsor.prizes (booth_id, title, description, value_usd, criteria)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
        )
        .bind(booth_id)
        .bind(&data.title)
        .bind(&data.description)
        .bind(data.value_usd)
        .bind(&data.criteria)
        .fetch_one(pool)
        .await?;

        let prize_id = row.id;
        let response = row.into();

        cache::invalidate_prizes_cache(conn.cloned(), &booth_id)
            .await
            .ok();
        log::info!("Created prize {prize_id} for booth {booth_id}");

        Ok(response)
    }

    /// List all prizes for a booth.
    ///
    /// # Expected Behavior
    ///
    /// Checks Redis cache first (`sponsor:booth:{boothId}:prizes`); on
    /// cache miss, queries `sponsor.prizes` for all prizes belonging to
    /// the given `booth_id`, ordered by `created_at` descending. Caches
    /// the result with a 5-minute TTL on cache miss. Returns total count
    /// for pagination.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` if the database query fails.
    /// Falls back to database on Redis errors.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `sponsor:booth:{boothId}:prizes` (read).
    /// - Queries `sponsor.prizes` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn list_prizes(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        booth_id: Uuid,
    ) -> Result<PrizeListResponse, SponsorError> {
        if let Ok(Some(cached)) = cache::get_prizes_from_cache(conn.cloned(), &booth_id).await {
            if let Ok(parsed) = serde_json::from_str::<PrizeListResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sponsor.prizes WHERE booth_id = $1")
                .bind(booth_id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let prizes = sqlx::query_as::<_, Prize>(
            "SELECT * FROM sponsor.prizes WHERE booth_id = $1 ORDER BY created_at DESC",
        )
        .bind(booth_id)
        .fetch_all(pool)
        .await?;

        let response = PrizeListResponse {
            prizes: prizes.into_iter().map(Into::into).collect(),
            total,
        };

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_prizes_cache(conn.cloned(), &booth_id, &json)
                .await
                .ok();
        }

        Ok(response)
    }

    /// Update an existing prize.
    ///
    /// # Expected Behavior
    ///
    /// Updates only the fields provided in `PrizeUpdate`. Returns the
    /// updated prize as a `PrizeResponse`. Invalidates the prizes cache
    /// for the prize's booth after update.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no prize exists with the given ID.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates row in `sponsor.prizes` (database write).
    /// - Invalidates `sponsor:booth:{boothId}:prizes` Redis cache (write).
    pub async fn update_prize(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        data: &PrizeUpdate,
    ) -> Result<PrizeResponse, SponsorError> {
        let existing = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Prize".into(), id.to_string()))?;

        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.title.is_some() {
            updates.push(format!("title = ${param_idx}"));
            param_idx += 1;
        }
        if data.description.is_some() {
            updates.push(format!("description = ${param_idx}"));
            param_idx += 1;
        }
        if data.value_usd.is_some() {
            updates.push(format!("value_usd = ${param_idx}"));
            param_idx += 1;
        }
        if data.criteria.is_some() {
            updates.push(format!("criteria = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Ok(existing.into());
        }

        let sql = format!(
            "UPDATE sponsor.prizes SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Prize>(&sql);

        if let Some(ref v) = data.title {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.value_usd {
            query = query.bind(v);
        }
        if let Some(ref v) = data.criteria {
            query = query.bind(v);
        }

        let row = query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Prize".into(), id.to_string()))?;

        cache::invalidate_prizes_cache(conn.cloned(), &existing.booth_id)
            .await
            .ok();
        log::info!("Updated prize {id}");

        Ok(row.into())
    }

    /// Delete a prize by ID.
    ///
    /// # Expected Behavior
    ///
    /// Deletes the prize row from `sponsor.prizes`. Returns `Ok(true)` if
    /// a row was deleted, `Ok(false)` if no row existed with the given ID.
    /// Invalidates the prizes cache for the prize's booth after deletion.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Deletes row from `sponsor.prizes` (database write).
    /// - Cascading deletes may affect `sponsor.submissions` depending on
    ///   foreign key constraints.
    /// - Invalidates `sponsor:booth:{boothId}:prizes` Redis cache (write).
    pub async fn delete_prize(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
    ) -> Result<bool, SponsorError> {
        let existing = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        let booth_id = match existing {
            Some(ref p) => p.booth_id,
            None => return Ok(false),
        };

        let result = sqlx::query("DELETE FROM sponsor.prizes WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            cache::invalidate_prizes_cache(conn.cloned(), &booth_id)
                .await
                .ok();
            log::info!("Deleted prize {id}");
        }

        Ok(deleted)
    }

    /// List all prizes belonging to booths owned by the given sponsor.
    ///
    /// # Expected Behavior
    ///
    /// Joins `sponsor.prizes` with `sponsor.booths` on `booth_id` and queries
    /// for all prizes where the booth's `sponsor_id` matches the provided
    /// value. Returns a `PrizeListResponse` ordered by `created_at` descending
    /// with total count for pagination.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.prizes` and `sponsor.booths` (database reads).
    pub async fn list_sponsor_prizes(
        pool: &PgPool,
        sponsor_id: &str,
    ) -> Result<PrizeListResponse, SponsorError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sponsor.prizes p JOIN sponsor.booths b ON p.booth_id = b.id WHERE b.sponsor_id = $1",
        )
        .bind(sponsor_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let prizes = sqlx::query_as::<_, Prize>(
            "SELECT p.* FROM sponsor.prizes p JOIN sponsor.booths b ON p.booth_id = b.id WHERE b.sponsor_id = $1 ORDER BY p.created_at DESC",
        )
        .bind(sponsor_id)
        .fetch_all(pool)
        .await?;

        Ok(PrizeListResponse {
            prizes: prizes.into_iter().map(Into::into).collect(),
            total,
        })
    }

    /// Create a prize for the sponsor's booth.
    ///
    /// # Expected Behavior
    ///
    /// Finds the booth owned by the given `sponsor_id`. If found, creates a
    /// prize for that booth. Returns the created prize as a `PrizeResponse`.
    /// Invalidates the prizes cache for the booth after creation.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists for the given
    /// `sponsor_id`.
    /// Returns `SponsorError::Database` if the insert fails.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.booths` to find the sponsor's booth (database read).
    /// - Inserts a row into `sponsor.prizes` (database write).
    /// - Invalidates `sponsor:booth:{boothId}:prizes` Redis cache (write).
    pub async fn create_sponsor_prize(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        sponsor_id: &str,
        data: &PrizeCreate,
    ) -> Result<PrizeResponse, SponsorError> {
        let booth = sqlx::query_as::<_, Booth>(
            "SELECT * FROM sponsor.booths WHERE sponsor_id = $1 LIMIT 1",
        )
        .bind(sponsor_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            SponsorError::NotFound("Booth".into(), format!("sponsor_id {sponsor_id}"))
        })?;

        let booth_id = booth.id;
        Self::create_prize(pool, conn, booth_id, data).await
    }

    /// Update a prize with ownership verification.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the prize's booth is owned by the given `sponsor_id` by
    /// fetching the prize then the booth. If the ownership check passes,
    /// delegates to `update_prize` for the actual update. Returns the updated
    /// prize as a `PrizeResponse`.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no prize exists with the given ID.
    /// Returns `SponsorError::Forbidden` if the prize's booth is not owned by
    /// the sponsor.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.prizes` and `sponsor.booths` to verify ownership
    ///   (database reads).
    /// - Delegates to `update_prize` which performs the update and cache
    ///   invalidation (database write, Redis writes).
    pub async fn update_prize_owner_check(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        sponsor_id: &str,
        data: &PrizeUpdate,
    ) -> Result<PrizeResponse, SponsorError> {
        let prize = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Prize".into(), id.to_string()))?;

        let booth = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
            .bind(prize.booth_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Booth".into(), prize.booth_id.to_string()))?;

        match &booth.sponsor_id {
            Some(sid) if sid == sponsor_id => {}
            _ => {
                return Err(SponsorError::Forbidden("You do not own this prize".into()));
            }
        }

        Self::update_prize(pool, conn, id, data).await
    }

    /// Delete a prize with ownership verification.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the prize's booth is owned by the given `sponsor_id`. If
    /// the ownership check passes, delegates to `delete_prize` for the actual
    /// deletion. Returns `Ok(true)` if deleted, `Ok(false)` if the prize does
    /// not exist.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Forbidden` if the prize's booth is not owned by
    /// the sponsor.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.prizes` and `sponsor.booths` to verify ownership
    ///   (database reads).
    /// - Delegates to `delete_prize` which performs the deletion and cache
    ///   invalidation (database write, Redis writes).
    pub async fn delete_prize_owner_check(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        sponsor_id: &str,
    ) -> Result<bool, SponsorError> {
        let prize = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        let booth_id = match prize {
            Some(ref p) => p.booth_id,
            None => return Ok(false),
        };

        let booth = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
            .bind(booth_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Booth".into(), booth_id.to_string()))?;

        match &booth.sponsor_id {
            Some(sid) if sid == sponsor_id => {}
            _ => {
                return Err(SponsorError::Forbidden("You do not own this prize".into()));
            }
        }

        Self::delete_prize(pool, conn, id).await
    }

    /// Select a winner for a prize with ownership verification.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the prize's booth is owned by the given `sponsor_id`.
    /// Then verifies a submission exists for the given (`prize_id`, `team_id`)
    /// pair. Sets the submission's status to 'awarded' and updates the
    /// prize's `winner_project_id` and `announced = true`. Returns the
    /// updated prize as a `PrizeResponse`. Invalidates the prizes cache
    /// after the update.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no prize exists with the given ID,
    /// or if no submission exists for the (`prize_id`, `team_id`) pair.
    /// Returns `SponsorError::Forbidden` if the prize's booth is not owned by
    /// the sponsor.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.prizes` and `sponsor.booths` to verify ownership
    ///   (database reads).
    /// - Reads from `sponsor.submissions` to verify existence (database read).
    /// - Updates submission status to 'awarded' (database write).
    /// - Updates prize `winner_project_id` and `announced` (database write).
    /// - Invalidates `sponsor:booth:{boothId}:prizes` Redis cache (write).
    pub async fn select_sponsor_winner(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        prize_id: Uuid,
        sponsor_id: &str,
        team_id: Uuid,
    ) -> Result<PrizeResponse, SponsorError> {
        let existing = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
            .bind(prize_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Prize".into(), prize_id.to_string()))?;

        let booth = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
            .bind(existing.booth_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Booth".into(), existing.booth_id.to_string()))?;

        match &booth.sponsor_id {
            Some(sid) if sid == sponsor_id => {}
            _ => {
                return Err(SponsorError::Forbidden("You do not own this prize".into()));
            }
        }

        let submission_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sponsor.submissions WHERE prize_id = $1 AND project_id = $2)",
        )
        .bind(prize_id)
        .bind(team_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !submission_exists {
            return Err(SponsorError::NotFound(
                "Submission".into(),
                format!("prize {prize_id} team {team_id}"),
            ));
        }

        sqlx::query(
            "UPDATE sponsor.submissions SET status = 'awarded' WHERE prize_id = $1 AND project_id = $2",
        )
        .bind(prize_id)
        .bind(team_id)
        .execute(pool)
        .await?;

        let row = sqlx::query_as::<_, Prize>(
            "UPDATE sponsor.prizes SET winner_project_id = $1, announced = true WHERE id = $2 RETURNING *",
        )
        .bind(team_id)
        .bind(prize_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Prize".into(), prize_id.to_string()))?;

        cache::invalidate_prizes_cache(conn.cloned(), &existing.booth_id)
            .await
            .ok();
        log::info!("Selected winner for prize {prize_id} (sponsor identity)");

        Ok(row.into())
    }

    /// Announce a winner for a prize.
    ///
    /// # Expected Behavior
    ///
    /// Sets `winner_project_id` and `announced = true` on the prize.
    /// Returns the updated prize as a `PrizeResponse`. Invalidates the
    /// prizes cache for the prize's booth after the update.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no prize exists with the given ID.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates row in `sponsor.prizes` (database write).
    /// - Invalidates `sponsor:booth:{boothId}:prizes` Redis cache (write).
    pub async fn announce_winner(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        data: &PrizeAnnounce,
    ) -> Result<PrizeResponse, SponsorError> {
        let existing = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Prize".into(), id.to_string()))?;

        let row = sqlx::query_as::<_, Prize>(
            "UPDATE sponsor.prizes SET winner_project_id = $1, announced = true WHERE id = $2 RETURNING *",
        )
        .bind(data.winner_project_id)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Prize".into(), id.to_string()))?;

        cache::invalidate_prizes_cache(conn.cloned(), &existing.booth_id)
            .await
            .ok();
        log::info!("Announced winner for prize {id}");

        Ok(row.into())
    }

    /// Select a winner for a prize from the submissions.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that a submission exists for the given (`prize_id`, `project_id`)
    /// pair, then sets `winner_project_id` and `announced = true` on the
    /// prize. Returns the updated prize as a `PrizeResponse`. Invalidates
    /// the prizes cache after the update.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no prize exists with the given ID,
    /// or if no submission exists for the given (`prize_id`, `project_id`) pair.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.submissions` to verify existence (database read).
    /// - Updates row in `sponsor.prizes` (database write).
    /// - Invalidates `sponsor:booth:{boothId}:prizes` Redis cache (write).
    pub async fn select_winner(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        prize_id: Uuid,
        project_id: Uuid,
    ) -> Result<PrizeResponse, SponsorError> {
        let submission_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sponsor.submissions WHERE prize_id = $1 AND project_id = $2)",
        )
        .bind(prize_id)
        .bind(project_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !submission_exists {
            return Err(SponsorError::NotFound(
                "Submission".into(),
                format!("prize {prize_id} project {project_id}"),
            ));
        }

        let existing = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
            .bind(prize_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Prize".into(), prize_id.to_string()))?;

        let row = sqlx::query_as::<_, Prize>(
            "UPDATE sponsor.prizes SET winner_project_id = $1, announced = true WHERE id = $2 RETURNING *",
        )
        .bind(project_id)
        .bind(prize_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Prize".into(), prize_id.to_string()))?;

        cache::invalidate_prizes_cache(conn.cloned(), &existing.booth_id)
            .await
            .ok();
        log::info!("Selected winner for prize {prize_id}");

        Ok(row.into())
    }
}

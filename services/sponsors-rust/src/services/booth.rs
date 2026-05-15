use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::SponsorError;
use crate::models::booth::{
    Booth, BoothAnalyticsResponse, BoothCreate, BoothListResponse, BoothPublish, BoothResponse,
    BoothUpdate,
};
use crate::services::cache;

pub struct BoothService;

impl BoothService {
    /// Create a new sponsor booth.
    ///
    /// # Expected Behavior
    ///
    /// Inserts a new row into `sponsor.booths` with the provided data.
    /// `published` defaults to `false`, `view_count` defaults to 0.
    /// Returns the created booth as a `BoothResponse`. Invalidates the
    /// published booths cache after creation. If `conn` is `None`, cache
    /// invalidation is skipped.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` if the insert fails (e.g., constraint
    /// violation).
    ///
    /// # Side Effects
    ///
    /// - Inserts a row into `sponsor.booths` (database write).
    /// - Invalidates `sponsor:booths:published` Redis cache (write), if Redis available.
    pub async fn create_booth(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        data: &BoothCreate,
    ) -> Result<BoothResponse, SponsorError> {
        let row = sqlx::query_as::<_, Booth>(
            "INSERT INTO sponsor.booths (sponsor_name, tagline, description, logo_url, banner_url, website_url, careers_url, api_docs_url, technologies, contact_email, discord_channel, theme_colors)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING *",
        )
        .bind(&data.sponsor_name)
        .bind(&data.tagline)
        .bind(&data.description)
        .bind(&data.logo_url)
        .bind(&data.banner_url)
        .bind(&data.website_url)
        .bind(&data.careers_url)
        .bind(&data.api_docs_url)
        .bind(&data.technologies)
        .bind(&data.contact_email)
        .bind(&data.discord_channel)
        .bind(&data.theme_colors)
        .fetch_one(pool)
        .await?;

        let booth_id = row.id;
        let response = row.into();

        cache::invalidate_published_booths_cache(conn.cloned())
            .await
            .ok();
        log::info!("Created booth {booth_id}");

        Ok(response)
    }

    /// List all published booths.
    ///
    /// # Expected Behavior
    ///
    /// Queries `sponsor.booths` where `published = true`, ordered by
    /// `created_at` descending with pagination. Checks Redis cache first
    /// (`sponsor:booths:published`); on cache miss, queries the database
    /// and caches the result with a 1-minute TTL. Returns total count for
    /// pagination. If `conn` is `None`, skips cache reads/writes and always
    /// queries the database.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` if the database query fails.
    /// Falls back to database on Redis errors.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `sponsor:booths:published` (read), if Redis available.
    /// - Queries `sponsor.booths` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write), if Redis available.
    pub async fn list_published_booths(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        limit: i64,
        offset: i64,
    ) -> Result<BoothListResponse, SponsorError> {
        if let Ok(Some(cached)) = cache::get_published_booths_from_cache(conn.cloned()).await {
            if let Ok(parsed) = serde_json::from_str::<BoothListResponse>(&cached) {
                return Ok(parsed);
            }
        }

        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sponsor.booths WHERE published = true")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let booths = sqlx::query_as::<_, Booth>(
            "SELECT * FROM sponsor.booths WHERE published = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let response = BoothListResponse {
            booths: booths.into_iter().map(Into::into).collect(),
            total,
        };

        if let Ok(json) = serde_json::to_string(&response) {
            cache::set_published_booths_cache(conn.cloned(), &json)
                .await
                .ok();
        }

        Ok(response)
    }

    /// Get a single booth by ID, incrementing the view count.
    ///
    /// # Expected Behavior
    ///
    /// Checks Redis cache first (`sponsor:booth:{id}`); on cache miss,
    /// queries the database. Increments `view_count` by 1 in the database
    /// regardless of cache hit. Returns the booth as a `BoothResponse`.
    /// Caches the result with a 5-minute TTL on cache miss.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists with the given ID.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from Redis cache `sponsor:booth:{id}` (read).
    /// - Increments `view_count` in `sponsor.booths` (database write).
    /// - Queries `sponsor.booths` on cache miss (database read).
    /// - Writes to Redis cache on cache miss (write).
    pub async fn get_booth(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
    ) -> Result<BoothResponse, SponsorError> {
        let _ = sqlx::query(
            "UPDATE sponsor.booths SET view_count = COALESCE(view_count, 0) + 1, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await;

        let booth = if let Ok(Some(cached)) = cache::get_booth_from_cache(conn.cloned(), &id).await
        {
            cached
        } else {
            let row = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| SponsorError::NotFound("Booth".into(), id.to_string()))?;

            cache::set_booth_cache(conn.cloned(), &row).await.ok();
            row
        };

        let mut response: BoothResponse = booth.into();
        response.view_count += 1;

        Ok(response)
    }

    /// Update an existing booth.
    ///
    /// # Expected Behavior
    ///
    /// Updates only the fields provided in `BoothUpdate`. If no fields
    /// are provided, only `updated_at` is refreshed. Returns the updated
    /// booth as a `BoothResponse`. Invalidates the booth cache, published
    /// booths cache, and prizes cache after update.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists with the given ID.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates row in `sponsor.booths` (database write).
    /// - Invalidates `sponsor:booth:{id}`, `sponsor:booths:published`,
    ///   and `sponsor:booth:{id}:prizes` Redis caches (writes).
    #[allow(clippy::too_many_lines)]
    pub async fn update_booth(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        data: &BoothUpdate,
    ) -> Result<BoothResponse, SponsorError> {
        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.sponsor_name.is_some() {
            updates.push(format!("sponsor_name = ${param_idx}"));
            param_idx += 1;
        }
        if data.tagline.is_some() {
            updates.push(format!("tagline = ${param_idx}"));
            param_idx += 1;
        }
        if data.description.is_some() {
            updates.push(format!("description = ${param_idx}"));
            param_idx += 1;
        }
        if data.logo_url.is_some() {
            updates.push(format!("logo_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.banner_url.is_some() {
            updates.push(format!("banner_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.website_url.is_some() {
            updates.push(format!("website_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.careers_url.is_some() {
            updates.push(format!("careers_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.api_docs_url.is_some() {
            updates.push(format!("api_docs_url = ${param_idx}"));
            param_idx += 1;
        }
        if data.technologies.is_some() {
            updates.push(format!("technologies = ${param_idx}"));
            param_idx += 1;
        }
        if data.contact_email.is_some() {
            updates.push(format!("contact_email = ${param_idx}"));
            param_idx += 1;
        }
        if data.discord_channel.is_some() {
            updates.push(format!("discord_channel = ${param_idx}"));
            param_idx += 1;
        }
        if data.theme_colors.is_some() {
            updates.push(format!("theme_colors = ${param_idx}"));
            param_idx += 1;
        }

        updates.push("updated_at = NOW()".to_string());

        if updates.len() == 1 {
            let row = sqlx::query_as::<_, Booth>(
                "UPDATE sponsor.booths SET updated_at = NOW() WHERE id = $1 RETURNING *",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Booth".into(), id.to_string()))?;

            cache::invalidate_booth_caches(conn.cloned(), &id).await;
            return Ok(row.into());
        }

        let sql = format!(
            "UPDATE sponsor.booths SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Booth>(&sql);

        if let Some(ref v) = data.sponsor_name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.tagline {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.logo_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.banner_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.website_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.careers_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.api_docs_url {
            query = query.bind(v);
        }
        if let Some(ref v) = data.technologies {
            query = query.bind(v);
        }
        if let Some(ref v) = data.contact_email {
            query = query.bind(v);
        }
        if let Some(ref v) = data.discord_channel {
            query = query.bind(v);
        }
        if let Some(ref v) = data.theme_colors {
            query = query.bind(v);
        }

        let row = query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Booth".into(), id.to_string()))?;

        cache::invalidate_booth_caches(conn.cloned(), &id).await;
        log::info!("Updated booth {id}");

        Ok(row.into())
    }

    /// Delete a booth by ID.
    ///
    /// # Expected Behavior
    ///
    /// Deletes the booth row from `sponsor.booths`. Returns `Ok(true)` if
    /// a row was deleted, `Ok(false)` if no row existed with the given ID.
    /// Invalidates all related caches after deletion.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Deletes row from `sponsor.booths` (database write).
    /// - Cascading deletes may affect `sponsor.prizes` and
    ///   `sponsor.submissions` depending on foreign key constraints.
    /// - Invalidates `sponsor:booth:{id}`, `sponsor:booths:published`,
    ///   and `sponsor:booth:{id}:prizes` Redis caches (writes).
    pub async fn delete_booth(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
    ) -> Result<bool, SponsorError> {
        let result = sqlx::query("DELETE FROM sponsor.booths WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            cache::invalidate_booth_caches(conn.cloned(), &id).await;
            log::info!("Deleted booth {id}");
        }

        Ok(deleted)
    }

    /// Publish or unpublish a booth.
    ///
    /// # Expected Behavior
    ///
    /// Toggles or sets the `published` flag on the booth. If `BoothPublish.published`
    /// is `Some(bool)`, sets it to that value. If `None`, toggles the current value.
    /// Returns the updated booth as a `BoothResponse`. Invalidates all related
    /// caches after the update.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists with the given ID.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Updates `published` column in `sponsor.booths` (database write).
    /// - Updates `updated_at` column (database write).
    /// - Invalidates `sponsor:booth:{id}`, `sponsor:booths:published`,
    ///   and `sponsor:booth:{id}:prizes` Redis caches (writes).
    pub async fn publish_booth(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        data: &BoothPublish,
    ) -> Result<BoothResponse, SponsorError> {
        let row = match data.published {
            Some(val) => {
                sqlx::query_as::<_, Booth>(
                    "UPDATE sponsor.booths SET published = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
                )
                .bind(val)
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| SponsorError::NotFound("Booth".into(), id.to_string()))?
            }
            None => {
                sqlx::query_as::<_, Booth>(
                    "UPDATE sponsor.booths SET published = NOT COALESCE(published, false), updated_at = NOW() WHERE id = $1 RETURNING *",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| SponsorError::NotFound("Booth".into(), id.to_string()))?
            }
        };

        cache::invalidate_booth_caches(conn.cloned(), &id).await;
        log::info!("Toggled publish state for booth {id}");

        Ok(row.into())
    }

    /// Get the booth owned by the given sponsor.
    ///
    /// # Expected Behavior
    ///
    /// Queries `sponsor.booths` for the first booth where `sponsor_id`
    /// matches the provided value. Returns `SponsorError::NotFound` if no
    /// booth exists for the given sponsor. Returns the booth as a
    /// `BoothResponse`.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists with the given
    /// `sponsor_id`.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.booths` (database read).
    pub async fn get_sponsor_booth(
        pool: &PgPool,
        sponsor_id: &str,
    ) -> Result<BoothResponse, SponsorError> {
        let row = sqlx::query_as::<_, Booth>(
            "SELECT * FROM sponsor.booths WHERE sponsor_id = $1 LIMIT 1",
        )
        .bind(sponsor_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            SponsorError::NotFound("Booth".into(), format!("sponsor_id {sponsor_id}"))
        })?;

        Ok(row.into())
    }

    /// Create a new booth with `sponsor_id` set from JWT identity.
    ///
    /// # Expected Behavior
    ///
    /// Inserts a new row into `sponsor.booths` with `sponsor_id` set to the
    /// provided value. All other fields are populated from the `BoothCreate`
    /// data. `published` defaults to `false`, `view_count` defaults to 0.
    /// Returns the created booth as a `BoothResponse`. Invalidates the
    /// published booths cache after creation.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` if the insert fails (e.g., constraint
    /// violation or duplicate `sponsor_id` if a unique constraint exists).
    ///
    /// # Side Effects
    ///
    /// - Inserts a row into `sponsor.booths` with `sponsor_id` (database write).
    /// - Invalidates `sponsor:booths:published` Redis cache (write).
    pub async fn create_sponsor_booth(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        sponsor_id: &str,
        data: &BoothCreate,
    ) -> Result<BoothResponse, SponsorError> {
        let row = sqlx::query_as::<_, Booth>(
            "INSERT INTO sponsor.booths (sponsor_id, sponsor_name, tagline, description, logo_url, banner_url, website_url, careers_url, api_docs_url, technologies, contact_email, discord_channel, theme_colors)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING *",
        )
        .bind(sponsor_id)
        .bind(&data.sponsor_name)
        .bind(&data.tagline)
        .bind(&data.description)
        .bind(&data.logo_url)
        .bind(&data.banner_url)
        .bind(&data.website_url)
        .bind(&data.careers_url)
        .bind(&data.api_docs_url)
        .bind(&data.technologies)
        .bind(&data.contact_email)
        .bind(&data.discord_channel)
        .bind(&data.theme_colors)
        .fetch_one(pool)
        .await?;

        let booth_id = row.id;
        let response = row.into();

        cache::invalidate_published_booths_cache(conn.cloned())
            .await
            .ok();
        log::info!("Created booth {booth_id} for sponsor {sponsor_id}");

        Ok(response)
    }

    /// Update a booth with ownership verification.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the booth with the given ID has `sponsor_id` matching
    /// the provided `sponsor_id`. If the ownership check passes, delegates to
    /// `update_booth` for the actual update. Returns the updated booth as a
    /// `BoothResponse`.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists with the given ID.
    /// Returns `SponsorError::Forbidden` if the booth's `sponsor_id` does not
    /// match the provided `sponsor_id`.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.booths` to verify ownership (database read).
    /// - Delegates to `update_booth` which performs the update and cache
    ///   invalidation (database write, Redis writes).
    pub async fn update_booth_owner_check(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        sponsor_id: &str,
        data: &BoothUpdate,
    ) -> Result<BoothResponse, SponsorError> {
        let booth = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Booth".into(), id.to_string()))?;

        match &booth.sponsor_id {
            Some(sid) if sid == sponsor_id => {}
            _ => {
                return Err(SponsorError::Forbidden("You do not own this booth".into()));
            }
        }

        Self::update_booth(pool, conn, id, data).await
    }

    /// Delete a booth with ownership verification.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the booth with the given ID has `sponsor_id` matching
    /// the provided `sponsor_id`. If the ownership check passes, delegates to
    /// `delete_booth` for the actual deletion. Returns `Ok(true)` if deleted,
    /// `Ok(false)` if the booth does not exist.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Forbidden` if the booth's `sponsor_id` does not
    /// match the provided `sponsor_id`.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.booths` to verify ownership (database read).
    /// - Delegates to `delete_booth` which performs the deletion and cache
    ///   invalidation (database write, Redis writes).
    pub async fn delete_booth_owner_check(
        pool: &PgPool,
        conn: Option<&MultiplexedConnection>,
        id: Uuid,
        sponsor_id: &str,
    ) -> Result<bool, SponsorError> {
        let booth = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        match booth {
            Some(b) => match &b.sponsor_id {
                Some(sid) if sid == sponsor_id => {}
                _ => {
                    return Err(SponsorError::Forbidden("You do not own this booth".into()));
                }
            },
            None => return Ok(false),
        }

        Self::delete_booth(pool, conn, id).await
    }

    /// Get analytics for a booth (view count, prize count, submission count).
    ///
    /// # Expected Behavior
    ///
    /// Queries the database for the booth's `view_count`, counts the number
    /// of prizes associated with the booth, and counts the total number of
    /// submissions across all of the booth's prizes. Returns a
    /// `BoothAnalyticsResponse`.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no booth exists with the given ID.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.booths` (database read).
    /// - Reads from `sponsor.prizes` (database read).
    /// - Reads from `sponsor.submissions` (database read).
    pub async fn get_booth_analytics(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<BoothAnalyticsResponse, SponsorError> {
        let booth = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| SponsorError::NotFound("Booth".into(), id.to_string()))?;

        let prize_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sponsor.prizes WHERE booth_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let submission_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sponsor.submissions s JOIN sponsor.prizes p ON s.prize_id = p.id WHERE p.booth_id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(BoothAnalyticsResponse {
            booth_id: id,
            view_count: booth.view_count.unwrap_or(0),
            prize_count,
            submission_count,
        })
    }
}

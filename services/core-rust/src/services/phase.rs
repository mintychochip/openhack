use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::CoreError;
use crate::models::phase::{
    HackathonPhase, NextTransitionResponse, PhaseCreate, PhaseResponse, PhaseUpdate,
};

pub struct PhaseService;

impl PhaseService {
    /// Create a new hackathon phase.
    ///
    /// # Expected Behavior
    ///
    /// Inserts a new row into `core.hackathon_phases`. Returns the created phase.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on insert failure.
    ///
    /// # Side Effects
    ///
    /// - Inserts a row into `core.hackathon_phases` (database write).
    pub async fn create_phase(
        pool: &PgPool,
        data: &PhaseCreate,
    ) -> Result<PhaseResponse, CoreError> {
        let phase = sqlx::query_as::<_, HackathonPhase>(
            "INSERT INTO core.hackathon_phases (hackathon_id, name, type, description, opens_at, closes_at, is_active, config) VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7) RETURNING id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at",
        )
        .bind(data.hackathon_id)
        .bind(&data.name)
        .bind(&data.r#type)
        .bind(&data.description)
        .bind(data.opens_at)
        .bind(data.closes_at)
        .bind(&data.config)
        .fetch_one(pool)
        .await?;

        log::info!(
            "Created phase {} for hackathon {}",
            phase.id,
            data.hackathon_id
        );
        Ok(phase.into())
    }

    /// Get a phase by ID.
    ///
    /// # Expected Behavior
    ///
    /// Fetches the phase. Returns `CoreError::NotFound` if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no phase exists with the given ID.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.hackathon_phases` (database read).
    pub async fn get_phase(pool: &PgPool, id: Uuid) -> Result<PhaseResponse, CoreError> {
        let phase = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Phase".into(), id.to_string()))?;

        Ok(phase.into())
    }

    /// List all phases.
    ///
    /// # Expected Behavior
    ///
    /// Returns all phases ordered by `opens_at` ascending.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.hackathon_phases` (database read).
    pub async fn list_phases(pool: &PgPool) -> Result<Vec<PhaseResponse>, CoreError> {
        let phases = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases ORDER BY opens_at ASC",
        )
        .fetch_all(pool)
        .await?;

        Ok(phases.into_iter().map(Into::into).collect())
    }

    /// Update a phase.
    ///
    /// # Expected Behavior
    ///
    /// Updates only the provided fields.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the phase does not exist.
    ///
    /// # Side Effects
    ///
    /// - Updates row in `core.hackathon_phases` (database write).
    pub async fn update_phase(
        pool: &PgPool,
        id: Uuid,
        data: &PhaseUpdate,
    ) -> Result<PhaseResponse, CoreError> {
        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.name.is_some() {
            updates.push(format!("name = ${param_idx}"));
            param_idx += 1;
        }
        if data.r#type.is_some() {
            updates.push(format!("type = ${param_idx}"));
            param_idx += 1;
        }
        if data.description.is_some() {
            updates.push(format!("description = ${param_idx}"));
            param_idx += 1;
        }
        if data.opens_at.is_some() {
            updates.push(format!("opens_at = ${param_idx}"));
            param_idx += 1;
        }
        if data.closes_at.is_some() {
            updates.push(format!("closes_at = ${param_idx}"));
            param_idx += 1;
        }
        if data.config.is_some() {
            updates.push(format!("config = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Self::get_phase(pool, id).await;
        }

        updates.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE core.hackathon_phases SET {} WHERE id = ${} RETURNING id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, HackathonPhase>(&sql);

        if let Some(ref v) = data.name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.r#type {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.opens_at {
            query = query.bind(v);
        }
        if let Some(ref v) = data.closes_at {
            query = query.bind(v);
        }
        if let Some(ref v) = data.config {
            query = query.bind(v);
        }

        let phase = query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| CoreError::NotFound("Phase".into(), id.to_string()))?;

        log::info!("Updated phase {id}");
        Ok(phase.into())
    }

    /// Delete a phase.
    ///
    /// # Expected Behavior
    ///
    /// Deletes the phase.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the phase does not exist.
    ///
    /// # Side Effects
    ///
    /// - Deletes from `core.hackathon_phases` (database write).
    pub async fn delete_phase(pool: &PgPool, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM core.hackathon_phases WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("Phase".into(), id.to_string()));
        }

        log::info!("Deleted phase {id}");
        Ok(())
    }

    /// Manually open a phase.
    ///
    /// # Expected Behavior
    ///
    /// Sets `is_active = TRUE` and `updated_at = NOW()` for the phase.
    /// Closes any other active phase for the same hackathon.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the phase does not exist.
    ///
    /// # Side Effects
    ///
    /// - Updates `core.hackathon_phases` (database write).
    pub async fn open_phase(pool: &PgPool, id: Uuid) -> Result<PhaseResponse, CoreError> {
        let phase = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Phase".into(), id.to_string()))?;

        sqlx::query(
            "UPDATE core.hackathon_phases SET is_active = FALSE, updated_at = NOW() WHERE hackathon_id = $1 AND is_active = TRUE",
        )
        .bind(phase.hackathon_id)
        .execute(pool)
        .await?;

        let updated = sqlx::query_as::<_, HackathonPhase>(
            "UPDATE core.hackathon_phases SET is_active = TRUE, updated_at = NOW() WHERE id = $1 RETURNING id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        log::info!("Opened phase {id}");
        Ok(updated.into())
    }

    /// Manually close a phase.
    ///
    /// # Expected Behavior
    ///
    /// Sets `is_active = FALSE` and `updated_at = NOW()`.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the phase does not exist.
    ///
    /// # Side Effects
    ///
    /// - Updates `core.hackathon_phases` (database write).
    pub async fn close_phase(pool: &PgPool, id: Uuid) -> Result<PhaseResponse, CoreError> {
        let updated = sqlx::query_as::<_, HackathonPhase>(
            "UPDATE core.hackathon_phases SET is_active = FALSE, updated_at = NOW() WHERE id = $1 RETURNING id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Phase".into(), id.to_string()))?;

        log::info!("Closed phase {id}");
        Ok(updated.into())
    }

    /// Get the current active phase for a hackathon.
    ///
    /// # Expected Behavior
    ///
    /// Returns the currently active phase for the given hackathon, or the
    /// phase that should be active based on `opens_at/closes_at` times.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no phase exists for the hackathon.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.hackathon_phases` (database read).
    pub async fn get_current_phase(
        pool: &PgPool,
        hackathon_id: Uuid,
    ) -> Result<Option<PhaseResponse>, CoreError> {
        let phase = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases WHERE hackathon_id = $1 AND is_active = TRUE ORDER BY opens_at DESC LIMIT 1",
        )
        .bind(hackathon_id)
        .fetch_optional(pool)
        .await?;

        Ok(phase.map(Into::into))
    }

    /// Get the next phase transition for a hackathon.
    ///
    /// # Expected Behavior
    ///
    /// Returns the next phase that will open or close, whichever is sooner.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no future transitions exist.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.hackathon_phases` (database read).
    pub async fn get_next_transition(
        pool: &PgPool,
        hackathon_id: Uuid,
    ) -> Result<Option<NextTransitionResponse>, CoreError> {
        let now = Utc::now();

        let next_open = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases WHERE hackathon_id = $1 AND is_active = FALSE AND opens_at > $2 ORDER BY opens_at ASC LIMIT 1",
        )
        .bind(hackathon_id)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        let next_close = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases WHERE hackathon_id = $1 AND is_active = TRUE AND closes_at > $2 ORDER BY closes_at ASC LIMIT 1",
        )
        .bind(hackathon_id)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        match (next_open, next_close) {
            (Some(open), Some(close)) => {
                let open_time = open.opens_at.unwrap_or_default();
                let close_time = close.closes_at.unwrap_or_default();
                if open_time < close_time {
                    Ok(Some(NextTransitionResponse {
                        phase: open.into(),
                        transition_at: open_time,
                        transition_type: "open".to_string(),
                    }))
                } else {
                    Ok(Some(NextTransitionResponse {
                        phase: close.into(),
                        transition_at: close_time,
                        transition_type: "close".to_string(),
                    }))
                }
            }
            (Some(open), None) => {
                let transition_at = open.opens_at.unwrap_or_default();
                Ok(Some(NextTransitionResponse {
                    phase: open.into(),
                    transition_at,
                    transition_type: "open".to_string(),
                }))
            }
            (None, Some(close)) => {
                let transition_at = close.closes_at.unwrap_or_default();
                Ok(Some(NextTransitionResponse {
                    phase: close.into(),
                    transition_at,
                    transition_type: "close".to_string(),
                }))
            }
            (None, None) => Ok(None),
        }
    }

    /// Check and auto-transition phases based on `opens_at/closes_at`.
    ///
    /// # Expected Behavior
    ///
    /// Called by the background scheduler every 30 seconds. Opens phases
    /// whose `opens_at` has passed and `is_active = FALSE`. Closes phases
    /// whose `closes_at` has passed and `is_active = TRUE`. Only one
    /// phase per hackathon can be active at a time; opening a new phase
    /// closes the previous one.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads and writes to `core.hackathon_phases` (database read/write).
    /// - Logs transitions at INFO level.
    pub async fn check_and_transition(pool: &PgPool) -> Result<(), CoreError> {
        let now = Utc::now();

        let to_close: Vec<HackathonPhase> = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases WHERE is_active = TRUE AND closes_at < $1",
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        for phase in &to_close {
            sqlx::query(
                "UPDATE core.hackathon_phases SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
            )
            .bind(phase.id)
            .execute(pool)
            .await?;
            log::info!("Scheduler closed phase {} ({})", phase.id, phase.name);
        }

        let to_open: Vec<HackathonPhase> = sqlx::query_as::<_, HackathonPhase>(
            "SELECT id, hackathon_id, name, type, description, opens_at, closes_at, is_active, config, created_at, updated_at FROM core.hackathon_phases WHERE is_active = FALSE AND opens_at <= $1 AND closes_at > $1",
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        for phase in &to_open {
            sqlx::query(
                "UPDATE core.hackathon_phases SET is_active = FALSE, updated_at = NOW() WHERE hackathon_id = $1 AND is_active = TRUE",
            )
            .bind(phase.hackathon_id)
            .execute(pool)
            .await?;

            sqlx::query(
                "UPDATE core.hackathon_phases SET is_active = TRUE, updated_at = NOW() WHERE id = $1",
            )
            .bind(phase.id)
            .execute(pool)
            .await?;
            log::info!("Scheduler opened phase {} ({})", phase.id, phase.name);
        }

        Ok(())
    }
}

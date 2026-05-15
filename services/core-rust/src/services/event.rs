use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::CoreError;
use crate::models::event::{
    AttendeeResponse, Event, EventCreate, EventListResponse, EventResponse, EventRsvp, EventUpdate,
};
use crate::services::publisher;

pub struct EventService;

impl EventService {
    /// Create a new event.
    ///
    /// # Expected Behavior
    ///
    /// Inserts a new row into `core.events`. Returns the created event
    /// with `rsvp_count` of 0.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on insert failure.
    ///
    /// # Side Effects
    ///
    /// - Inserts a row into `core.events` (database write).
    pub async fn create_event(
        pool: &PgPool,
        data: &EventCreate,
    ) -> Result<EventResponse, CoreError> {
        let event = sqlx::query_as::<_, Event>(
            "INSERT INTO core.events (title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at",
        )
        .bind(&data.title)
        .bind(&data.description)
        .bind(data.start_time)
        .bind(data.end_time)
        .bind(&data.location)
        .bind(&data.r#type)
        .bind(data.capacity)
        .bind(&data.speaker_name)
        .bind(&data.speaker_bio)
        .fetch_one(pool)
        .await?;

        log::info!("Created event {}", event.id);
        Ok(Self::event_to_response(event, 0))
    }

    /// Get an event by ID with RSVP count.
    ///
    /// # Expected Behavior
    ///
    /// Fetches the event and counts its RSVPs. Returns `CoreError::NotFound`
    /// if the event does not exist.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no event exists with the given ID.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.events` and `core.event_rsvps` (database reads).
    pub async fn get_event(pool: &PgPool, id: Uuid) -> Result<EventResponse, CoreError> {
        let event = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at FROM core.events WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Event".into(), id.to_string()))?;

        let rsvp_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.event_rsvps WHERE event_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        Ok(Self::event_to_response(event, rsvp_count))
    }

    /// List events with optional type filter and pagination.
    ///
    /// # Expected Behavior
    ///
    /// Returns a paginated list of events ordered by `start_time` descending.
    /// Optional filter by event type.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.events` (database read).
    pub async fn list_events(
        pool: &PgPool,
        type_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<EventListResponse, CoreError> {
        let (events, total) = if let Some(t) = type_filter {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM core.events WHERE type = $1")
                .bind(t)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

            let events = sqlx::query_as::<_, Event>(
                "SELECT id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at FROM core.events WHERE type = $1 ORDER BY start_time DESC NULLS LAST LIMIT $2 OFFSET $3",
            )
            .bind(t)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            (events, total)
        } else {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM core.events")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

            let events = sqlx::query_as::<_, Event>(
                "SELECT id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at FROM core.events ORDER BY start_time DESC NULLS LAST LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            (events, total)
        };

        let mut responses = Vec::new();
        for event in events {
            let rsvp_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM core.event_rsvps WHERE event_id = $1")
                    .bind(event.id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);

            responses.push(Self::event_to_response(event, rsvp_count));
        }

        Ok(EventListResponse {
            events: responses,
            total,
        })
    }

    /// Update an event.
    ///
    /// # Expected Behavior
    ///
    /// Updates only the provided fields. Returns the updated event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the event does not exist.
    ///
    /// # Side Effects
    ///
    /// - Updates row in `core.events` (database write).
    pub async fn update_event(
        pool: &PgPool,
        id: Uuid,
        data: &EventUpdate,
    ) -> Result<EventResponse, CoreError> {
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
        if data.start_time.is_some() {
            updates.push(format!("start_time = ${param_idx}"));
            param_idx += 1;
        }
        if data.end_time.is_some() {
            updates.push(format!("end_time = ${param_idx}"));
            param_idx += 1;
        }
        if data.location.is_some() {
            updates.push(format!("location = ${param_idx}"));
            param_idx += 1;
        }
        if data.r#type.is_some() {
            updates.push(format!("type = ${param_idx}"));
            param_idx += 1;
        }
        if data.capacity.is_some() {
            updates.push(format!("capacity = ${param_idx}"));
            param_idx += 1;
        }
        if data.speaker_name.is_some() {
            updates.push(format!("speaker_name = ${param_idx}"));
            param_idx += 1;
        }
        if data.speaker_bio.is_some() {
            updates.push(format!("speaker_bio = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Self::get_event(pool, id).await;
        }

        let sql = format!(
            "UPDATE core.events SET {} WHERE id = ${} RETURNING id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Event>(&sql);

        if let Some(ref v) = data.title {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.start_time {
            query = query.bind(v);
        }
        if let Some(ref v) = data.end_time {
            query = query.bind(v);
        }
        if let Some(ref v) = data.location {
            query = query.bind(v);
        }
        if let Some(ref v) = data.r#type {
            query = query.bind(v);
        }
        if let Some(ref v) = data.capacity {
            query = query.bind(v);
        }
        if let Some(ref v) = data.speaker_name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.speaker_bio {
            query = query.bind(v);
        }

        let event = query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| CoreError::NotFound("Event".into(), id.to_string()))?;

        log::info!("Updated event {id}");
        let rsvp_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.event_rsvps WHERE event_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        Ok(Self::event_to_response(event, rsvp_count))
    }

    /// Delete an event.
    ///
    /// # Expected Behavior
    ///
    /// Deletes the event and its RSVPs (cascade).
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the event does not exist.
    ///
    /// # Side Effects
    ///
    /// - Deletes from `core.events` (database write, cascades to rsvps).
    pub async fn delete_event(pool: &PgPool, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM core.events WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("Event".into(), id.to_string()));
        }

        log::info!("Deleted event {id}");
        Ok(())
    }

    /// RSVP to an event.
    ///
    /// # Expected Behavior
    ///
    /// Creates an RSVP for the user. Checks capacity. If Redis is available,
    /// publishes an `event.rsvp` event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the event does not exist.
    /// Returns `CoreError::Conflict` if the user already RSVP'd.
    /// Returns `CoreError::Validation` if the event is at capacity.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.events` and `core.event_rsvps` (database reads).
    /// - Inserts into `core.event_rsvps` (database write).
    /// - Publishes `event.rsvp` event to Redis, if available.
    pub async fn rsvp_event(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        event_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        let event = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at FROM core.events WHERE id = $1",
        )
        .bind(event_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Event".into(), event_id.to_string()))?;

        if let Some(cap) = event.capacity {
            let rsvp_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM core.event_rsvps WHERE event_id = $1")
                    .bind(event_id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);

            if rsvp_count >= i64::from(cap) {
                return Err(CoreError::Validation("Event is at capacity".into()));
            }
        }

        let existing: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.event_rsvps WHERE event_id = $1 AND user_id = $2)",
        )
        .bind(event_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if existing {
            return Err(CoreError::Conflict("Already RSVP'd to this event".into()));
        }

        sqlx::query(
            "INSERT INTO core.event_rsvps (event_id, user_id, attended, checked_in) VALUES ($1, $2, FALSE, FALSE)",
        )
        .bind(event_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        publisher::event_rsvp(redis_conn, &event_id.to_string(), &user_id.to_string()).await;
        log::info!("User {user_id} RSVP'd to event {event_id}");
        Ok(())
    }

    /// Cancel an RSVP.
    ///
    /// # Expected Behavior
    ///
    /// Removes the user's RSVP from the event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no RSVP exists.
    ///
    /// # Side Effects
    ///
    /// - Deletes from `core.event_rsvps` (database write).
    pub async fn cancel_rsvp(
        pool: &PgPool,
        event_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        let result =
            sqlx::query("DELETE FROM core.event_rsvps WHERE event_id = $1 AND user_id = $2")
                .bind(event_id)
                .bind(user_id)
                .execute(pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(
                "RSVP".into(),
                format!("event {event_id} user {user_id}"),
            ));
        }

        log::info!("User {user_id} cancelled RSVP to event {event_id}");
        Ok(())
    }

    /// Get event attendees with optional attended-only filter.
    ///
    /// # Expected Behavior
    ///
    /// Returns a paginated list of attendees for the event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the event does not exist.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.event_rsvps` (database read).
    pub async fn get_attendees(
        pool: &PgPool,
        event_id: Uuid,
        attended_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AttendeeResponse>, CoreError> {
        let _event = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at FROM core.events WHERE id = $1",
        )
        .bind(event_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Event".into(), event_id.to_string()))?;

        let rsvps = if attended_only {
            sqlx::query_as::<_, EventRsvp>(
                "SELECT event_id, user_id, rsvped_at, attended, checked_in FROM core.event_rsvps WHERE event_id = $1 AND (attended = TRUE OR checked_in = TRUE) ORDER BY rsvped_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(event_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, EventRsvp>(
                "SELECT event_id, user_id, rsvped_at, attended, checked_in FROM core.event_rsvps WHERE event_id = $1 ORDER BY rsvped_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(event_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(rsvps
            .into_iter()
            .map(|r| AttendeeResponse {
                user_id: r.user_id,
                attended: r.attended.unwrap_or(false),
                checked_in: r.checked_in.unwrap_or(false),
                rsvped_at: r.rsvped_at,
            })
            .collect())
    }

    /// Mark an attendee as having attended.
    ///
    /// # Expected Behavior
    ///
    /// Sets `attended = TRUE` for the given event/user RSVP.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no RSVP exists.
    ///
    /// # Side Effects
    ///
    /// - Updates `core.event_rsvps` (database write).
    pub async fn mark_attended(
        pool: &PgPool,
        event_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        let result = sqlx::query(
            "UPDATE core.event_rsvps SET attended = TRUE WHERE event_id = $1 AND user_id = $2",
        )
        .bind(event_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(
                "RSVP".into(),
                format!("event {event_id} user {user_id}"),
            ));
        }

        log::info!("Marked user {user_id} attended event {event_id}");
        Ok(())
    }

    /// List all events (admin endpoint, no type filter).
    ///
    /// # Expected Behavior
    ///
    /// Returns all events ordered by `created_at` descending with pagination.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.events` (database read).
    pub async fn list_events_admin(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EventResponse>, CoreError> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT id, title, description, start_time, end_time, location, type, capacity, speaker_name, speaker_bio, created_at FROM core.events ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let mut responses = Vec::new();
        for event in events {
            let rsvp_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM core.event_rsvps WHERE event_id = $1")
                    .bind(event.id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);

            responses.push(Self::event_to_response(event, rsvp_count));
        }

        Ok(responses)
    }

    fn event_to_response(event: Event, rsvp_count: i64) -> EventResponse {
        EventResponse {
            id: event.id,
            title: event.title,
            description: event.description,
            start_time: event.start_time,
            end_time: event.end_time,
            location: event.location,
            event_type: event.r#type,
            capacity: event.capacity,
            speaker_name: event.speaker_name,
            speaker_bio: event.speaker_bio,
            rsvp_count,
            created_at: event.created_at,
        }
    }
}

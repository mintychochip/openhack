use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of an event.
///
/// # Expected Behavior
///
/// Maps directly to the `core.events` table. `type` categorizes the event
/// (e.g., "workshop", "talk", "social"). `capacity` limits RSVPs (null
/// means unlimited).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub r#type: Option<String>,
    pub capacity: Option<i32>,
    pub speaker_name: Option<String>,
    pub speaker_bio: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Database row representation of an event RSVP.
///
/// # Expected Behavior
///
/// Maps directly to the `core.event_rsvps` table. Composite primary key
/// is (`event_id`, `user_id`). `attended` and `checked_in` track physical
/// attendance.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventRsvp {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub rsvped_at: Option<DateTime<Utc>>,
    pub attended: Option<bool>,
    pub checked_in: Option<bool>,
}

/// Request body for creating a new event.
///
/// # Expected Behavior
///
/// `title` is required. All other fields are optional.
#[derive(Debug, Clone, Deserialize)]
pub struct EventCreate {
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub r#type: Option<String>,
    pub capacity: Option<i32>,
    pub speaker_name: Option<String>,
    pub speaker_bio: Option<String>,
}

/// Request body for updating an event.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated.
#[derive(Debug, Clone, Deserialize)]
pub struct EventUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub r#type: Option<String>,
    pub capacity: Option<i32>,
    pub speaker_name: Option<String>,
    pub speaker_bio: Option<String>,
}

/// Response body for a single event with RSVP count.
///
/// # Expected Behavior
///
/// Contains event fields plus `rsvp_count` derived from counting
/// `event_rsvps` rows.
#[derive(Debug, Clone, Serialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub capacity: Option<i32>,
    pub speaker_name: Option<String>,
    pub speaker_bio: Option<String>,
    pub rsvp_count: i64,
    pub created_at: Option<DateTime<Utc>>,
}

/// Response body for a paginated list of events.
///
/// # Expected Behavior
///
/// Contains the list of event responses and a total count for pagination.
#[derive(Debug, Clone, Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventResponse>,
    pub total: i64,
}

/// Query parameters for listing events.
///
/// # Expected Behavior
///
/// All fields are optional. `type` filters by event type. `limit` defaults
/// to 20, `offset` defaults to 0.
#[derive(Debug, Clone, Deserialize)]
pub struct EventListQuery {
    pub r#type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for listing event attendees.
///
/// # Expected Behavior
///
/// `attended_only` filters to only checked-in attendees. `limit` and `offset`
/// control pagination.
#[derive(Debug, Clone, Deserialize)]
pub struct AttendeeListQuery {
    pub attended_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Response body for an event attendee.
///
/// # Expected Behavior
///
/// Contains the `user_id`, attended status, and RSVP timestamp.
#[derive(Debug, Clone, Serialize)]
pub struct AttendeeResponse {
    pub user_id: Uuid,
    pub attended: bool,
    pub checked_in: bool,
    pub rsvped_at: Option<DateTime<Utc>>,
}

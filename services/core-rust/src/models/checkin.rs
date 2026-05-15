use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of an event check-in.
///
/// # Expected Behavior
///
/// Maps directly to the `core.event_checkins` table. `qr_code` is a unique
/// string generated per event for scanning. `user_id` is populated when
/// the QR code is scanned.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventCheckin {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Option<Uuid>,
    pub qr_code: String,
    pub checked_in: Option<bool>,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Request body for scanning a check-in QR code.
///
/// # Expected Behavior
///
/// `qr_code` is the string scanned from the QR code.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckinScanRequest {
    pub qr_code: String,
}

/// Response body for a QR code.
///
/// # Expected Behavior
///
/// Contains the check-in ID and the QR code string for display.
#[derive(Debug, Clone, Serialize)]
pub struct QrCodeResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub qr_code: String,
    pub checked_in: bool,
}

/// Response body for check-in statistics.
///
/// # Expected Behavior
///
/// Contains counts for total RSVPs, checked-in attendees, and total
/// check-in codes for an event.
#[derive(Debug, Clone, Serialize)]
pub struct CheckinStatsResponse {
    pub event_id: Uuid,
    pub total_rsvps: i64,
    pub checked_in: i64,
    pub total_qr_codes: i64,
}

/// Response body for a check-in attendee.
///
/// # Expected Behavior
///
/// Contains user ID and check-in timestamp.
#[derive(Debug, Clone, Serialize)]
pub struct CheckinAttendeeResponse {
    pub user_id: Option<Uuid>,
    pub qr_code: String,
    pub checked_in: bool,
    pub checked_in_at: Option<DateTime<Utc>>,
}

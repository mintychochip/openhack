use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::CoreError;
use crate::models::checkin::{
    CheckinAttendeeResponse, CheckinStatsResponse, EventCheckin, QrCodeResponse,
};

pub struct CheckinService;

impl CheckinService {
    /// Generate a random 16-character QR code string.
    fn generate_qr_code() -> String {
        let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
        let mut rng = rand::thread_rng();
        (0..16)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect()
    }

    /// Create a QR code for an event.
    ///
    /// # Expected Behavior
    ///
    /// Generates a unique QR code string and inserts it into
    /// `core.event_checkins` for the given event. Returns the QR code.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on insert failure.
    ///
    /// # Side Effects
    ///
    /// - Inserts a row into `core.event_checkins` (database write).
    pub async fn create_qr_code(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<QrCodeResponse, CoreError> {
        let qr_code = Self::generate_qr_code();

        let checkin = sqlx::query_as::<_, EventCheckin>(
            "INSERT INTO core.event_checkins (event_id, qr_code, checked_in) VALUES ($1, $2, FALSE) RETURNING id, event_id, user_id, qr_code, checked_in, checked_in_at, created_at",
        )
        .bind(event_id)
        .bind(&qr_code)
        .fetch_one(pool)
        .await?;

        log::info!("Created QR code for event {event_id}");
        Ok(QrCodeResponse {
            id: checkin.id,
            event_id: checkin.event_id,
            qr_code: checkin.qr_code,
            checked_in: checkin.checked_in.unwrap_or(false),
        })
    }

    /// Get QR codes for an event.
    ///
    /// # Expected Behavior
    ///
    /// Returns all QR codes for the given event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.event_checkins` (database read).
    pub async fn get_qr_codes(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<Vec<QrCodeResponse>, CoreError> {
        let checkins = sqlx::query_as::<_, EventCheckin>(
            "SELECT id, event_id, user_id, qr_code, checked_in, checked_in_at, created_at FROM core.event_checkins WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_all(pool)
        .await?;

        Ok(checkins
            .into_iter()
            .map(|c| QrCodeResponse {
                id: c.id,
                event_id: c.event_id,
                qr_code: c.qr_code,
                checked_in: c.checked_in.unwrap_or(false),
            })
            .collect())
    }

    /// Scan a QR code and check in a user.
    ///
    /// # Expected Behavior
    ///
    /// Finds the checkin record by QR code. If not already checked in,
    /// marks it as checked in with the user's ID and timestamp. Also
    /// updates the `event_rsvps` `checked_in` field.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the QR code does not exist.
    /// Returns `CoreError::Conflict` if already checked in.
    ///
    /// # Side Effects
    ///
    /// - Updates `core.event_checkins` (database write).
    /// - Updates `core.event_rsvps` (database write).
    pub async fn scan_checkin(
        pool: &PgPool,
        qr_code: &str,
        user_id: Uuid,
    ) -> Result<QrCodeResponse, CoreError> {
        let checkin = sqlx::query_as::<_, EventCheckin>(
            "SELECT id, event_id, user_id, qr_code, checked_in, checked_in_at, created_at FROM core.event_checkins WHERE qr_code = $1",
        )
        .bind(qr_code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("QRCode".into(), qr_code.to_string()))?;

        if checkin.checked_in.unwrap_or(false) {
            return Err(CoreError::Conflict("QR code already used".into()));
        }

        let updated = sqlx::query_as::<_, EventCheckin>(
            "UPDATE core.event_checkins SET checked_in = TRUE, checked_in_at = NOW(), user_id = $1 WHERE qr_code = $2 RETURNING id, event_id, user_id, qr_code, checked_in, checked_in_at, created_at",
        )
        .bind(user_id)
        .bind(qr_code)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            "UPDATE core.event_rsvps SET checked_in = TRUE WHERE event_id = $1 AND user_id = $2",
        )
        .bind(updated.event_id)
        .bind(user_id)
        .execute(pool)
        .await
        .ok();

        log::info!(
            "User {} checked in via QR code for event {}",
            user_id,
            updated.event_id
        );

        Ok(QrCodeResponse {
            id: updated.id,
            event_id: updated.event_id,
            qr_code: updated.qr_code,
            checked_in: true,
        })
    }

    /// Get check-in statistics for an event.
    ///
    /// # Expected Behavior
    ///
    /// Returns counts of total RSVPs, checked-in attendees, and total
    /// QR codes for the event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.event_rsvps` and `core.event_checkins` (database reads).
    pub async fn get_checkin_stats(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<CheckinStatsResponse, CoreError> {
        let total_rsvps: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.event_rsvps WHERE event_id = $1")
                .bind(event_id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let checked_in: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM core.event_rsvps WHERE event_id = $1 AND checked_in = TRUE",
        )
        .bind(event_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let total_qr_codes: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.event_checkins WHERE event_id = $1")
                .bind(event_id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        Ok(CheckinStatsResponse {
            event_id,
            total_rsvps,
            checked_in,
            total_qr_codes,
        })
    }

    /// Get check-in attendees for an event.
    ///
    /// # Expected Behavior
    ///
    /// Returns all checkin records for the event with user and status info.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.event_checkins` (database read).
    pub async fn get_checkin_attendees(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<Vec<CheckinAttendeeResponse>, CoreError> {
        let checkins = sqlx::query_as::<_, EventCheckin>(
            "SELECT id, event_id, user_id, qr_code, checked_in, checked_in_at, created_at FROM core.event_checkins WHERE event_id = $1 ORDER BY checked_in_at DESC NULLS LAST",
        )
        .bind(event_id)
        .fetch_all(pool)
        .await?;

        Ok(checkins
            .into_iter()
            .map(|c| CheckinAttendeeResponse {
                user_id: c.user_id,
                qr_code: c.qr_code,
                checked_in: c.checked_in.unwrap_or(false),
                checked_in_at: c.checked_in_at,
            })
            .collect())
    }
}

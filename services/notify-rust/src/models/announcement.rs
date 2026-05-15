use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Announcement {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub audience: String,
    pub channels: Vec<String>,
    pub status: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub sent_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub content: String,
    pub audience: String,
    pub channels: Vec<String>,
    pub status: Option<String>,
    pub scheduled_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAnnouncementsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
}

impl ListAnnouncementsQuery {
    /// Return the effective limit, clamped between 1 and 200, defaulting to 50.
    ///
    /// # Expected Behavior
    ///
    /// If `limit` is None, returns 50. If less than 1, returns 1.
    /// If greater than 200, returns 200. Otherwise returns the provided value.
    ///
    /// # Errors
    ///
    /// None. All inputs are valid.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 200)
    }

    /// Return the effective offset, defaulting to 0 if None.
    ///
    /// # Expected Behavior
    ///
    /// If `offset` is None, returns 0. If negative, returns 0.
    /// Otherwise returns the provided value.
    ///
    /// # Errors
    ///
    /// None.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn effective_offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

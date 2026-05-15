use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a project submission for a prize.
///
/// # Expected Behavior
///
/// Maps directly to the `sponsor.submissions` table. Each row represents
/// a unique (`prize_id`, `project_id`) pair enforced by a UNIQUE constraint.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub prize_id: Uuid,
    pub project_id: Uuid,
    pub status: Option<String>,
    pub submitted_at: Option<NaiveDateTime>,
}

/// Request body for submitting a project for a prize.
///
/// # Expected Behavior
///
/// `project_id` is required. The `prize_id` is determined from the URL
/// path parameter rather than this body. Duplicate submissions for the
/// same (`prize_id`, `project_id`) pair will fail with a unique constraint
/// violation from the database.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmissionCreate {
    pub project_id: Uuid,
}

/// Response body for a single submission.
///
/// # Expected Behavior
///
/// Contains all submission fields including the generated UUID and
/// submission timestamp.
#[derive(Debug, Clone, Serialize)]
pub struct SubmissionResponse {
    pub id: Uuid,
    pub prize_id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub submitted_at: Option<NaiveDateTime>,
}

impl From<Submission> for SubmissionResponse {
    fn from(s: Submission) -> Self {
        Self {
            id: s.id,
            prize_id: s.prize_id,
            project_id: s.project_id,
            status: s.status.unwrap_or_else(|| "pending".to_string()),
            submitted_at: s.submitted_at,
        }
    }
}

/// Response body for a list of submissions.
///
/// # Expected Behavior
///
/// Contains the list of submission responses and a total count for pagination.
#[derive(Debug, Clone, Serialize)]
pub struct SubmissionListResponse {
    pub submissions: Vec<SubmissionResponse>,
    pub total: i64,
}

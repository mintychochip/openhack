use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::SponsorError;
use crate::models::booth::Booth;
use crate::models::prize::Prize;
use crate::models::submission::{
    Submission, SubmissionCreate, SubmissionListResponse, SubmissionResponse,
};

pub struct SubmissionService;

impl SubmissionService {
    /// Submit a project for a prize.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the prize exists, then inserts a new row into
    /// `sponsor.submissions` with the given `prize_id` and `project_id`.
    /// If a submission for the same (`prize_id`, `project_id`) pair already
    /// exists, the database UNIQUE constraint will cause a database error
    /// which is propagated as `SponsorError::Database`.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no prize exists with the given
    /// `prize_id`.
    /// Returns `SponsorError::Database` if the insert fails (e.g., unique
    /// constraint violation on (`prize_id`, `project_id`)).
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.prizes` to verify existence (database read).
    /// - Inserts a row into `sponsor.submissions` (database write).
    pub async fn submit_project(
        pool: &PgPool,
        prize_id: Uuid,
        data: &SubmissionCreate,
    ) -> Result<SubmissionResponse, SponsorError> {
        let prize_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sponsor.prizes WHERE id = $1)")
                .bind(prize_id)
                .fetch_one(pool)
                .await
                .unwrap_or(false);

        if !prize_exists {
            return Err(SponsorError::NotFound("Prize".into(), prize_id.to_string()));
        }

        let row = sqlx::query_as::<_, Submission>(
            "INSERT INTO sponsor.submissions (prize_id, project_id)
             VALUES ($1, $2)
             RETURNING *",
        )
        .bind(prize_id)
        .bind(data.project_id)
        .fetch_one(pool)
        .await?;

        log::info!(
            "Submitted project {} for prize {}",
            data.project_id,
            prize_id
        );

        Ok(row.into())
    }

    /// List all submissions for a prize.
    ///
    /// # Expected Behavior
    ///
    /// Queries `sponsor.submissions` for all submissions belonging to the
    /// given `prize_id`, ordered by `submitted_at` descending. Returns
    /// total count for pagination.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.submissions` (database read).
    pub async fn list_submissions(
        pool: &PgPool,
        prize_id: Uuid,
    ) -> Result<SubmissionListResponse, SponsorError> {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sponsor.submissions WHERE prize_id = $1")
                .bind(prize_id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let submissions = sqlx::query_as::<_, Submission>(
            "SELECT * FROM sponsor.submissions WHERE prize_id = $1 ORDER BY submitted_at DESC",
        )
        .bind(prize_id)
        .fetch_all(pool)
        .await?;

        Ok(SubmissionListResponse {
            submissions: submissions.into_iter().map(Into::into).collect(),
            total,
        })
    }

    /// List all submissions for prizes belonging to booths owned by the given sponsor.
    ///
    /// # Expected Behavior
    ///
    /// Joins `sponsor.submissions` with `sponsor.prizes` and `sponsor.booths`
    /// to find all submissions where the booth's `sponsor_id` matches the
    /// provided value. Returns a `SubmissionListResponse` ordered by
    /// `submitted_at` descending with total count for pagination.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.submissions`, `sponsor.prizes`, and
    ///   `sponsor.booths` (database reads).
    pub async fn list_sponsor_submissions(
        pool: &PgPool,
        sponsor_id: &str,
    ) -> Result<SubmissionListResponse, SponsorError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sponsor.submissions s JOIN sponsor.prizes p ON s.prize_id = p.id JOIN sponsor.booths b ON p.booth_id = b.id WHERE b.sponsor_id = $1",
        )
        .bind(sponsor_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let submissions = sqlx::query_as::<_, Submission>(
            "SELECT s.* FROM sponsor.submissions s JOIN sponsor.prizes p ON s.prize_id = p.id JOIN sponsor.booths b ON p.booth_id = b.id WHERE b.sponsor_id = $1 ORDER BY s.submitted_at DESC",
        )
        .bind(sponsor_id)
        .fetch_all(pool)
        .await?;

        Ok(SubmissionListResponse {
            submissions: submissions.into_iter().map(Into::into).collect(),
            total,
        })
    }

    /// Approve a submission with ownership verification.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the submission's prize's booth is owned by the given
    /// `sponsor_id`. If the ownership check passes, updates the submission's
    /// status to 'approved'. Returns the updated submission as a
    /// `SubmissionResponse`.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no submission exists with the given
    /// ID.
    /// Returns `SponsorError::Forbidden` if the submission's prize's booth is
    /// not owned by the sponsor.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.submissions`, `sponsor.prizes`, and
    ///   `sponsor.booths` to verify ownership (database reads).
    /// - Updates `status` column in `sponsor.submissions` to 'approved'
    ///   (database write).
    pub async fn approve_submission(
        pool: &PgPool,
        submission_id: Uuid,
        sponsor_id: &str,
    ) -> Result<SubmissionResponse, SponsorError> {
        let submission =
            sqlx::query_as::<_, Submission>("SELECT * FROM sponsor.submissions WHERE id = $1")
                .bind(submission_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| {
                    SponsorError::NotFound("Submission".into(), submission_id.to_string())
                })?;

        verify_submission_ownership(pool, &submission, sponsor_id).await?;

        let row = sqlx::query_as::<_, Submission>(
            "UPDATE sponsor.submissions SET status = 'approved' WHERE id = $1 RETURNING *",
        )
        .bind(submission_id)
        .fetch_one(pool)
        .await?;

        log::info!("Approved submission {submission_id}");

        Ok(row.into())
    }

    /// Reject a submission with ownership verification.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the submission's prize's booth is owned by the given
    /// `sponsor_id`. If the ownership check passes, updates the submission's
    /// status to 'rejected'. Returns the updated submission as a
    /// `SubmissionResponse`.
    ///
    /// # Errors
    ///
    /// Returns `SponsorError::NotFound` if no submission exists with the given
    /// ID.
    /// Returns `SponsorError::Forbidden` if the submission's prize's booth is
    /// not owned by the sponsor.
    /// Returns `SponsorError::Database` on database query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `sponsor.submissions`, `sponsor.prizes`, and
    ///   `sponsor.booths` to verify ownership (database reads).
    /// - Updates `status` column in `sponsor.submissions` to 'rejected'
    ///   (database write).
    pub async fn reject_submission(
        pool: &PgPool,
        submission_id: Uuid,
        sponsor_id: &str,
    ) -> Result<SubmissionResponse, SponsorError> {
        let submission =
            sqlx::query_as::<_, Submission>("SELECT * FROM sponsor.submissions WHERE id = $1")
                .bind(submission_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| {
                    SponsorError::NotFound("Submission".into(), submission_id.to_string())
                })?;

        verify_submission_ownership(pool, &submission, sponsor_id).await?;

        let row = sqlx::query_as::<_, Submission>(
            "UPDATE sponsor.submissions SET status = 'rejected' WHERE id = $1 RETURNING *",
        )
        .bind(submission_id)
        .fetch_one(pool)
        .await?;

        log::info!("Rejected submission {submission_id}");

        Ok(row.into())
    }
}

/// Verify that a submission belongs to a booth owned by the given sponsor.
///
/// # Expected Behavior
///
/// Fetches the prize and booth for the submission. Returns `Ok(())` if the
/// booth's `sponsor_id` matches the provided value. Returns
/// `SponsorError::Forbidden` if the sponsor does not own the booth.
///
/// # Errors
///
/// Returns `SponsorError::NotFound` if the prize or booth does not exist.
/// Returns `SponsorError::Forbidden` if the booth's `sponsor_id` does not
/// match the provided `sponsor_id`.
/// Returns `SponsorError::Database` on database query failure.
///
/// # Side Effects
///
/// - Reads from `sponsor.prizes` and `sponsor.booths` (database reads).
async fn verify_submission_ownership(
    pool: &PgPool,
    submission: &Submission,
    sponsor_id: &str,
) -> Result<(), SponsorError> {
    let prize = sqlx::query_as::<_, Prize>("SELECT * FROM sponsor.prizes WHERE id = $1")
        .bind(submission.prize_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Prize".into(), submission.prize_id.to_string()))?;

    let booth = sqlx::query_as::<_, Booth>("SELECT * FROM sponsor.booths WHERE id = $1")
        .bind(prize.booth_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Booth".into(), prize.booth_id.to_string()))?;

    match &booth.sponsor_id {
        Some(sid) if sid == sponsor_id => Ok(()),
        _ => Err(SponsorError::Forbidden(
            "You do not own this submission".into(),
        )),
    }
}

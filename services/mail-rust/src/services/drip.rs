use crate::errors::MailError;
use crate::models::drip::{
    DripCampaign, DripCampaignCreate, DripCampaignListResponse, DripCampaignQuery,
    DripCampaignResponse, DripCampaignUpdate, DripEnrollment, DripEnrollRequest,
    DripEnrollmentListResponse, DripEnrollmentResponse, DripLog, DripStep, DripStepCreate,
    DripStepListResponse, DripStepResponse, DripStepUpdate, ProcessPendingResponse,
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct DripService;

impl DripService {
    pub async fn create_campaign(
        pool: &PgPool,
        data: &DripCampaignCreate,
    ) -> Result<DripCampaignResponse, MailError> {
        let campaign = sqlx::query_as::<_, DripCampaign>(
            "INSERT INTO mail.drip_campaigns (name, description, trigger_event, created_by)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(&data.name)
        .bind(&data.description)
        .bind(&data.trigger_event)
        .bind(data.created_by)
        .fetch_one(pool)
        .await?;

        Ok(campaign.into())
    }

    pub async fn get_campaign(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<DripCampaignResponse, MailError> {
        let campaign = sqlx::query_as::<_, DripCampaign>(
            "SELECT * FROM mail.drip_campaigns WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| MailError::NotFound("Campaign".into(), id.to_string()))?;

        Ok(campaign.into())
    }

    pub async fn list_campaigns(
        pool: &PgPool,
        query: &DripCampaignQuery,
    ) -> Result<DripCampaignListResponse, MailError> {
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let mut where_clauses = Vec::new();
        let mut param_idx = 1;

        if let Some(ref trigger) = query.trigger_event {
            where_clauses.push(format!("trigger_event = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref status) = query.status {
            where_clauses.push(format!("status = ${param_idx}"));
            param_idx += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let total: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM mail.drip_campaigns {}",
            where_clause
        ))
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let sql = format!(
            "SELECT * FROM mail.drip_campaigns {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_idx,
            param_idx + 1
        );
        let mut db_query = sqlx::query_as::<_, DripCampaign>(&sql);

        if let Some(ref trigger) = query.trigger_event {
            db_query = db_query.bind(trigger);
        }
        if let Some(ref status) = query.status {
            db_query = db_query.bind(status);
        }

        let campaigns = db_query
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok(DripCampaignListResponse {
            campaigns: campaigns.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn update_campaign(
        pool: &PgPool,
        id: Uuid,
        data: &DripCampaignUpdate,
    ) -> Result<DripCampaignResponse, MailError> {
        let mut updates = Vec::new();
        let mut param_idx = 1;

        if let Some(ref name) = data.name {
            updates.push(format!("name = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref description) = data.description {
            updates.push(format!("description = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref status) = data.status {
            updates.push(format!("status = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Self::get_campaign(pool, id).await;
        }

        let sql = format!(
            "UPDATE mail.drip_campaigns SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut db_query = sqlx::query_as::<_, DripCampaign>(&sql);

        if let Some(ref name) = data.name {
            db_query = db_query.bind(name);
        }
        if let Some(ref description) = data.description {
            db_query = db_query.bind(description);
        }
        if let Some(ref status) = data.status {
            db_query = db_query.bind(status);
        }

        let campaign = db_query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| MailError::NotFound("Campaign".into(), id.to_string()))?;

        Ok(campaign.into())
    }

    pub async fn delete_campaign(pool: &PgPool, id: Uuid) -> Result<bool, MailError> {
        let result = sqlx::query("DELETE FROM mail.drip_campaigns WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_step(
        pool: &PgPool,
        campaign_id: Uuid,
        data: &DripStepCreate,
    ) -> Result<DripStepResponse, MailError> {
        let step = sqlx::query_as::<_, DripStep>(
            "INSERT INTO mail.drip_steps (campaign_id, step_order, delay_hours, template_id, subject, body_html, body_text)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        )
        .bind(campaign_id)
        .bind(data.step_order)
        .bind(data.delay_hours)
        .bind(data.template_id)
        .bind(&data.subject)
        .bind(&data.body_html)
        .bind(&data.body_text)
        .fetch_one(pool)
        .await?;

        Ok(step.into())
    }

    pub async fn list_steps(
        pool: &PgPool,
        campaign_id: Uuid,
    ) -> Result<DripStepListResponse, MailError> {
        let steps = sqlx::query_as::<_, DripStep>(
            "SELECT * FROM mail.drip_steps WHERE campaign_id = $1 ORDER BY step_order",
        )
        .bind(campaign_id)
        .fetch_all(pool)
        .await?;

        let total = steps.len() as i64;

        Ok(DripStepListResponse {
            steps: steps.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn update_step(
        pool: &PgPool,
        id: Uuid,
        data: &DripStepUpdate,
    ) -> Result<DripStepResponse, MailError> {
        let mut updates = Vec::new();
        let mut param_idx = 1;

        if let Some(step_order) = data.step_order {
            updates.push(format!("step_order = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(delay_hours) = data.delay_hours {
            updates.push(format!("delay_hours = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref subject) = data.subject {
            updates.push(format!("subject = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref body_html) = data.body_html {
            updates.push(format!("body_html = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref body_text) = data.body_text {
            updates.push(format!("body_text = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            let step = sqlx::query_as::<_, DripStep>(
                "SELECT * FROM mail.drip_steps WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| MailError::NotFound("Step".into(), id.to_string()))?;

            return Ok(step.into());
        }

        let sql = format!(
            "UPDATE mail.drip_steps SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut db_query = sqlx::query_as::<_, DripStep>(&sql);

        if let Some(step_order) = data.step_order {
            db_query = db_query.bind(step_order);
        }
        if let Some(delay_hours) = data.delay_hours {
            db_query = db_query.bind(delay_hours);
        }
        if let Some(ref subject) = data.subject {
            db_query = db_query.bind(subject);
        }
        if let Some(ref body_html) = data.body_html {
            db_query = db_query.bind(body_html);
        }
        if let Some(ref body_text) = data.body_text {
            db_query = db_query.bind(body_text);
        }

        let step = db_query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| MailError::NotFound("Step".into(), id.to_string()))?;

        Ok(step.into())
    }

    pub async fn delete_step(pool: &PgPool, id: Uuid) -> Result<bool, MailError> {
        let result = sqlx::query("DELETE FROM mail.drip_steps WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn enroll(
        pool: &PgPool,
        campaign_id: Uuid,
        request: &DripEnrollRequest,
    ) -> Result<DripEnrollmentResponse, MailError> {
        let campaign = sqlx::query_as::<_, DripCampaign>(
            "SELECT * FROM mail.drip_campaigns WHERE id = $1 AND status = 'active'",
        )
        .bind(campaign_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| MailError::NotFound("Campaign".into(), campaign_id.to_string()))?;

        let trigger_data_json = request.trigger_data.as_ref().map(|d| d.to_string());

        let first_step_at = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MIN(delay_hours) FROM mail.drip_steps WHERE campaign_id = $1",
        )
        .bind(campaign_id)
        .fetch_one(pool)
        .await?;

        let next_step_at = first_step_at.map(|hours| Utc::now() + Duration::hours(hours as i64));

        let enrollment = sqlx::query_as::<_, DripEnrollment>(
            "INSERT INTO mail.drip_enrollments (campaign_id, user_id, trigger_event, trigger_data, next_step_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (campaign_id, user_id) DO UPDATE SET status = 'active', next_step_at = EXCLUDED.next_step_at
             RETURNING *",
        )
        .bind(campaign_id)
        .bind(request.user_id)
        .bind(&campaign.trigger_event)
        .bind(trigger_data_json)
        .bind(next_step_at)
        .fetch_one(pool)
        .await?;

        Ok(enrollment.into())
    }

    pub async fn list_enrollments(
        pool: &PgPool,
        campaign_id: Uuid,
    ) -> Result<DripEnrollmentListResponse, MailError> {
        let enrollments = sqlx::query_as::<_, DripEnrollment>(
            "SELECT * FROM mail.drip_enrollments WHERE campaign_id = $1 ORDER BY enrolled_at DESC",
        )
        .bind(campaign_id)
        .fetch_all(pool)
        .await?;

        let total = enrollments.len() as i64;

        Ok(DripEnrollmentListResponse {
            enrollments: enrollments.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn unsubscribe(
        pool: &PgPool,
        enrollment_id: Uuid,
    ) -> Result<DripEnrollmentResponse, MailError> {
        let enrollment = sqlx::query_as::<_, DripEnrollment>(
            "UPDATE mail.drip_enrollments SET status = 'unsubscribed'
             WHERE id = $1 RETURNING *",
        )
        .bind(enrollment_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| MailError::NotFound("Enrollment".into(), enrollment_id.to_string()))?;

        Ok(enrollment.into())
    }

    pub async fn process_pending(
        pool: &PgPool,
    ) -> Result<ProcessPendingResponse, MailError> {
        let now = Utc::now();

        let pending_enrollments = sqlx::query_as::<_, DripEnrollment>(
            "SELECT * FROM mail.drip_enrollments 
             WHERE status = 'active' 
             AND next_step_at <= $1 
             AND current_step >= 0",
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        let mut processed_count = 0i32;
        let mut sent_count = 0i32;
        let mut failed_count = 0i32;
        let mut skipped_count = 0i32;

        for enrollment in pending_enrollments {
            processed_count += 1;

            let step = sqlx::query_as::<_, DripStep>(
                "SELECT * FROM mail.drip_steps 
                 WHERE campaign_id = $1 AND step_order = $2",
            )
            .bind(enrollment.campaign_id)
            .bind(enrollment.current_step + 1)
            .fetch_optional(pool)
            .await?;

            match step {
                Some(s) => {
                    let log = sqlx::query_as::<_, DripLog>(
                        "INSERT INTO mail.drip_logs (enrollment_id, step_id, status, created_at)
                         VALUES ($1, $2, 'pending', NOW()) RETURNING *",
                    )
                    .bind(enrollment.id)
                    .bind(s.id)
                    .fetch_one(pool)
                    .await?;

                    match Self::send_drip_email(pool, &enrollment, &s).await {
                        Ok(_) => {
                            sqlx::query(
                                "UPDATE mail.drip_logs SET status = 'sent', sent_at = NOW() WHERE id = $1",
                            )
                            .bind(log.id)
                            .execute(pool)
                            .await?;

                            sent_count += 1;

                            let next_step = sqlx::query_as::<_, DripStep>(
                                "SELECT * FROM mail.drip_steps 
                                 WHERE campaign_id = $1 AND step_order = $2",
                            )
                            .bind(enrollment.campaign_id)
                            .bind(enrollment.current_step + 2)
                            .fetch_optional(pool)
                            .await?;

                            if let Some(next) = next_step {
                                let next_at = now + Duration::hours(next.delay_hours as i64);
                                sqlx::query(
                                    "UPDATE mail.drip_enrollments 
                                     SET current_step = current_step + 1, next_step_at = $1 
                                     WHERE id = $2",
                                )
                                .bind(next_at)
                                .bind(enrollment.id)
                                .execute(pool)
                                .await?;
                            } else {
                                sqlx::query(
                                    "UPDATE mail.drip_enrollments 
                                     SET current_step = current_step + 1, status = 'completed', completed_at = NOW() 
                                     WHERE id = $1",
                                )
                                .bind(enrollment.id)
                                .execute(pool)
                                .await?;
                            }
                        }
                        Err(e) => {
                            sqlx::query(
                                "UPDATE mail.drip_logs SET status = 'failed', error_message = $1 WHERE id = $2",
                            )
                            .bind(e.to_string())
                            .bind(log.id)
                            .execute(pool)
                            .await?;

                            failed_count += 1;
                        }
                    }
                }
                None => {
                    sqlx::query(
                        "UPDATE mail.drip_enrollments SET status = 'completed', completed_at = NOW() WHERE id = $1",
                    )
                    .bind(enrollment.id)
                    .execute(pool)
                    .await?;

                    skipped_count += 1;
                }
            }
        }

        Ok(ProcessPendingResponse {
            processed_count,
            sent_count,
            failed_count,
            skipped_count,
        })
    }

    async fn send_drip_email(
        pool: &PgPool,
        enrollment: &DripEnrollment,
        step: &DripStep,
    ) -> Result<(), MailError> {
        let user_email: Option<String> = sqlx::query_scalar(
            "SELECT email FROM auth.users WHERE id = $1",
        )
        .bind(enrollment.user_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        let email = user_email.ok_or_else(|| {
            MailError::NotFound("User email".into(), enrollment.user_id.to_string())
        })?;

        let body = step.body_html.clone().or(step.body_text.clone()).unwrap_or_default();

        sqlx::query(
            "INSERT INTO mail.sent_emails (recipient, subject, body_html, body_text, status)
             VALUES ($1, $2, $3, $4, 'pending') RETURNING id",
        )
        .bind(&email)
        .bind(&step.subject)
        .bind(&step.body_html)
        .bind(&step.body_text)
        .fetch_one(pool)
        .await?;

        Ok(())
    }
}

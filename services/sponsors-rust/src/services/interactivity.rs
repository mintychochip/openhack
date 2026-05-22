use crate::errors::SponsorError;
use crate::models::interactivity::{
    BoothAnalyticsResponse, BoothMessage, BoothMessageCreate, BoothMessageResponse, BoothPoll,
    BoothPollCreate, BoothPollListResponse, BoothPollOption, BoothPollResponse,
    BoothPollResponseData, BoothResource, BoothResourceCreate, BoothResourceResponse,
    BoothVisitor, BoothVisitorCreate, BoothVisitorResponse,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct InteractivityService;

impl InteractivityService {
    pub async fn record_visit(
        pool: &PgPool,
        booth_id: Uuid,
        data: &BoothVisitorCreate,
    ) -> Result<BoothVisitorResponse, SponsorError> {
        let visitor = sqlx::query_as::<_, BoothVisitor>(
            "INSERT INTO sponsor.booth_visitors (booth_id, user_id, source, metadata)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(booth_id)
        .bind(data.user_id)
        .bind(&data.source)
        .bind(&data.metadata)
        .fetch_one(pool)
        .await?;

        Ok(visitor.into())
    }

    pub async fn update_visit_duration(
        pool: &PgPool,
        visit_id: Uuid,
        duration_seconds: i32,
    ) -> Result<BoothVisitorResponse, SponsorError> {
        let visitor = sqlx::query_as::<_, BoothVisitor>(
            "UPDATE sponsor.booth_visitors SET duration_seconds = $1 WHERE id = $2 RETURNING *",
        )
        .bind(duration_seconds)
        .bind(visit_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Visit".into(), visit_id.to_string()))?;

        Ok(visitor.into())
    }

    pub async fn get_visitors(
        pool: &PgPool,
        booth_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BoothVisitorResponse>, SponsorError> {
        let visitors = sqlx::query_as::<_, BoothVisitor>(
            "SELECT * FROM sponsor.booth_visitors WHERE booth_id = $1 ORDER BY visited_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(booth_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(visitors.into_iter().map(Into::into).collect())
    }

    pub async fn create_message(
        pool: &PgPool,
        booth_id: Uuid,
        data: &BoothMessageCreate,
    ) -> Result<BoothMessageResponse, SponsorError> {
        let message = sqlx::query_as::<_, BoothMessage>(
            "INSERT INTO sponsor.booth_messages (booth_id, user_id, message, parent_message_id)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(booth_id)
        .bind(data.user_id)
        .bind(&data.message)
        .bind(data.parent_message_id)
        .fetch_one(pool)
        .await?;

        Ok(message.into())
    }

    pub async fn get_messages(
        pool: &PgPool,
        booth_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BoothMessageResponse>, SponsorError> {
        let messages = sqlx::query_as::<_, BoothMessage>(
            "SELECT * FROM sponsor.booth_messages 
             WHERE booth_id = $1 
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(booth_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(messages.into_iter().map(Into::into).collect())
    }

    pub async fn mark_message_read(
        pool: &PgPool,
        message_id: Uuid,
    ) -> Result<BoothMessageResponse, SponsorError> {
        let message = sqlx::query_as::<_, BoothMessage>(
            "UPDATE sponsor.booth_messages SET is_read = TRUE WHERE id = $1 RETURNING *",
        )
        .bind(message_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Message".into(), message_id.to_string()))?;

        Ok(message.into())
    }

    pub async fn create_poll(
        pool: &PgPool,
        booth_id: Uuid,
        data: &BoothPollCreate,
        created_by: Option<Uuid>,
    ) -> Result<BoothPoll, SponsorError> {
        let options_json = serde_json::to_value(&data.options)
            .map_err(|e| SponsorError::Validation(format!("Invalid poll options: {}", e)))?;

        let poll = sqlx::query_as::<_, BoothPoll>(
            "INSERT INTO sponsor.booth_polls (booth_id, question, description, options, allow_multiple, ends_at, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        )
        .bind(booth_id)
        .bind(&data.question)
        .bind(&data.description)
        .bind(options_json)
        .bind(data.allow_multiple.unwrap_or(false))
        .bind(data.ends_at)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(poll)
    }

    pub async fn get_polls(
        pool: &PgPool,
        booth_id: Uuid,
    ) -> Result<BoothPollListResponse, SponsorError> {
        let polls = sqlx::query_as::<_, BoothPoll>(
            "SELECT * FROM sponsor.booth_polls WHERE booth_id = $1 ORDER BY created_at DESC",
        )
        .bind(booth_id)
        .fetch_all(pool)
        .await?;

        let mut poll_responses = Vec::new();

        for poll in &polls {
            let total_responses: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sponsor.booth_poll_responses WHERE poll_id = $1",
            )
            .bind(poll.id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            let options_len = poll
                .options
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0);

            let mut option_counts = Vec::with_capacity(options_len);
            for i in 0..options_len {
                let count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM sponsor.booth_poll_responses 
                     WHERE poll_id = $1 AND $2 = ANY(option_indices)",
                )
                .bind(poll.id)
                .bind(i as i32)
                .fetch_one(pool)
                .await
                .unwrap_or(0);
                option_counts.push(count);
            }

            poll_responses.push(BoothPollResponseData {
                id: poll.id,
                booth_id: poll.booth_id,
                question: poll.question.clone(),
                description: poll.description.clone(),
                options: poll.options.clone(),
                allow_multiple: poll.allow_multiple,
                is_active: poll.is_active,
                ends_at: poll.ends_at,
                total_responses,
                option_counts,
            });
        }

        Ok(BoothPollListResponse {
            polls: poll_responses,
            total: polls.len() as i64,
        })
    }

    pub async fn vote_poll(
        pool: &PgPool,
        poll_id: Uuid,
        user_id: Uuid,
        option_indices: Vec<i32>,
    ) -> Result<BoothPollResponse, SponsorError> {
        let poll = sqlx::query_as::<_, BoothPoll>(
            "SELECT * FROM sponsor.booth_polls WHERE id = $1",
        )
        .bind(poll_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Poll".into(), poll_id.to_string()))?;

        if !poll.is_active {
            return Err(SponsorError::Validation("Poll is not active".into()));
        }

        if let Some(ends_at) = poll.ends_at {
            if Utc::now() > ends_at {
                return Err(SponsorError::Validation("Poll has ended".into()));
            }
        }

        let options_len = poll
            .options
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);

        for &idx in &option_indices {
            if idx < 0 || idx as usize >= options_len {
                return Err(SponsorError::Validation(format!(
                    "Invalid option index: {}",
                    idx
                )));
            }
        }

        if !poll.allow_multiple && option_indices.len() > 1 {
            return Err(SponsorError::Validation(
                "This poll only allows one option".into(),
            ));
        }

        let response = sqlx::query_as::<_, BoothPollResponse>(
            "INSERT INTO sponsor.booth_poll_responses (poll_id, user_id, option_indices)
             VALUES ($1, $2, $3)
             ON CONFLICT (poll_id, user_id) DO UPDATE SET option_indices = EXCLUDED.option_indices, responded_at = NOW()
             RETURNING *",
        )
        .bind(poll_id)
        .bind(user_id)
        .bind(&option_indices)
        .fetch_one(pool)
        .await?;

        Ok(response)
    }

    pub async fn create_resource(
        pool: &PgPool,
        booth_id: Uuid,
        data: &BoothResourceCreate,
    ) -> Result<BoothResourceResponse, SponsorError> {
        let resource = sqlx::query_as::<_, BoothResource>(
            "INSERT INTO sponsor.booth_resources (booth_id, title, description, resource_type, url)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(booth_id)
        .bind(&data.title)
        .bind(&data.description)
        .bind(&data.resource_type)
        .bind(&data.url)
        .fetch_one(pool)
        .await?;

        Ok(resource.into())
    }

    pub async fn get_resources(
        pool: &PgPool,
        booth_id: Uuid,
    ) -> Result<Vec<BoothResourceResponse>, SponsorError> {
        let resources = sqlx::query_as::<_, BoothResource>(
            "SELECT * FROM sponsor.booth_resources WHERE booth_id = $1 ORDER BY created_at DESC",
        )
        .bind(booth_id)
        .fetch_all(pool)
        .await?;

        Ok(resources.into_iter().map(Into::into).collect())
    }

    pub async fn increment_download(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<BoothResourceResponse, SponsorError> {
        let resource = sqlx::query_as::<_, BoothResource>(
            "UPDATE sponsor.booth_resources SET download_count = download_count + 1 WHERE id = $1 RETURNING *",
        )
        .bind(resource_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| SponsorError::NotFound("Resource".into(), resource_id.to_string()))?;

        Ok(resource.into())
    }

    pub async fn get_analytics(
        pool: &PgPool,
        booth_id: Uuid,
    ) -> Result<BoothAnalyticsResponse, SponsorError> {
        let total_visitors: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sponsor.booth_visitors WHERE booth_id = $1",
        )
        .bind(booth_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let unique_visitors: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT user_id) FROM sponsor.booth_visitors WHERE booth_id = $1 AND user_id IS NOT NULL",
        )
        .bind(booth_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let avg_duration: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(duration_seconds) FROM sponsor.booth_visitors WHERE booth_id = $1 AND duration_seconds IS NOT NULL",
        )
        .bind(booth_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let total_messages: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sponsor.booth_messages WHERE booth_id = $1",
        )
        .bind(booth_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let unread_messages: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sponsor.booth_messages WHERE booth_id = $1 AND is_read = FALSE AND is_sponsor_reply = FALSE",
        )
        .bind(booth_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let total_poll_responses: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sponsor.booth_poll_responses pr
             JOIN sponsor.booth_polls p ON pr.poll_id = p.id
             WHERE p.booth_id = $1",
        )
        .bind(booth_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let total_resource_downloads: i64 = sqlx::query_scalar(
            "SELECT SUM(download_count) FROM sponsor.booth_resources WHERE booth_id = $1",
        )
        .bind(booth_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);

        let visitors_by_source: serde_json::Value = sqlx::query_scalar::<_, serde_json::Value>(
            "SELECT COALESCE(
                json_object_agg(source, count),
                '{}'::json
            ) FROM (
                SELECT source, COUNT(*) as count 
                FROM sponsor.booth_visitors 
                WHERE booth_id = $1 AND source IS NOT NULL
                GROUP BY source
            ) t",
        )
        .bind(booth_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(serde_json::json!({}));

        Ok(BoothAnalyticsResponse {
            total_visitors,
            unique_visitors,
            avg_duration_seconds: avg_duration,
            total_messages,
            unread_messages,
            total_poll_responses,
            total_resource_downloads,
            visitors_by_source,
        })
    }
}

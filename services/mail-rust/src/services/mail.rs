use crate::errors::MailError;
use crate::models::broadcast::{
    BroadcastCreate, BroadcastListResponse, BroadcastQuery, BroadcastResponse, BroadcastSchedule,
    MailBroadcast,
};
use crate::models::event::MailEvent;
use crate::models::message::{MailMessage, MessageListResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub struct MailService;

impl MailService {
    pub async fn send_email(
        pool: &PgPool,
        smtp: &crate::services::smtp::SmtpProvider,
        to: &[String],
        cc: &[String],
        bcc: &[String],
        subject: &str,
        body_html: &str,
        body_text: Option<&str>,
        template_id: Option<Uuid>,
        priority: &str,
        metadata: &serde_json::Value,
    ) -> Result<(Uuid, String), MailError> {
        let id = Uuid::new_v4();
        let message_id_str = format!("<{id}@mail.openhack.local>");

        sqlx::query(
            "INSERT INTO mail.messages
             (id, message_id, to_addrs, cc_addrs, bcc_addrs, subject, body_html, body_text,
              status, priority, template_id, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'queued', $9, $10, $11)",
        )
        .bind(id)
        .bind(&message_id_str)
        .bind(to)
        .bind(cc)
        .bind(bcc)
        .bind(subject)
        .bind(body_html)
        .bind(body_text)
        .bind(priority)
        .bind(template_id)
        .bind(metadata)
        .execute(pool)
        .await?;

        Self::create_event(pool, id, "queued", &serde_json::json!({})).await?;

        if let Err(e) = smtp.send_email(to, subject, body_html) {
            sqlx::query(
                "UPDATE mail.messages SET status = 'failed', bounce_reason = $1 WHERE id = $2",
            )
            .bind(e)
            .bind(id)
            .execute(pool)
            .await?;
            Self::create_event(pool, id, "failed", &serde_json::json!({})).await?;
        } else {
            sqlx::query("UPDATE mail.messages SET status = 'sent', sent_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            Self::create_event(pool, id, "sent", &serde_json::json!({})).await?;
        }

        Ok((id, message_id_str))
    }

    pub async fn send_bulk(
        pool: &PgPool,
        smtp: &crate::services::smtp::SmtpProvider,
        emails: &[serde_json::Value],
        batch_priority: &str,
    ) -> Result<(String, Vec<String>, usize), MailError> {
        let batch_id = Uuid::new_v4().to_string();
        let mut message_ids = Vec::new();

        for email in emails {
            let to: Vec<String> = email
                .get("to")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let subject = email.get("subject").and_then(|v| v.as_str()).unwrap_or("");
            let body_html = email
                .get("body_html")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if to.is_empty() {
                continue;
            }

            match Self::send_email(
                pool,
                smtp,
                &to,
                &[],
                &[],
                subject,
                body_html,
                None,
                None,
                batch_priority,
                &serde_json::json!({}),
            )
            .await
            {
                Ok((_, mid)) => message_ids.push(mid),
                Err(e) => log::warn!("Bulk send failed for recipient: {e:?}"),
            }
        }

        let total = message_ids.len();
        Ok((batch_id, message_ids, total))
    }

    pub async fn get_message(pool: &PgPool, message_id: &str) -> Result<MailMessage, MailError> {
        let row =
            sqlx::query_as::<_, MailMessage>("SELECT * FROM mail.messages WHERE message_id = $1")
                .bind(message_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| MailError::MessageNotFound(message_id.to_string()))?;

        Ok(row)
    }

    pub async fn list_messages(
        pool: &PgPool,
        page: i64,
        page_size: i64,
        status_filter: Option<&str>,
    ) -> Result<MessageListResponse, MailError> {
        let offset = (page - 1) * page_size;

        let (messages, total) = if let Some(status) = status_filter {
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM mail.messages WHERE status = $1")
                    .bind(status)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            let rows = sqlx::query_as::<_, MailMessage>(
                "SELECT * FROM mail.messages WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(status)
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        } else {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mail.messages")
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            let rows = sqlx::query_as::<_, MailMessage>(
                "SELECT * FROM mail.messages ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        };

        Ok(MessageListResponse {
            messages: messages.into_iter().map(Into::into).collect(),
            total,
            page,
            page_size,
        })
    }

    pub async fn get_message_events(
        pool: &PgPool,
        message_id: Uuid,
    ) -> Result<Vec<MailEvent>, MailError> {
        let events = sqlx::query_as::<_, MailEvent>(
            "SELECT * FROM mail.events WHERE message_id = $1 ORDER BY occurred_at DESC",
        )
        .bind(message_id)
        .fetch_all(pool)
        .await?;
        Ok(events)
    }

    pub async fn update_message_status(
        pool: &PgPool,
        message_id: &str,
        status: &str,
        event_data: &serde_json::Value,
    ) -> Result<(), MailError> {
        let msg = Self::get_message(pool, message_id).await?;

        match status {
            "delivered" => {
                sqlx::query("UPDATE mail.messages SET status = 'delivered', delivered_at = NOW() WHERE id = $1")
                    .bind(msg.id)
                    .execute(pool)
                    .await?;
            }
            "bounced" => {
                let reason = event_data
                    .get("bounce_reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                sqlx::query(
                    "UPDATE mail.messages SET status = 'bounced', bounce_reason = $1 WHERE id = $2",
                )
                .bind(reason)
                .bind(msg.id)
                .execute(pool)
                .await?;
            }
            _ => {
                sqlx::query("UPDATE mail.messages SET status = $1 WHERE id = $2")
                    .bind(status)
                    .bind(msg.id)
                    .execute(pool)
                    .await?;
            }
        }

        Self::create_event(pool, msg.id, status, event_data).await?;
        Ok(())
    }

    async fn create_event(
        pool: &PgPool,
        message_id: Uuid,
        event_type: &str,
        event_data: &serde_json::Value,
    ) -> Result<(), MailError> {
        sqlx::query(
            "INSERT INTO mail.events (id, message_id, event_type, event_data)
             VALUES (gen_random_uuid(), $1, $2, $3)",
        )
        .bind(message_id)
        .bind(event_type)
        .bind(event_data)
        .execute(pool)
        .await?;
        Ok(())
    }
}

pub struct BroadcastService;

impl BroadcastService {
    pub async fn create(
        pool: &PgPool,
        data: &BroadcastCreate,
    ) -> Result<BroadcastResponse, MailError> {
        let row = sqlx::query_as::<_, MailBroadcast>(
            "INSERT INTO mail.broadcasts (name, template_id, subject, body_html, body_text, audience_filter, status)
             VALUES ($1, $2, $3, $4, $5, $6, 'draft') RETURNING *",
        )
        .bind(&data.name)
        .bind(data.template_id)
        .bind(&data.subject)
        .bind(&data.body_html)
        .bind(&data.body_text)
        .bind(&data.audience_filter)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list(
        pool: &PgPool,
        query: &BroadcastQuery,
    ) -> Result<BroadcastListResponse, MailError> {
        let (rows, total) = if let Some(ref status) = query.status_filter {
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM mail.broadcasts WHERE status = $1")
                    .bind(status)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            let rows = sqlx::query_as::<_, MailBroadcast>(
                "SELECT * FROM mail.broadcasts WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(status)
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        } else {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mail.broadcasts")
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            let rows = sqlx::query_as::<_, MailBroadcast>(
                "SELECT * FROM mail.broadcasts ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        };

        Ok(BroadcastListResponse {
            broadcasts: rows.into_iter().map(Into::into).collect(),
            total,
            page: (query.offset / query.limit.max(1)) + 1,
            page_size: query.limit,
        })
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<BroadcastResponse, MailError> {
        let row = sqlx::query_as::<_, MailBroadcast>("SELECT * FROM mail.broadcasts WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| MailError::BroadcastNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn schedule(
        pool: &PgPool,
        id: Uuid,
        data: &BroadcastSchedule,
    ) -> Result<BroadcastResponse, MailError> {
        let row = sqlx::query_as::<_, MailBroadcast>(
            "UPDATE mail.broadcasts SET status = 'scheduled', scheduled_at = $1 WHERE id = $2 RETURNING *",
        )
        .bind(&data.scheduled_at)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| MailError::BroadcastNotFound(id.to_string()))?;

        Ok(row.into())
    }
}

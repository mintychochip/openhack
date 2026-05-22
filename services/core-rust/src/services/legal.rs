use crate::errors::CoreError;
use crate::models::legal::{
    HackathonRule, HackathonRuleCreate, HackathonRuleListResponse, HackathonRuleQuery,
    HackathonRuleResponse, HackathonRuleUpdate, PrizeFulfillment, PrizeFulfillmentCreate,
    PrizeFulfillmentListResponse, PrizeFulfillmentQuery, PrizeFulfillmentResponse,
    PrizeFulfillmentUpdate, ShippingAddress,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct LegalService;

impl LegalService {
    pub async fn create_rule(
        pool: &PgPool,
        data: &HackathonRuleCreate,
    ) -> Result<HackathonRuleResponse, CoreError> {
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM core.hackathon_rules WHERE event_id = $1",
        )
        .bind(data.event_id)
        .fetch_one(pool)
        .await?;

        let version = max_version.unwrap_or(1);

        let rule = sqlx::query_as::<_, HackathonRule>(
            "INSERT INTO core.hackathon_rules (event_id, title, rules_text, version, created_by)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(data.event_id)
        .bind(&data.title)
        .bind(&data.rules_text)
        .bind(version)
        .bind(data.created_by)
        .fetch_one(pool)
        .await?;

        Ok(rule.into())
    }

    pub async fn get_rule(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<HackathonRuleResponse, CoreError> {
        let rule = sqlx::query_as::<_, HackathonRule>(
            "SELECT * FROM core.hackathon_rules WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Rule".into(), id.to_string()))?;

        Ok(rule.into())
    }

    pub async fn list_rules(
        pool: &PgPool,
        query: &HackathonRuleQuery,
    ) -> Result<HackathonRuleListResponse, CoreError> {
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let mut where_clauses = Vec::new();
        let mut param_idx = 1;

        if let Some(event_id) = query.event_id {
            where_clauses.push(format!("event_id = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(true) = query.published_only {
            where_clauses.push(format!("published = ${param_idx}"));
            param_idx += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_query = format!(
            "SELECT COUNT(*) FROM core.hackathon_rules {}",
            where_clause
        );

        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let data_query = format!(
            "SELECT * FROM core.hackathon_rules {} ORDER BY version DESC, created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_idx,
            param_idx + 1
        );

        let mut db_query = sqlx::query_as::<_, HackathonRule>(&data_query);

        if let Some(event_id) = query.event_id {
            db_query = db_query.bind(event_id);
        }
        if let Some(true) = query.published_only {
            db_query = db_query.bind(true);
        }

        let rules = db_query
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok(HackathonRuleListResponse {
            rules: rules.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn update_rule(
        pool: &PgPool,
        id: Uuid,
        data: &HackathonRuleUpdate,
    ) -> Result<HackathonRuleResponse, CoreError> {
        let mut updates = Vec::new();
        let mut param_idx = 1;

        if let Some(ref title) = data.title {
            updates.push(format!("title = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref rules_text) = data.rules_text {
            updates.push(format!("rules_text = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(published) = data.published {
            updates.push(format!("published = ${param_idx}"));
            if published {
                updates.push("published_at = COALESCE(published_at, NOW())".to_string());
            }
            param_idx += 1;
        }

        if updates.is_empty() {
            return Self::get_rule(pool, id).await;
        }

        let sql = format!(
            "UPDATE core.hackathon_rules SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut db_query = sqlx::query_as::<_, HackathonRule>(&sql);

        if let Some(ref title) = data.title {
            db_query = db_query.bind(title);
        }
        if let Some(ref rules_text) = data.rules_text {
            db_query = db_query.bind(rules_text);
        }
        if let Some(published) = data.published {
            db_query = db_query.bind(published);
        }

        let rule = db_query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| CoreError::NotFound("Rule".into(), id.to_string()))?;

        Ok(rule.into())
    }

    pub async fn publish_rule(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<HackathonRuleResponse, CoreError> {
        let rule = sqlx::query_as::<_, HackathonRule>(
            "UPDATE core.hackathon_rules SET published = true, published_at = NOW()
             WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Rule".into(), id.to_string()))?;

        Ok(rule.into())
    }

    pub async fn delete_rule(pool: &PgPool, id: Uuid) -> Result<bool, CoreError> {
        let result = sqlx::query("DELETE FROM core.hackathon_rules WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

pub struct FulfillmentService;

impl FulfillmentService {
    pub async fn create(
        pool: &PgPool,
        data: &PrizeFulfillmentCreate,
    ) -> Result<PrizeFulfillmentResponse, CoreError> {
        let fulfillment = sqlx::query_as::<_, PrizeFulfillment>(
            "INSERT INTO core.prize_fulfillment (prize_id, sponsor_id, winner_project_id, winner_user_id)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(data.prize_id)
        .bind(data.sponsor_id)
        .bind(data.winner_project_id)
        .bind(data.winner_user_id)
        .fetch_one(pool)
        .await?;

        Ok(fulfillment.into())
    }

    pub async fn get(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<PrizeFulfillmentResponse, CoreError> {
        let fulfillment = sqlx::query_as::<_, PrizeFulfillment>(
            "SELECT * FROM core.prize_fulfillment WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Fulfillment".into(), id.to_string()))?;

        Ok(fulfillment.into())
    }

    pub async fn list(
        pool: &PgPool,
        query: &PrizeFulfillmentQuery,
    ) -> Result<PrizeFulfillmentListResponse, CoreError> {
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let mut where_clauses = Vec::new();
        let mut param_idx = 1;

        if let Some(prize_id) = query.prize_id {
            where_clauses.push(format!("prize_id = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(project_id) = query.project_id {
            where_clauses.push(format!("winner_project_id = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(user_id) = query.user_id {
            where_clauses.push(format!("winner_user_id = ${param_idx}"));
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

        let count_query = format!(
            "SELECT COUNT(*) FROM core.prize_fulfillment {}",
            where_clause
        );

        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let data_query = format!(
            "SELECT * FROM core.prize_fulfillment {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_idx,
            param_idx + 1
        );

        let mut db_query = sqlx::query_as::<_, PrizeFulfillment>(&data_query);

        if let Some(prize_id) = query.prize_id {
            db_query = db_query.bind(prize_id);
        }
        if let Some(project_id) = query.project_id {
            db_query = db_query.bind(project_id);
        }
        if let Some(user_id) = query.user_id {
            db_query = db_query.bind(user_id);
        }
        if let Some(ref status) = query.status {
            db_query = db_query.bind(status);
        }

        let fulfillments = db_query
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok(PrizeFulfillmentListResponse {
            fulfillments: fulfillments.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        data: &PrizeFulfillmentUpdate,
    ) -> Result<PrizeFulfillmentResponse, CoreError> {
        let mut updates = Vec::new();
        let mut param_idx = 1;

        if let Some(ref status) = data.status {
            updates.push(format!("status = ${param_idx}"));
            param_idx += 1;

            if status == "claimed" {
                updates.push("claimed_at = COALESCE(claimed_at, NOW())".to_string());
            } else if status == "shipped" {
                updates.push("shipped_at = COALESCE(shipped_at, NOW())".to_string());
            } else if status == "fulfilled" {
                updates.push("delivered_at = COALESCE(delivered_at, NOW())".to_string());
            }
        }
        if let Some(ref addr) = data.shipping_address {
            let addr_json = serde_json::to_value(addr)
                .map_err(|e| CoreError::Validation(format!("Invalid address: {}", e)))?;
            updates.push(format!("shipping_address = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref tracking) = data.tracking_number {
            updates.push(format!("tracking_number = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref carrier) = data.carrier {
            updates.push(format!("carrier = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(delivery) = data.estimated_delivery {
            updates.push(format!("estimated_delivery = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref notes) = data.notes {
            updates.push(format!("notes = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Self::get(pool, id).await;
        }

        let sql = format!(
            "UPDATE core.prize_fulfillment SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut db_query = sqlx::query_as::<_, PrizeFulfillment>(&sql);

        if let Some(ref status) = data.status {
            db_query = db_query.bind(status);
        }
        if let Some(ref addr) = data.shipping_address {
            let addr_json = serde_json::to_value(addr).map_err(|e| {
                CoreError::Validation(format!("Invalid address: {}", e))
            })?;
            db_query = db_query.bind(addr_json);
        }
        if let Some(ref tracking) = data.tracking_number {
            db_query = db_query.bind(tracking);
        }
        if let Some(ref carrier) = data.carrier {
            db_query = db_query.bind(carrier);
        }
        if let Some(delivery) = data.estimated_delivery {
            db_query = db_query.bind(delivery);
        }
        if let Some(ref notes) = data.notes {
            db_query = db_query.bind(notes);
        }

        let fulfillment = db_query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| CoreError::NotFound("Fulfillment".into(), id.to_string()))?;

        Ok(fulfillment.into())
    }

    pub async fn claim(
        pool: &PgPool,
        id: Uuid,
        shipping_address: &ShippingAddress,
    ) -> Result<PrizeFulfillmentResponse, CoreError> {
        let addr_json = serde_json::to_value(shipping_address)
            .map_err(|e| CoreError::Validation(format!("Invalid address: {}", e)))?;

        let fulfillment = sqlx::query_as::<_, PrizeFulfillment>(
            "UPDATE core.prize_fulfillment 
             SET status = 'claimed', shipping_address = $1, claimed_at = NOW()
             WHERE id = $2 AND status = 'pending' RETURNING *",
        )
        .bind(addr_json)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Fulfillment".into(), id.to_string()))?;

        Ok(fulfillment.into())
    }

    pub async fn ship(
        pool: &PgPool,
        id: Uuid,
        tracking_number: &str,
        carrier: &str,
        estimated_delivery: Option<chrono::NaiveDate>,
    ) -> Result<PrizeFulfillmentResponse, CoreError> {
        let fulfillment = sqlx::query_as::<_, PrizeFulfillment>(
            "UPDATE core.prize_fulfillment 
             SET status = 'shipped', tracking_number = $1, carrier = $2, 
                 estimated_delivery = $3, shipped_at = NOW()
             WHERE id = $4 AND status = 'claimed' RETURNING *",
        )
        .bind(tracking_number)
        .bind(carrier)
        .bind(estimated_delivery)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Fulfillment".into(), id.to_string()))?;

        Ok(fulfillment.into())
    }

    pub async fn fulfill(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<PrizeFulfillmentResponse, CoreError> {
        let fulfillment = sqlx::query_as::<_, PrizeFulfillment>(
            "UPDATE core.prize_fulfillment 
             SET status = 'fulfilled', delivered_at = NOW()
             WHERE id = $1 AND status IN ('shipped', 'claimed') RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Fulfillment".into(), id.to_string()))?;

        Ok(fulfillment.into())
    }

    pub async fn decline(
        pool: &PgPool,
        id: Uuid,
        notes: &str,
    ) -> Result<PrizeFulfillmentResponse, CoreError> {
        let fulfillment = sqlx::query_as::<_, PrizeFulfillment>(
            "UPDATE core.prize_fulfillment 
             SET status = 'declined', notes = COALESCE(notes || E'\\n\\n' || $1, $1)
             WHERE id = $2 AND status = 'pending' RETURNING *",
        )
        .bind(notes)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Fulfillment".into(), id.to_string()))?;

        Ok(fulfillment.into())
    }
}

use crate::errors::JudgingError;
use crate::models::advancement::{Advancement, AdvancementListResponse};
use crate::models::phase::{
    Phase, PhaseCreate, PhaseLeaderboardEntry, PhaseLeaderboardResponse, PhaseListResponse,
    PhaseQuery, PhaseResponse, PhaseUpdate,
};
use rust_decimal::prelude::ToPrimitive;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PhaseService;

impl PhaseService {
    pub async fn create(pool: &PgPool, data: &PhaseCreate) -> Result<PhaseResponse, JudgingError> {
        let row = sqlx::query_as::<_, Phase>(
            "INSERT INTO judging.phases (name, description, type, rubric_id, opens_at, closes_at,
             advancement_count, parent_phase_id, status, scoring_method, normalization_config)
             VALUES ($1, $2, $3, $4, $5::timestamptz, $6::timestamptz, $7, $8, 'draft', $9, $10) RETURNING *",
        )
        .bind(&data.name)
        .bind(&data.description)
        .bind(&data.phase_type)
        .bind(data.rubric_id)
        .bind(&data.opens_at)
        .bind(&data.closes_at)
        .bind(data.advancement_count)
        .bind(data.parent_phase_id)
        .bind(&data.scoring_method)
        .bind(&data.normalization_config)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<PhaseResponse, JudgingError> {
        let row = sqlx::query_as::<_, Phase>("SELECT * FROM judging.phases WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::PhaseNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn list(
        pool: &PgPool,
        query: &PhaseQuery,
    ) -> Result<PhaseListResponse, JudgingError> {
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let (rows, total) = if let Some(ref status) = query.status_filter {
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM judging.phases WHERE status = $1")
                    .bind(status)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            let rows = sqlx::query_as::<_, Phase>(
                "SELECT * FROM judging.phases WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        } else {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM judging.phases")
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            let rows = sqlx::query_as::<_, Phase>(
                "SELECT * FROM judging.phases ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        };

        Ok(PhaseListResponse {
            phases: rows.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        data: &PhaseUpdate,
    ) -> Result<PhaseResponse, JudgingError> {
        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.name.is_some() {
            updates.push(format!("name = ${param_idx}"));
            param_idx += 1;
        }
        if data.description.is_some() {
            updates.push(format!("description = ${param_idx}"));
            param_idx += 1;
        }
        if data.status.is_some() {
            updates.push(format!("status = ${param_idx}"));
            param_idx += 1;
        }
        if data.scoring_method.is_some() {
            updates.push(format!("scoring_method = ${param_idx}"));
            param_idx += 1;
        }
        if data.rubric_id.is_some() {
            updates.push(format!("rubric_id = ${param_idx}"));
            param_idx += 1;
        }
        if data.advancement_count.is_some() {
            updates.push(format!("advancement_count = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Self::get(pool, id).await;
        }

        updates.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE judging.phases SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Phase>(&sql);

        if let Some(ref v) = data.name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.status {
            query = query.bind(v);
        }
        if let Some(ref v) = data.scoring_method {
            query = query.bind(v);
        }
        if let Some(ref v) = data.rubric_id {
            query = query.bind(v);
        }
        if let Some(v) = data.advancement_count {
            query = query.bind(v);
        }

        let row = query.bind(id).fetch_optional(pool).await?;
        let row = row.ok_or_else(|| JudgingError::PhaseNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn open_phase(pool: &PgPool, id: Uuid) -> Result<PhaseResponse, JudgingError> {
        let row = sqlx::query_as::<_, Phase>(
            "UPDATE judging.phases SET status = 'open', opens_at = COALESCE(opens_at, NOW()), updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| JudgingError::PhaseNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn close_phase(pool: &PgPool, id: Uuid) -> Result<PhaseResponse, JudgingError> {
        let row = sqlx::query_as::<_, Phase>(
            "UPDATE judging.phases SET status = 'closed', closes_at = COALESCE(closes_at, NOW()), updated_at = NOW() WHERE id = $1 AND status = 'open' RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| JudgingError::PhaseClosed(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn finalize_phase(pool: &PgPool, id: Uuid) -> Result<PhaseResponse, JudgingError> {
        let phase = sqlx::query_as::<_, Phase>("SELECT * FROM judging.phases WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::PhaseNotFound(id.to_string()))?;

        if phase.status != "closed" {
            return Err(JudgingError::PhaseClosed(id.to_string()));
        }

        let row = sqlx::query_as::<_, Phase>(
            "UPDATE judging.phases SET status = 'finalized', updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_leaderboard(
        pool: &PgPool,
        phase_id: Uuid,
    ) -> Result<PhaseLeaderboardResponse, JudgingError> {
        let phase = sqlx::query_as::<_, Phase>("SELECT * FROM judging.phases WHERE id = $1")
            .bind(phase_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::PhaseNotFound(phase_id.to_string()))?;

        let rows = sqlx::query(
            "SELECT a.project_id, SUM(s.total_score) as total_score, COUNT(s.id) as score_count,
                    ROW_NUMBER() OVER (ORDER BY SUM(s.total_score) DESC) as rank
             FROM judging.scores s
             JOIN judging.assignments a ON s.assignment_id = a.id
             WHERE a.phase_id = $1 AND s.total_score IS NOT NULL
             GROUP BY a.project_id
             ORDER BY total_score DESC",
        )
        .bind(phase_id)
        .fetch_all(pool)
        .await?;

        let entries: Vec<PhaseLeaderboardEntry> = rows
            .iter()
            .map(|row| PhaseLeaderboardEntry {
                rank: i32::try_from(row.get::<i64, _>("rank")).unwrap_or(0),
                team_id: Uuid::nil(),
                project_id: row.get::<Uuid, _>("project_id"),
                total_score: row
                    .get::<Option<rust_decimal::Decimal>, _>("total_score")
                    .map_or(0.0, |d: rust_decimal::Decimal| d.to_f64().unwrap_or(0.0)),
                score_count: i32::try_from(row.get::<i64, _>("score_count")).unwrap_or(0),
                normalized_score: None,
            })
            .collect();

        let total = i64::try_from(entries.len()).unwrap_or(i64::MAX);

        Ok(PhaseLeaderboardResponse {
            phase_id,
            phase_name: phase.name,
            scoring_method: phase.scoring_method,
            entries,
            total,
        })
    }

    pub async fn get_advancements(
        pool: &PgPool,
        phase_id: Uuid,
    ) -> Result<AdvancementListResponse, JudgingError> {
        let rows = sqlx::query_as::<_, Advancement>(
            "SELECT * FROM judging.phase_advancements WHERE to_phase_id = $1 OR from_phase_id = $1 ORDER BY advanced_at DESC",
        )
        .bind(phase_id)
        .fetch_all(pool)
        .await?;

        let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Ok(AdvancementListResponse {
            advancements: rows,
            total,
        })
    }
}

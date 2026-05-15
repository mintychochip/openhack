use crate::errors::JudgingError;
use crate::models::assignment::{
    Assignment, AssignmentBulkRequest, AssignmentCreate, AssignmentListResponse, AssignmentQuery,
    AssignmentResponse, AssignmentUpdate,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct AssignmentService;

impl AssignmentService {
    pub async fn create(
        pool: &PgPool,
        data: &AssignmentCreate,
    ) -> Result<AssignmentResponse, JudgingError> {
        let row = sqlx::query_as::<_, Assignment>(
            "INSERT INTO judging.assignments (judge_id, project_id, rubric_id, priority, status)
             VALUES ($1, $2, $3, $4, 'pending') RETURNING *",
        )
        .bind(data.judge_id)
        .bind(data.project_id)
        .bind(data.rubric_id)
        .bind(data.priority)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list(
        pool: &PgPool,
        query: &AssignmentQuery,
    ) -> Result<AssignmentListResponse, JudgingError> {
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let (rows, total) = if let (Some(judge_id), Some(ref status)) =
            (query.judge_id, &query.status_filter)
        {
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM judging.assignments WHERE judge_id = $1 AND status = $2",
            )
            .bind(judge_id)
            .bind(status)
            .fetch_one(pool)
            .await
            .unwrap_or(0);
            let rows = sqlx::query_as::<_, Assignment>(
                "SELECT * FROM judging.assignments WHERE judge_id = $1 AND status = $2 ORDER BY assigned_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(judge_id)
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        } else if let Some(judge_id) = query.judge_id {
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM judging.assignments WHERE judge_id = $1")
                    .bind(judge_id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            let rows = sqlx::query_as::<_, Assignment>(
                "SELECT * FROM judging.assignments WHERE judge_id = $1 ORDER BY assigned_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(judge_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        } else if let Some(ref status) = query.status_filter {
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM judging.assignments WHERE status = $1")
                    .bind(status)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            let rows = sqlx::query_as::<_, Assignment>(
                "SELECT * FROM judging.assignments WHERE status = $1 ORDER BY assigned_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        } else {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM judging.assignments")
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            let rows = sqlx::query_as::<_, Assignment>(
                "SELECT * FROM judging.assignments ORDER BY assigned_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;
            (rows, count)
        };

        Ok(AssignmentListResponse {
            assignments: rows.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        data: &AssignmentUpdate,
    ) -> Result<AssignmentResponse, JudgingError> {
        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.status.is_some() {
            updates.push(format!("status = ${param_idx}"));
            param_idx += 1;
        }
        if data.rubric_id.is_some() {
            updates.push(format!("rubric_id = ${param_idx}"));
            param_idx += 1;
        }
        if data.priority.is_some() {
            updates.push(format!("priority = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            let row =
                sqlx::query_as::<_, Assignment>("SELECT * FROM judging.assignments WHERE id = $1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
                    .ok_or_else(|| JudgingError::AssignmentNotFound(id.to_string()))?;
            return Ok(row.into());
        }

        let sql = format!(
            "UPDATE judging.assignments SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Assignment>(&sql);

        if let Some(ref v) = data.status {
            query = query.bind(v);
        }
        if let Some(ref v) = data.rubric_id {
            query = query.bind(v);
        }
        if let Some(ref v) = data.priority {
            query = query.bind(v);
        }

        let row = query.bind(id).fetch_optional(pool).await?;
        let row = row.ok_or_else(|| JudgingError::AssignmentNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, JudgingError> {
        let result = sqlx::query("DELETE FROM judging.assignments WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn bulk_assign(
        pool: &PgPool,
        data: &AssignmentBulkRequest,
    ) -> Result<Vec<AssignmentResponse>, JudgingError> {
        let mut results = Vec::new();

        match data.distribution.as_str() {
            "random" => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                for project_id in &data.project_ids {
                    let mut judges = data.judge_ids.clone();
                    judges.shuffle(&mut rng);
                    for judge_id in judges
                        .iter()
                        .take(usize::try_from(data.assignments_per_project).unwrap_or(0))
                    {
                        let row = sqlx::query_as::<_, Assignment>(
                            "INSERT INTO judging.assignments (judge_id, project_id, rubric_id, status)
                             VALUES ($1, $2, $3, 'pending') RETURNING *",
                        )
                        .bind(judge_id)
                        .bind(project_id)
                        .bind(data.rubric_id)
                        .fetch_optional(pool)
                        .await;
                        if let Ok(Some(r)) = row {
                            results.push(r.into());
                        }
                    }
                }
            }
            "round_robin" => {
                for (i, project_id) in data.project_ids.iter().enumerate() {
                    for j in 0..data.assignments_per_project {
                        let judge_idx =
                            (i + usize::try_from(j).unwrap_or(0)) % data.judge_ids.len();
                        let row = sqlx::query_as::<_, Assignment>(
                            "INSERT INTO judging.assignments (judge_id, project_id, rubric_id, status)
                             VALUES ($1, $2, $3, 'pending') RETURNING *",
                        )
                        .bind(data.judge_ids[judge_idx])
                        .bind(project_id)
                        .bind(data.rubric_id)
                        .fetch_optional(pool)
                        .await;
                        if let Ok(Some(r)) = row {
                            results.push(r.into());
                        }
                    }
                }
            }
            "manual" => {
                for ma in &data.assignments {
                    let row = sqlx::query_as::<_, Assignment>(
                        "INSERT INTO judging.assignments (judge_id, project_id, rubric_id, status)
                         VALUES ($1, $2, $3, 'pending') RETURNING *",
                    )
                    .bind(ma.judge_id)
                    .bind(ma.project_id)
                    .bind(data.rubric_id)
                    .fetch_optional(pool)
                    .await;
                    if let Ok(Some(r)) = row {
                        results.push(r.into());
                    }
                }
            }
            _ => {
                return Err(JudgingError::Validation(format!(
                    "Unknown distribution: {}",
                    data.distribution
                )));
            }
        }

        Ok(results)
    }
}

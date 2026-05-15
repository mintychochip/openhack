use crate::errors::JudgingError;
use crate::models::dashboard::DashboardResponse;
use crate::models::rubric::{
    CriterionInput, Rubric, RubricCreate, RubricListResponse, RubricResponse, RubricUpdate,
    RubricVersion, RubricVersionListResponse, RubricVersionResponse, ScoreValidationRequest,
    ScoreValidationResponse,
};
use crate::models::score::{Score, ScoreCreate, ScoreListResponse, ScoreResponse, ScoreUpdate};
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

pub struct JudgingService;

impl JudgingService {
    pub async fn create_rubric(
        pool: &PgPool,
        data: &RubricCreate,
    ) -> Result<RubricResponse, JudgingError> {
        let criteria_json = serde_json::to_value(&data.criteria)
            .map_err(|e| JudgingError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, Rubric>(
            "INSERT INTO judging.rubrics (name, description, criteria)
             VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&data.name)
        .bind(&data.description)
        .bind(&criteria_json)
        .fetch_one(pool)
        .await?;

        let rubric_id = row.id;

        let _ = sqlx::query(
            "INSERT INTO judging.rubric_versions (rubric_id, version, criteria)
             VALUES ($1, 1, $2)",
        )
        .bind(rubric_id)
        .bind(&criteria_json)
        .execute(pool)
        .await;

        Ok(row.into())
    }

    pub async fn list_rubrics(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<RubricListResponse, JudgingError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM judging.rubrics")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let rubrics = sqlx::query_as::<_, Rubric>(
            "SELECT * FROM judging.rubrics ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(RubricListResponse {
            rubrics: rubrics.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn get_rubric(pool: &PgPool, id: Uuid) -> Result<RubricResponse, JudgingError> {
        let row = sqlx::query_as::<_, Rubric>("SELECT * FROM judging.rubrics WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::RubricNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn update_rubric(
        pool: &PgPool,
        id: Uuid,
        data: &RubricUpdate,
    ) -> Result<RubricResponse, JudgingError> {
        let existing = sqlx::query_as::<_, Rubric>("SELECT * FROM judging.rubrics WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::RubricNotFound(id.to_string()))?;

        if let Some(ref criteria) = data.criteria {
            let _criteria_json = serde_json::to_value(criteria)
                .map_err(|e| JudgingError::Internal(e.to_string()))?;

            let _ = sqlx::query(
                "INSERT INTO judging.rubric_versions (rubric_id, version, criteria)
                 SELECT $1, COALESCE(MAX(version), 0) + 1, $2
                 FROM judging.rubric_versions WHERE rubric_id = $1",
            )
            .bind(id)
            .bind(&existing.criteria)
            .execute(pool)
            .await;
        }

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
        if data.criteria.is_some() {
            updates.push(format!("criteria = ${param_idx}"));
            param_idx += 1;
        }

        updates.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE judging.rubrics SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Rubric>(&sql);

        if let Some(ref v) = data.name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.criteria {
            let criteria_json =
                serde_json::to_value(v).map_err(|e| JudgingError::Internal(e.to_string()))?;
            query = query.bind(criteria_json);
        }

        let row = query.bind(id).fetch_optional(pool).await?;
        let row = row.ok_or_else(|| JudgingError::RubricNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn delete_rubric(pool: &PgPool, id: Uuid) -> Result<bool, JudgingError> {
        let result = sqlx::query("DELETE FROM judging.rubrics WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list_rubric_versions(
        pool: &PgPool,
        rubric_id: Uuid,
    ) -> Result<RubricVersionListResponse, JudgingError> {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM judging.rubric_versions WHERE rubric_id = $1")
                .bind(rubric_id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let versions = sqlx::query_as::<_, RubricVersion>(
            "SELECT * FROM judging.rubric_versions WHERE rubric_id = $1 ORDER BY version DESC",
        )
        .bind(rubric_id)
        .fetch_all(pool)
        .await?;

        Ok(RubricVersionListResponse {
            versions: versions.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn get_rubric_version(
        pool: &PgPool,
        rubric_id: Uuid,
        version: Option<i32>,
    ) -> Result<RubricVersionResponse, JudgingError> {
        let row = if let Some(v) = version {
            sqlx::query_as::<_, RubricVersion>(
                "SELECT * FROM judging.rubric_versions WHERE rubric_id = $1 AND version = $2",
            )
            .bind(rubric_id)
            .bind(v)
            .fetch_optional(pool)
            .await?
        } else {
            sqlx::query_as::<_, RubricVersion>(
                "SELECT * FROM judging.rubric_versions WHERE rubric_id = $1 ORDER BY version DESC LIMIT 1",
            )
            .bind(rubric_id)
            .fetch_optional(pool)
            .await?
        };

        let row = row.ok_or_else(|| JudgingError::RubricNotFound(rubric_id.to_string()))?;
        Ok(row.into())
    }

    pub async fn create_rubric_version(
        pool: &PgPool,
        rubric_id: Uuid,
    ) -> Result<RubricVersionResponse, JudgingError> {
        let existing = sqlx::query_as::<_, Rubric>("SELECT * FROM judging.rubrics WHERE id = $1")
            .bind(rubric_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::RubricNotFound(rubric_id.to_string()))?;

        let row = sqlx::query_as::<_, RubricVersion>(
            "INSERT INTO judging.rubric_versions (rubric_id, version, criteria)
             SELECT $1, COALESCE(MAX(version), 0) + 1, $2
             FROM judging.rubric_versions WHERE rubric_id = $1
             RETURNING *",
        )
        .bind(rubric_id)
        .bind(&existing.criteria)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn validate_score(
        pool: &PgPool,
        req: &ScoreValidationRequest,
    ) -> Result<ScoreValidationResponse, JudgingError> {
        let assignment = sqlx::query_as::<_, crate::models::assignment::Assignment>(
            "SELECT * FROM judging.assignments WHERE id = $1",
        )
        .bind(req.assignment_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| JudgingError::AssignmentNotFound(req.assignment_id.to_string()))?;

        let rubric_id = assignment
            .rubric_id
            .ok_or_else(|| JudgingError::ScoreValidation("No rubric assigned".into()))?;

        let rubric = sqlx::query_as::<_, Rubric>("SELECT * FROM judging.rubrics WHERE id = $1")
            .bind(rubric_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::RubricNotFound(rubric_id.to_string()))?;

        let criteria_list: Vec<CriterionInput> = serde_json::from_value(rubric.criteria)
            .map_err(|e| JudgingError::Internal(e.to_string()))?;

        let mut errors = Vec::new();
        let mut normalized_total = 0.0_f64;
        let mut total_weight = 0.0_f64;

        for criterion in &criteria_list {
            total_weight += criterion.weight;
            match req.scores.get(&criterion.name) {
                Some(score) => {
                    if *score > criterion.max_score {
                        errors.push(format!(
                            "Score for {} exceeds max_score {}: {}",
                            criterion.name, criterion.max_score, score
                        ));
                    }
                    if *score < 0.0 {
                        errors.push(format!("Score for {} cannot be negative", criterion.name));
                    }
                    normalized_total += (score / criterion.max_score) * criterion.weight * 10.0;
                }
                None if criterion.required => {
                    errors.push(format!("Missing required criterion: {}", criterion.name));
                }
                None => {
                    normalized_total += 0.0;
                }
            }
        }

        let valid = errors.is_empty();
        let normalized = if total_weight > 0.0 {
            Some(
                rust_decimal::Decimal::from_f64_retain(
                    normalized_total / total_weight * total_weight,
                )
                .unwrap_or(rust_decimal::Decimal::ZERO),
            )
        } else {
            None
        };

        Ok(ScoreValidationResponse {
            valid,
            errors,
            normalized_total: normalized,
        })
    }

    pub async fn submit_score(
        pool: &PgPool,
        data: &ScoreCreate,
    ) -> Result<ScoreResponse, JudgingError> {
        let assignment = sqlx::query_as::<_, crate::models::assignment::Assignment>(
            "SELECT * FROM judging.assignments WHERE id = $1",
        )
        .bind(data.assignment_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| JudgingError::AssignmentNotFound(data.assignment_id.to_string()))?;

        let scores_json = serde_json::to_value(&data.scores)
            .map_err(|e| JudgingError::Internal(e.to_string()))?;

        let total: f64 = data.scores.values().sum();
        let total_decimal =
            rust_decimal::Decimal::from_f64_retain(total).unwrap_or(rust_decimal::Decimal::ZERO);

        let row = sqlx::query_as::<_, Score>(
            "INSERT INTO judging.scores (assignment_id, scores, total_score, comment, feedback)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(data.assignment_id)
        .bind(&scores_json)
        .bind(total_decimal)
        .bind(&data.comment)
        .bind(&data.feedback)
        .fetch_one(pool)
        .await?;

        if assignment.started_at.is_none() {
            let _ = sqlx::query(
                "UPDATE judging.assignments SET started_at = NOW() WHERE id = $1 AND started_at IS NULL",
            )
            .bind(data.assignment_id)
            .execute(pool)
            .await;
        }

        let _ = sqlx::query(
            "UPDATE judging.assignments SET status = 'completed', completed_at = NOW() WHERE id = $1",
        )
        .bind(data.assignment_id)
        .execute(pool)
        .await;

        Ok(ScoreResponse {
            id: row.id,
            assignment_id: row.assignment_id,
            scores: row.scores,
            total_score: row.total.map(|d| d.to_string()),
            comment: row.comment,
            feedback: row.feedback,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_normalized: false,
            judge_stats: None,
        })
    }

    pub async fn get_scores_for_project(
        pool: &PgPool,
        project_id: Uuid,
    ) -> Result<ScoreListResponse, JudgingError> {
        let rows = sqlx::query_as::<_, Score>(
            "SELECT s.* FROM judging.scores s
             JOIN judging.assignments a ON s.assignment_id = a.id
             WHERE a.project_id = $1
             ORDER BY s.created_at DESC",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Ok(ScoreListResponse {
            scores: rows
                .into_iter()
                .map(|r| ScoreResponse {
                    id: r.id,
                    assignment_id: r.assignment_id,
                    scores: r.scores,
                    total_score: r.total.map(|d| d.to_string()),
                    comment: r.comment,
                    feedback: r.feedback,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    is_normalized: false,
                    judge_stats: None,
                })
                .collect(),
            total,
        })
    }

    pub async fn get_scores_by_judge(
        pool: &PgPool,
        judge_id: Uuid,
    ) -> Result<ScoreListResponse, JudgingError> {
        let rows = sqlx::query_as::<_, Score>(
            "SELECT s.* FROM judging.scores s
             JOIN judging.assignments a ON s.assignment_id = a.id
             WHERE a.judge_id = $1
             ORDER BY s.created_at DESC",
        )
        .bind(judge_id)
        .fetch_all(pool)
        .await?;

        let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Ok(ScoreListResponse {
            scores: rows
                .into_iter()
                .map(|r| ScoreResponse {
                    id: r.id,
                    assignment_id: r.assignment_id,
                    scores: r.scores,
                    total_score: r.total.map(|d| d.to_string()),
                    comment: r.comment,
                    feedback: r.feedback,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    is_normalized: false,
                    judge_stats: None,
                })
                .collect(),
            total,
        })
    }

    pub async fn update_score(
        pool: &PgPool,
        score_id: Uuid,
        data: &ScoreUpdate,
    ) -> Result<ScoreResponse, JudgingError> {
        let _existing = sqlx::query_as::<_, Score>("SELECT * FROM judging.scores WHERE id = $1")
            .bind(score_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| JudgingError::ScoreNotFound(score_id.to_string()))?;

        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.scores.is_some() {
            updates.push(format!("scores = ${param_idx}"));
            param_idx += 1;
        }
        if data.comment.is_some() {
            updates.push(format!("comment = ${param_idx}"));
            param_idx += 1;
        }
        if data.feedback.is_some() {
            updates.push(format!("feedback = ${param_idx}"));
            param_idx += 1;
        }

        if data.scores.is_some() {
            updates.push(format!("total_score = ${param_idx}"));
            param_idx += 1;
        }

        updates.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE judging.scores SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Score>(&sql);

        if let Some(ref scores) = data.scores {
            let scores_json =
                serde_json::to_value(scores).map_err(|e| JudgingError::Internal(e.to_string()))?;
            let total: f64 = scores.values().sum();
            let total_decimal = rust_decimal::Decimal::from_f64_retain(total)
                .unwrap_or(rust_decimal::Decimal::ZERO);
            query = query.bind(scores_json);
            query = query.bind(total_decimal);
        }
        if let Some(ref v) = data.comment {
            query = query.bind(v);
        }
        if let Some(ref v) = data.feedback {
            query = query.bind(v);
        }

        let row = query.bind(score_id).fetch_optional(pool).await?;
        let row = row.ok_or_else(|| JudgingError::ScoreNotFound(score_id.to_string()))?;

        Ok(ScoreResponse {
            id: row.id,
            assignment_id: row.assignment_id,
            scores: row.scores,
            total_score: row.total.map(|d| d.to_string()),
            comment: row.comment,
            feedback: row.feedback,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_normalized: false,
            judge_stats: None,
        })
    }

    pub async fn get_dashboard(pool: &PgPool) -> Result<DashboardResponse, JudgingError> {
        let total_projects: i64 =
            sqlx::query_scalar("SELECT COUNT(DISTINCT project_id) FROM judging.assignments")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let total_assigned: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM judging.assignments WHERE status != 'pending' OR status IS NULL",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let total_completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM judging.assignments WHERE status = 'completed'",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let total_pending: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM judging.assignments WHERE status = 'pending'")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let avg_score: Option<rust_decimal::Decimal> = sqlx::query_scalar(
            "SELECT AVG(total_score) FROM judging.scores WHERE total_score IS NOT NULL",
        )
        .fetch_optional(pool)
        .await?
        .flatten();

        Ok(DashboardResponse {
            total_projects,
            total_assigned,
            total_completed,
            total_pending,
            avg_score: avg_score.map(|d| d.to_f64().unwrap_or(0.0)),
        })
    }
}

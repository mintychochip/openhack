use crate::errors::JudgingError;
use crate::models::normalization::{JudgeStats, NormalizationResponse};
use rust_decimal::prelude::ToPrimitive;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn _compute_judge_statistics(
    pool: &PgPool,
    judge_id: Uuid,
    phase_id: Option<Uuid>,
) -> Result<JudgeStats, JudgingError> {
    let phase_filter = if phase_id.is_some() {
        "AND a.phase_id = $3"
    } else {
        ""
    };

    let sql = format!(
        "SELECT AVG(s.total_score) as mean_score,
                COALESCE(STDDEV_POP(s.total_score), 0) as std_dev,
                MIN(s.total_score) as min_score,
                MAX(s.total_score) as max_score,
                COUNT(s.id) as total_scores_count
         FROM judging.scores s
         JOIN judging.assignments a ON s.assignment_id = a.id
         WHERE a.judge_id = $1 AND s.total_score IS NOT NULL {phase_filter}"
    );

    let mut query = sqlx::query(&sql).bind(judge_id);
    if let Some(pid) = phase_id {
        query = query.bind(pid);
    }

    let row = query.fetch_one(pool).await?;

    let mean: Option<rust_decimal::Decimal> = row.get("mean_score");
    let std_dev: Option<rust_decimal::Decimal> = row.get("std_dev");
    let min: Option<rust_decimal::Decimal> = row.get("min_score");
    let max: Option<rust_decimal::Decimal> = row.get("max_score");
    let count: i64 = row.get::<Option<i64>, _>("total_scores_count").unwrap_or(0);

    let mean_score = mean.unwrap_or(rust_decimal::Decimal::ZERO);
    let std_dev_val = std_dev.unwrap_or(rust_decimal::Decimal::ZERO);
    let min_score = min.unwrap_or(rust_decimal::Decimal::ZERO);
    let max_score = max.unwrap_or(rust_decimal::Decimal::ZERO);

    let result = sqlx::query_as::<_, JudgeStats>(
        "INSERT INTO judging.judge_stats (judge_id, phase_id, mean_score, std_dev, min_score, max_score, total_scores_count, computed_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
         ON CONFLICT (judge_id, phase_id) DO UPDATE
         SET mean_score = EXCLUDED.mean_score, std_dev = EXCLUDED.std_dev,
             min_score = EXCLUDED.min_score, max_score = EXCLUDED.max_score,
             total_scores_count = EXCLUDED.total_scores_count, computed_at = NOW()
         RETURNING *",
    )
    .bind(judge_id)
    .bind(phase_id)
    .bind(mean_score)
    .bind(std_dev_val)
    .bind(min_score)
    .bind(max_score)
    .bind(i32::try_from(count).unwrap_or(0))
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn apply_normalization(
    pool: &PgPool,
    phase_id: Uuid,
    method: &str,
) -> Result<i32, JudgingError> {
    let stats =
        sqlx::query_as::<_, JudgeStats>("SELECT * FROM judging.judge_stats WHERE phase_id = $1")
            .bind(phase_id)
            .fetch_all(pool)
            .await?;

    if stats.is_empty() {
        return Err(JudgingError::Normalization(
            "No judge stats found for phase".into(),
        ));
    }

    let scores = sqlx::query_as::<_, crate::models::score::Score>(
        "SELECT s.* FROM judging.scores s
         JOIN judging.assignments a ON s.assignment_id = a.id
         WHERE a.phase_id = $1 AND s.total_score IS NOT NULL",
    )
    .bind(phase_id)
    .fetch_all(pool)
    .await?;

    let mut updated = 0;

    for score in &scores {
        let assignment = sqlx::query_as::<_, crate::models::assignment::Assignment>(
            "SELECT * FROM judging.assignments WHERE id = $1",
        )
        .bind(score.assignment_id)
        .fetch_optional(pool)
        .await?;

        if let Some(assignment) = assignment {
            if let Some(judge_stats) = stats.iter().find(|s| s.judge_id == assignment.judge_id) {
                let raw = score.total.unwrap_or(rust_decimal::Decimal::ZERO);
                let raw_f = raw.to_f64().unwrap_or(0.0);
                let mean_f = judge_stats.mean_score.to_f64().unwrap_or(0.0);
                let std_f = judge_stats.std_dev.to_f64().unwrap_or(1.0);

                let normalized = match method {
                    "zscore" => {
                        if std_f == 0.0 {
                            0.0
                        } else {
                            (raw_f - mean_f) / std_f
                        }
                    }
                    "minmax" => {
                        let min_f = judge_stats.min_score.to_f64().unwrap_or(0.0);
                        let max_f = judge_stats.max_score.to_f64().unwrap_or(100.0);
                        if (max_f - min_f).abs() < f64::EPSILON {
                            50.0
                        } else {
                            ((raw_f - min_f) / (max_f - min_f)) * 100.0
                        }
                    }
                    _ => raw_f,
                };

                let normalized_decimal = rust_decimal::Decimal::from_f64_retain(normalized)
                    .unwrap_or(rust_decimal::Decimal::ZERO);

                let _ = sqlx::query("UPDATE judging.scores SET total_score = $1 WHERE id = $2")
                    .bind(normalized_decimal)
                    .bind(score.id)
                    .execute(pool)
                    .await;

                updated += 1;
            }
        }
    }

    Ok(updated)
}

pub async fn get_normalization_status(
    pool: &PgPool,
    phase_id: Uuid,
) -> Result<NormalizationResponse, JudgingError> {
    let _stats =
        sqlx::query_as::<_, JudgeStats>("SELECT * FROM judging.judge_stats WHERE phase_id = $1")
            .bind(phase_id)
            .fetch_all(pool)
            .await?;

    Ok(NormalizationResponse {
        success: true,
        result: None,
        preview: None,
        errors: vec![],
    })
}

pub async fn get_normalization_stats(
    pool: &PgPool,
    phase_id: Uuid,
) -> Result<serde_json::Value, JudgingError> {
    let stats =
        sqlx::query_as::<_, JudgeStats>("SELECT * FROM judging.judge_stats WHERE phase_id = $1")
            .bind(phase_id)
            .fetch_all(pool)
            .await?;

    Ok(serde_json::json!({
        "judge_stats": stats,
        "phase_id": phase_id,
    }))
}

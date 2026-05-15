use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::errors::AnalyticsError;
use crate::models::chart::{
    CategoryDistributionParams, CategoryDistributionResponse, FunnelResponse, FunnelStage,
    HeatmapCell, HeatmapQueryParams, HeatmapResponse, PieSlice, TimelinePoint, TimelineQueryParams,
    TimelineResponse, CHART_COLORS,
};

/// Retrieve the 4-stage registration funnel.
///
/// # Expected Behavior
///
/// Queries counts for four event types in order:
/// "user.registered" → "team.created" → "project.created" → "project.submitted".
/// Computes conversion rates between consecutive stages. The first stage
/// always has a conversion rate of 100.0. Subsequent stages show the
/// percentage relative to the previous stage (0.0–100.0).
///
/// # Errors
///
/// - `AnalyticsError::Database` if any SQL query fails.
///
/// # Side Effects
///
/// - Executes 4 SQL COUNT queries against the database (read-only).
pub async fn get_registration_funnel(pool: &PgPool) -> Result<FunnelResponse, AnalyticsError> {
    let stages_config = [
        ("Registered", "user.registered"),
        ("Joined Team", "team.created"),
        ("Created Project", "project.created"),
        ("Submitted", "project.submitted"),
    ];

    let mut stages: Vec<FunnelStage> = Vec::new();

    for (name, event_type) in &stages_config {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM analytics.events WHERE event_type = $1")
                .bind(*event_type)
                .fetch_one(pool)
                .await?;

        stages.push(FunnelStage {
            name: name.to_string(),
            count: count.0,
            conversion_rate: 100.0,
        });
    }

    for i in 1..stages.len() {
        if stages[i - 1].count > 0 {
            stages[i].conversion_rate =
                (stages[i].count as f64 / stages[i - 1].count as f64) * 100.0;
        } else {
            stages[i].conversion_rate = 0.0;
        }
    }

    Ok(FunnelResponse { stages })
}

/// Retrieve time-bucketed submission timeline with peak detection.
///
/// # Expected Behavior
///
/// Queries the `analytics.events` table for "project.submitted" events
/// within the last `params.days` days, grouped by `date_trunc` using
/// "hour" or "day" depending on `params.period`. Marks the bucket(s)
/// with the highest count as `is_peak = true`. Returns a `TimelineResponse`.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn get_submission_timeline(
    pool: &PgPool,
    params: &TimelineQueryParams,
) -> Result<TimelineResponse, AnalyticsError> {
    let trunc = if params.period == "hourly" {
        "hour"
    } else {
        "day"
    };
    let since = Utc::now() - Duration::days(params.days);

    let query = format!(
        "SELECT date_trunc('{trunc}', occurred_at) AS bucket, COUNT(*) AS count \
         FROM analytics.events WHERE event_type = $1 AND occurred_at >= $2 \
         GROUP BY bucket ORDER BY bucket"
    );

    let rows: Vec<(chrono::DateTime<Utc>, i64)> = sqlx::query_as(&query)
        .bind("project.submitted")
        .bind(since)
        .fetch_all(pool)
        .await?;

    let max_count = rows.iter().map(|(_, c)| *c).max().unwrap_or(0);

    let data: Vec<TimelinePoint> = rows
        .into_iter()
        .map(|(bucket, count)| TimelinePoint {
            bucket: bucket.to_rfc3339(),
            count,
            is_peak: count == max_count && max_count > 0,
        })
        .collect();

    Ok(TimelineResponse { data })
}

/// Retrieve category distribution as pie chart data.
///
/// # Expected Behavior
///
/// Queries the `analytics.events` table for "project.created" events,
/// grouping by the `category_field` key in the `event_data` JSONB column.
/// Assigns colors from `CHART_COLORS` cycling as needed. Returns a
/// `CategoryDistributionResponse`.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn get_category_distribution(
    pool: &PgPool,
    params: &CategoryDistributionParams,
) -> Result<CategoryDistributionResponse, AnalyticsError> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT jsonb_extract_path_text(event_data, $1) AS category, COUNT(*) AS count \
         FROM analytics.events WHERE event_type = 'project.created' \
         AND jsonb_extract_path_text(event_data, $1) IS NOT NULL \
         GROUP BY category ORDER BY count DESC",
    )
    .bind(&params.category_field)
    .fetch_all(pool)
    .await?;

    let data: Vec<PieSlice> = rows
        .into_iter()
        .enumerate()
        .map(|(i, (label, value))| PieSlice {
            label,
            value,
            color: CHART_COLORS[i % CHART_COLORS.len()].to_string(),
        })
        .collect();

    Ok(CategoryDistributionResponse { data })
}

/// Retrieve engagement heatmap data.
///
/// # Expected Behavior
///
/// Queries the `analytics.events` table for events with a non-null `user_id`
/// within the last `params.days` days. Groups by day-of-week (0–6,
/// Sunday–Saturday) and hour (0–23). Returns a `HeatmapResponse` with
/// an ordered list of `HeatmapCell`.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn get_engagement_heatmap(
    pool: &PgPool,
    params: &HeatmapQueryParams,
) -> Result<HeatmapResponse, AnalyticsError> {
    let since = Utc::now() - Duration::days(params.days);

    let rows: Vec<(i32, i32, i64)> = sqlx::query_as(
        "SELECT EXTRACT(DOW FROM occurred_at)::int AS day, \
         EXTRACT(HOUR FROM occurred_at)::int AS hour, COUNT(*) AS count \
         FROM analytics.events WHERE user_id IS NOT NULL AND occurred_at >= $1 \
         GROUP BY day, hour ORDER BY day, hour",
    )
    .bind(since)
    .fetch_all(pool)
    .await?;

    let data: Vec<HeatmapCell> = rows
        .into_iter()
        .map(|(day, hour, count)| HeatmapCell { day, hour, count })
        .collect();

    Ok(HeatmapResponse { data })
}

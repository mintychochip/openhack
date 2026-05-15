use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::errors::AnalyticsError;
use crate::models::chart::metric_type_to_event_type;
use crate::models::metric::{
    CategoryCount, MetricsByCategoryParams, MetricsOverTimeParams, MetricsSummary, TimeBucket,
};

/// Retrieve aggregate summary metrics.
///
/// # Expected Behavior
///
/// Queries the `analytics.events` table for total counts of registrations,
/// teams, projects, and submissions. Also computes active users (distinct
/// `user_id` with events in the last 30 days). Returns a `MetricsSummary`.
///
/// # Errors
///
/// - `AnalyticsError::Database` if any SQL query fails.
///
/// # Side Effects
///
/// - Executes 5 SQL queries against the database (read-only).
/// - Logs at ERROR level on query failure.
pub async fn get_summary(pool: &PgPool) -> Result<MetricsSummary, AnalyticsError> {
    let total_registrations: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM analytics.events WHERE event_type = $1")
            .bind("user.registered")
            .fetch_one(pool)
            .await?;

    let total_teams: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM analytics.events WHERE event_type = $1")
            .bind("team.created")
            .fetch_one(pool)
            .await?;

    let total_projects: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM analytics.events WHERE event_type = $1")
            .bind("project.created")
            .fetch_one(pool)
            .await?;

    let total_submissions: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM analytics.events WHERE event_type = $1")
            .bind("project.submitted")
            .fetch_one(pool)
            .await?;

    let thirty_days_ago = Utc::now() - Duration::days(30);
    let active_users: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT user_id) FROM analytics.events WHERE user_id IS NOT NULL AND occurred_at >= $1",
    )
    .bind(thirty_days_ago)
    .fetch_one(pool)
    .await?;

    Ok(MetricsSummary {
        total_registrations: total_registrations.0,
        total_teams: total_teams.0,
        total_projects: total_projects.0,
        total_submissions: total_submissions.0,
        active_users: active_users.0,
    })
}

/// Retrieve time-bucketed metrics over a given period.
///
/// # Expected Behavior
///
/// Maps `params.metric_type` to an event type, then queries the
/// `analytics.events` table with `date_trunc` using the specified period
/// ("hourly" or "daily"). Returns a list of `TimeBucket` with ISO-formatted
/// bucket timestamps and counts. Looks back `params.days` days from now.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn get_metrics_over_time(
    pool: &PgPool,
    params: &MetricsOverTimeParams,
) -> Result<Vec<TimeBucket>, AnalyticsError> {
    let event_type = metric_type_to_event_type(&params.metric_type);
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
        .bind(event_type)
        .bind(since)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|(bucket, count)| TimeBucket {
            bucket: bucket.to_rfc3339(),
            count,
        })
        .collect())
}

/// Retrieve a category breakdown for a given metric.
///
/// # Expected Behavior
///
/// Maps `params.metric_name` to an event type, then queries the
/// `analytics.events` table grouping by the `category` key in the
/// `event_data` JSONB column. Returns a list of `CategoryCount` ordered
/// by count descending.
///
/// # Errors
///
/// - `AnalyticsError::Database` if the SQL query fails.
///
/// # Side Effects
///
/// - Executes 1 SQL query against the database (read-only).
pub async fn get_metrics_by_category(
    pool: &PgPool,
    params: &MetricsByCategoryParams,
) -> Result<Vec<CategoryCount>, AnalyticsError> {
    let event_type = metric_type_to_event_type(&params.metric_name);

    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT jsonb_extract_path_text(event_data, 'category') AS category, COUNT(*) AS count \
         FROM analytics.events WHERE event_type = $1 \
         AND jsonb_extract_path_text(event_data, 'category') IS NOT NULL \
         GROUP BY category ORDER BY count DESC",
    )
    .bind(event_type)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect())
}

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Create a `PostgreSQL` connection pool with tuned settings.
///
/// # Errors
///
/// Returns `sqlx::Error` if the pool cannot be created.
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;

    log::info!("PostgreSQL connection pool created successfully (max_conns={max_connections})");
    Ok(pool)
}

/// Verify database connectivity by executing `SELECT 1`.
///
/// # Errors
///
/// Returns `sqlx::Error` if the query fails.
pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => {
            log::info!("Database health check passed");
            Ok(())
        }
        Err(e) => {
            log::error!("Database health check failed: {e}");
            Err(e)
        }
    }
}

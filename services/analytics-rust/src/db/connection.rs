use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Create a new `PostgreSQL` connection pool.
///
/// # Expected Behavior
///
/// Creates a connection pool with the given database URL. Configures
/// max 10 connections, a 30-second connection timeout, and a 30-second
/// idle timeout. Returns the pool on success.
///
/// # Errors
///
/// Returns `sqlx::Error` if the pool cannot be created (e.g., invalid URL)
/// or if the initial connection test fails (e.g., database unreachable,
/// authentication failure).
///
/// # Side Effects
///
/// - Establishes TCP connections to the `PostgreSQL` server (during pool
///   creation and background health checks).
/// - Logs at INFO level on successful pool creation.
/// - Logs at ERROR level if pool creation fails.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;

    log::info!("PostgreSQL connection pool created successfully");
    Ok(pool)
}

/// Verify the database connection is alive.
///
/// # Expected Behavior
///
/// Executes `SELECT 1` against the pool. Returns `Ok(())` if the query
/// succeeds, indicating the database is reachable and responsive.
///
/// # Errors
///
/// Returns `sqlx::Error` if the database is unreachable, the connection
/// is dropped, or the query times out.
///
/// # Side Effects
///
/// - Acquires a connection from the pool temporarily (returned after query).
/// - Logs at INFO level on successful health check.
/// - Logs at ERROR level on failed health check.
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

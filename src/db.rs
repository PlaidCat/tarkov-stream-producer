use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Error;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, Error> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn init_schema(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS raids (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            map TEXT NOT NULL,
            started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            ended_at TIMESTAMP,
            kills INTEGER NOT NULL DEFAULT 0,
            survived BOOLEAN NOT NULL DEFAULT 0
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        // use in-memory database for testing
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("Failed to Execute test query");

        // Verify connection by running a simple query
        let result: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to execute test query");

        assert_eq!(result.0, 1);
    }

    #[tokio::test]
    async fn test_schema_initialization() {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("Failed to create pool");

        // Initialize schema
        init_schema(&pool)
            .await
            .expect("Failed to initialize schema");

        // Verify table exists by qurying sqlite_master
        let table_exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='raids'"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check table existence");

        assert_eq!(table_exists.0, 1);
    }
}

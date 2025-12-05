//! # PostgreSQL Database Integration
//!
//! High-performance PostgreSQL integration using SQLx with connection pooling,
//! transaction support, and query building capabilities.

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, types::Json, PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

use crate::config::DatabaseConfig;

/// Database manager for PostgreSQL operations
pub struct DatabaseManager {
    pool: PgPool,
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// Create a new database manager with the given configuration
    #[instrument(skip(config), fields(database_url = %config.database_url))]
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Initializing database connection pool");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&config.database_url)
            .await?;

        // Test the connection
        let version: String = sqlx::query_scalar("SELECT version()")
            .fetch_one(&pool)
            .await?;

        info!("Connected to PostgreSQL: {}", version);

        Ok(Self { pool, config })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Execute a query and return typed results
    #[instrument(skip(self, params), fields(query = %query))]
    pub async fn execute_query<T>(&self, query: &str, params: &[Json<T>]) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Unpin,
    {
        debug!("Executing query: {}", query);

        let mut query_builder = sqlx::query_as::<_, T>(query);

        // Bind JSON parameters
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        debug!("Query returned {} rows", rows.len());
        Ok(rows)
    }

    /// Execute a raw SQL query
    #[instrument(skip(self), fields(query = %query))]
    pub async fn execute_raw(&self, query: &str) -> Result<()> {
        debug!("Executing raw query: {}", query);

        sqlx::query(query).execute(&self.pool).await?;

        Ok(())
    }

    /// Execute a query with a single result
    #[instrument(skip(self, params), fields(query = %query))]
    pub async fn execute_query_one<T>(&self, query: &str, params: &[Json<T>]) -> Result<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Unpin,
    {
        debug!("Executing query for single result: {}", query);

        let mut query_builder = sqlx::query_as::<_, T>(query);

        // Bind JSON parameters
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let result = query_builder.fetch_optional(&self.pool).await?;

        Ok(result)
    }

    /// Execute a transaction with automatic rollback on error
    #[instrument(skip(self, operation))]
    pub async fn execute_transaction<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        debug!("Starting database transaction");

        let mut tx = self.pool.begin().await?;

        // Execute the operation
        let result = operation(&mut tx).await?;

        // Commit the transaction
        tx.commit().await?;

        debug!("Transaction completed successfully");
        Ok(result)
    }

    /// Execute a transaction with manual control
    #[instrument(skip(self, operation))]
    pub async fn execute_transaction_manual<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> Fut,
        Fut: std::future::Future<Output = Result<(T, bool)>>, // (result, should_commit)
    {
        debug!("Starting manual transaction");

        let mut tx = self.pool.begin().await?;

        // Execute the operation
        let (result, should_commit) = operation(&mut tx).await?;

        if should_commit {
            tx.commit().await?;
            debug!("Transaction committed");
        } else {
            tx.rollback().await?;
            debug!("Transaction rolled back");
        }

        Ok(result)
    }

    /// Check database health
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<()> {
        debug!("Performing database health check");

        let result: (i32,) = sqlx::query_as("SELECT 1 as health")
            .fetch_one(&self.pool)
            .await?;

        if result.0 == 1 {
            info!("Database health check passed");
            Ok(())
        } else {
            error!("Database health check failed");
            Err(anyhow::anyhow!("Health check returned unexpected result"))
        }
    }

    /// Get database statistics
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        debug!("Gathering database statistics");

        let stats: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT
                (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections,
                (SELECT count(*) FROM pg_stat_activity) as active_connections,
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_queries,
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'idle') as idle_connections"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            max_connections: stats.0 as u32,
            active_connections: stats.1 as u32,
            active_queries: stats.2 as u32,
            idle_connections: stats.3 as u32,
        })
    }

    /// Create a backup of the database
    #[instrument(skip(self))]
    pub async fn create_backup(&self, backup_name: &str) -> Result<()> {
        debug!("Creating database backup: {}", backup_name);

        // This would typically use pg_dump, but for now we'll use SQLx's built-in functionality
        // In a real implementation, you'd want to use pg_dump or similar tool
        warn!("Database backup functionality is a placeholder - implement with pg_dump");

        Ok(())
    }

    /// Get the current database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get connection pool size
    pub fn pool_size(&self) -> u32 {
        self.pool.size()
    }

    /// Get number of idle connections
    pub fn idle_connections(&self) -> u32 {
        self.pool.num_idle() as u32
    }

    /// Get number of used connections
    pub fn used_connections(&self) -> u32 {
        self.pool.size().saturating_sub(self.pool.num_idle() as u32)
    }
}

/// Database statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseStats {
    pub max_connections: u32,
    pub active_connections: u32,
    pub active_queries: u32,
    pub idle_connections: u32,
}

impl std::fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Database Stats: max={}, active={}, idle={}, queries={}",
            self.max_connections,
            self.active_connections,
            self.idle_connections,
            self.active_queries
        )
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_config_creation() {
        let config = DatabaseConfig {
            database_url: "postgresql://localhost/test".to_string(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            enable_ssl: false,
            connect_timeout: Duration::from_secs(10),
        };

        assert_eq!(config.max_connections, 10);
        assert_eq!(config.database_url, "postgresql://localhost/test");
        assert!(!config.enable_ssl);
    }

    #[test]
    fn test_database_stats_display() {
        let stats = DatabaseStats {
            max_connections: 100,
            active_connections: 25,
            active_queries: 10,
            idle_connections: 15,
        };

        let display = format!("{}", stats);
        assert!(display.contains("max=100"));
        assert!(display.contains("active=25"));
        assert!(display.contains("idle=15"));
        assert!(display.contains("queries=10"));
    }
}

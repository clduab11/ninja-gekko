//! # Database Migration System
//!
//! File-based database migration system with locking, versioning,
//! and rollback capabilities for PostgreSQL.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs as async_fs;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

use crate::config::MigrationConfig;

/// Migration status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    Pending,
    Applied,
    Failed,
    RollingBack,
    RolledBack,
}

/// Database migration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub description: String,
    pub checksum: String,
    pub applied_at: Option<SystemTime>,
    pub rolled_back_at: Option<SystemTime>,
    pub status: MigrationStatus,
    pub execution_time_ms: Option<u64>,
}

/// Migration lock for preventing concurrent migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationLock {
    pub locked_at: SystemTime,
    pub locked_by: String,
    pub expires_at: SystemTime,
}

/// Migration manager for handling database schema changes
pub struct MigrationManager {
    config: MigrationConfig,
    migration_dir: PathBuf,
    applied_migrations: Mutex<HashMap<i64, Migration>>,
}

impl MigrationManager {
    /// Create a new migration manager
    #[instrument(skip(config))]
    pub fn new(config: MigrationConfig, migration_dir: impl AsRef<Path>) -> Result<Self> {
        info!("Initializing migration manager");

        let migration_dir = migration_dir.as_ref().to_path_buf();

        // Ensure migration directory exists
        if !migration_dir.exists() {
            fs::create_dir_all(&migration_dir)?;
            info!("Created migration directory: {}", migration_dir.display());
        }

        Ok(Self {
            config,
            migration_dir,
            applied_migrations: Mutex::new(HashMap::new()),
        })
    }

    /// Load applied migrations from database
    #[instrument(skip(self, pool))]
    pub async fn load_applied_migrations(&self, pool: &sqlx::PgPool) -> Result<()> {
        debug!("Loading applied migrations from database");

        let rows = sqlx::query!(
            r#"
            SELECT version, name, description, checksum, applied_at, status, execution_time_ms
            FROM schema_migrations
            ORDER BY version ASC
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut applied = HashMap::new();
        for row in rows {
            let status = match row.status.as_str() {
                "applied" => MigrationStatus::Applied,
                "failed" => MigrationStatus::Failed,
                "rolled_back" => MigrationStatus::RolledBack,
                _ => MigrationStatus::Pending,
            };

            let migration = Migration {
                version: row.version,
                name: row.name,
                description: row.description,
                checksum: row.checksum,
                applied_at: row.applied_at.map(|dt| dt.into()),
                rolled_back_at: None, // TODO: Add rolled_back_at to schema
                status,
                execution_time_ms: row.execution_time_ms.map(|ms| ms as u64),
            };

            applied.insert(row.version, migration);
        }

        let mut migrations = self.applied_migrations.lock().await;
        *migrations = applied;

        info!("Loaded {} applied migrations", migrations.len());
        Ok(())
    }

    /// Create a new migration file
    #[instrument(skip(self))]
    pub async fn create_migration(&self, name: &str, description: &str) -> Result<PathBuf> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let filename = format!("V{:013}__{}.sql", timestamp, name);
        let filepath = self.migration_dir.join(&filename);

        // Check if file already exists
        if filepath.exists() {
            return Err(anyhow!(
                "Migration file already exists: {}",
                filepath.display()
            ));
        }

        let content = format!(
            r#"-- Migration: {}
-- Description: {}
-- Created: {}

-- Write your UP migration SQL here
-- Example:
-- CREATE TABLE example (
--     id SERIAL PRIMARY KEY,
--     name VARCHAR(255) NOT NULL,
--     created_at TIMESTAMP DEFAULT NOW()
-- );

-- Write your DOWN migration SQL here
-- Example:
-- DROP TABLE example;
"#,
            name,
            description,
            chrono::Utc::now().to_rfc3339()
        );

        async_fs::write(&filepath, content).await?;

        info!("Created migration file: {}", filepath.display());
        Ok(filepath)
    }

    /// Get pending migrations that haven't been applied yet
    #[instrument(skip(self))]
    pub async fn get_pending_migrations(&self) -> Result<Vec<MigrationFile>> {
        debug!("Finding pending migrations");

        let mut pending = Vec::new();

        // Get all migration files
        let mut entries = async_fs::read_dir(&self.migration_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(migration_file) = self.parse_migration_filename(filename)? {
                        // Check if already applied
                        let applied_migrations = self.applied_migrations.lock().await;
                        if !applied_migrations.contains_key(&migration_file.version) {
                            let content = async_fs::read_to_string(&path).await?;
                            let migration_file = MigrationFile {
                                path,
                                content,
                                ..migration_file
                            };
                            pending.push(migration_file);
                        }
                    }
                }
            }
        }

        // Sort by version
        pending.sort_by_key(|m| m.version);

        info!("Found {} pending migrations", pending.len());
        Ok(pending)
    }

    /// Parse migration filename to extract version and name
    fn parse_migration_filename(&self, filename: &str) -> Result<Option<MigrationFile>> {
        let parts: Vec<&str> = filename.split("__").collect();
        if parts.len() != 2 {
            return Ok(None);
        }

        let version_part = parts[0];
        let name_part = parts[1];

        if !version_part.starts_with('V') {
            return Ok(None);
        }

        let version_str = &version_part[1..];
        let version = version_part[1..]
            .parse::<i64>()
            .map_err(|e| anyhow!("Invalid version number '{}': {}", version_str, e))?;

        let name = name_part
            .strip_suffix(".sql")
            .unwrap_or(name_part)
            .to_string();

        Ok(Some(MigrationFile {
            version,
            name,
            description: String::new(), // Will be read from file
            checksum: String::new(),    // Will be calculated
            path: PathBuf::new(),
            content: String::new(),
        }))
    }

    /// Calculate checksum for migration content
    fn calculate_checksum(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Acquire migration lock to prevent concurrent migrations
    #[instrument(skip(self, pool))]
    async fn acquire_lock(&self, pool: &sqlx::PgPool, lock_id: &str) -> Result<()> {
        debug!("Acquiring migration lock: {}", lock_id);

        let expires_at = SystemTime::now() + Duration::from_secs(self.config.lock_timeout_seconds);

        let result = sqlx::query!(
            r#"
            INSERT INTO schema_migration_locks (lock_id, locked_by, expires_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (lock_id) DO UPDATE SET
                locked_by = EXCLUDED.locked_by,
                expires_at = EXCLUDED.expires_at
            RETURNING lock_id
            "#,
            lock_id,
            "migration_manager",
            expires_at.duration_since(UNIX_EPOCH)?.as_secs() as i64
        )
        .execute(pool)
        .await;

        match result {
            Ok(_) => {
                info!("Successfully acquired migration lock: {}", lock_id);
                Ok(())
            }
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.constraint().is_some() {
                    error!("Failed to acquire migration lock, another process may be running migrations");
                    Err(anyhow!("Migration lock is held by another process"))
                } else {
                    Err(anyhow!("Database error acquiring lock: {}", db_err))
                }
            }
            Err(e) => Err(anyhow!("Error acquiring migration lock: {}", e)),
        }
    }

    /// Release migration lock
    #[instrument(skip(self, pool))]
    async fn release_lock(&self, pool: &sqlx::PgPool, lock_id: &str) -> Result<()> {
        debug!("Releasing migration lock: {}", lock_id);

        let rows_affected = sqlx::query!(
            "DELETE FROM schema_migration_locks WHERE lock_id = $1",
            lock_id
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected > 0 {
            info!("Released migration lock: {}", lock_id);
        } else {
            warn!("Migration lock not found or already released: {}", lock_id);
        }

        Ok(())
    }

    /// Run all pending migrations
    #[instrument(skip(self, pool))]
    pub async fn run_migrations(&self, pool: &sqlx::PgPool) -> Result<MigrationResult> {
        info!("Starting migration run");

        // Acquire lock
        self.acquire_lock(pool, "schema_migration").await?;

        let result = self.run_migrations_locked(pool).await;

        // Release lock
        self.release_lock(pool, "schema_migration").await?;

        match &result {
            Ok(summary) => info!("Migration completed successfully: {:?}", summary),
            Err(e) => error!("Migration failed: {}", e),
        }

        result
    }

    /// Run migrations with lock held
    #[instrument(skip(self, pool))]
    async fn run_migrations_locked(&self, pool: &sqlx::PgPool) -> Result<MigrationResult> {
        let pending = self.get_pending_migrations().await?;

        if pending.is_empty() {
            info!("No pending migrations to run");
            return Ok(MigrationResult {
                total_migrations: 0,
                successful_migrations: 0,
                failed_migrations: 0,
                total_time_ms: 0,
                migrations: Vec::new(),
            });
        }

        info!("Running {} pending migrations", pending.len());

        let start_time = std::time::Instant::now();
        let mut successful = 0;
        let mut failed = 0;
        let mut migration_results = Vec::new();

        for migration_file in pending {
            let migration_result = self.apply_migration(pool, &migration_file).await;

            match migration_result {
                Ok(_) => successful += 1,
                Err(_) => failed += 1,
            }

            migration_results.push(migration_result);
        }

        let total_time = start_time.elapsed().as_millis() as u64;

        Ok(MigrationResult {
            total_migrations: pending.len(),
            successful_migrations: successful,
            failed_migrations: failed,
            total_time_ms: total_time,
            migrations: migration_results,
        })
    }

    /// Apply a single migration
    #[instrument(skip(self, pool, migration_file))]
    async fn apply_migration(
        &self,
        pool: &sqlx::PgPool,
        migration_file: &MigrationFile,
    ) -> Result<()> {
        debug!(
            "Applying migration: {} - {}",
            migration_file.version, migration_file.name
        );

        let start_time = std::time::Instant::now();

        // Begin transaction
        let mut tx = pool.begin().await?;

        // Create schema_migrations table if it doesn't exist
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                checksum VARCHAR(64) NOT NULL,
                applied_at TIMESTAMP WITH TIME ZONE,
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                execution_time_ms BIGINT
            )
            "#,
        )
        .execute(&mut *tx)
        .await?;

        // Parse and execute the migration SQL
        match self.execute_migration_sql(&mut *tx, migration_file).await {
            Ok(_) => {
                // Record successful migration
                let execution_time = start_time.elapsed().as_millis() as u64;

                sqlx::query!(
                    r#"
                    INSERT INTO schema_migrations (version, name, description, checksum, applied_at, status, execution_time_ms)
                    VALUES ($1, $2, $3, $4, NOW(), 'applied', $5)
                    "#,
                    migration_file.version,
                    migration_file.name,
                    migration_file.description,
                    migration_file.checksum,
                    execution_time as i64
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;

                // Update in-memory cache
                let mut applied_migrations = self.applied_migrations.lock().await;
                applied_migrations.insert(
                    migration_file.version,
                    Migration {
                        version: migration_file.version,
                        name: migration_file.name.clone(),
                        description: migration_file.description.clone(),
                        checksum: migration_file.checksum.clone(),
                        applied_at: Some(SystemTime::now()),
                        rolled_back_at: None,
                        status: MigrationStatus::Applied,
                        execution_time_ms: Some(execution_time),
                    },
                );

                info!(
                    "Successfully applied migration: {} - {}",
                    migration_file.version, migration_file.name
                );
                Ok(())
            }
            Err(e) => {
                // Record failed migration
                sqlx::query!(
                    r#"
                    INSERT INTO schema_migrations (version, name, description, checksum, applied_at, status, execution_time_ms)
                    VALUES ($1, $2, $3, $4, NOW(), 'failed', $5)
                    "#,
                    migration_file.version,
                    migration_file.name,
                    migration_file.description,
                    migration_file.checksum,
                    start_time.elapsed().as_millis() as i64
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;

                error!(
                    "Failed to apply migration {} - {}: {}",
                    migration_file.version, migration_file.name, e
                );
                Err(e)
            }
        }
    }

    /// Execute migration SQL
    #[instrument(skip(self, tx, migration_file))]
    async fn execute_migration_sql(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        migration_file: &MigrationFile,
    ) -> Result<()> {
        debug!(
            "Executing migration SQL for: {} - {}",
            migration_file.version, migration_file.name
        );

        // Split SQL by statements (basic semicolon splitting)
        let statements: Vec<&str> = migration_file
            .content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .collect();

        for statement in statements {
            if !statement.is_empty() {
                sqlx::query(statement).execute(&mut **tx).await?;
            }
        }

        debug!("Successfully executed migration SQL");
        Ok(())
    }

    /// Rollback the last applied migration
    #[instrument(skip(self, pool))]
    pub async fn rollback_migration(&self, pool: &sqlx::PgPool) -> Result<Option<Migration>> {
        info!("Rolling back last migration");

        self.acquire_lock(pool, "schema_migration_rollback").await?;

        let result = self.rollback_migration_locked(pool).await;

        self.release_lock(pool, "schema_migration_rollback").await?;

        match result {
            Ok(Some(migration)) => {
                info!(
                    "Successfully rolled back migration: {} - {}",
                    migration.version, migration.name
                );
                Ok(Some(migration))
            }
            Ok(None) => {
                info!("No migrations to rollback");
                Ok(None)
            }
            Err(e) => {
                error!("Failed to rollback migration: {}", e);
                Err(e)
            }
        }
    }

    /// Rollback migration with lock held
    #[instrument(skip(self, pool))]
    async fn rollback_migration_locked(&self, pool: &sqlx::PgPool) -> Result<Option<Migration>> {
        let applied_migrations = self.applied_migrations.lock().await;
        let mut versions: Vec<i64> = applied_migrations.keys().cloned().collect();
        versions.sort_by(|a, b| b.cmp(a)); // Sort descending

        if let Some(&latest_version) = versions.first() {
            if let Some(migration) = applied_migrations.get(&latest_version) {
                if migration.status == MigrationStatus::Applied {
                    // Find the corresponding migration file for rollback SQL
                    let migration_file = self.find_migration_file(latest_version).await?;

                    let mut tx = pool.begin().await?;

                    // Execute rollback SQL
                    if let Some(rollback_sql) = self.extract_rollback_sql(&migration_file.content) {
                        let statements: Vec<&str> = rollback_sql
                            .split(';')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty() && !s.starts_with("--"))
                            .collect();

                        for statement in statements {
                            if !statement.is_empty() {
                                sqlx::query(statement).execute(&mut *tx).await?;
                            }
                        }
                    }

                    // Update migration record
                    sqlx::query!(
                        "UPDATE schema_migrations SET status = 'rolled_back', rolled_back_at = NOW() WHERE version = $1",
                        latest_version
                    )
                    .execute(&mut *tx)
                    .await?;

                    tx.commit().await?;

                    // Update in-memory cache
                    drop(applied_migrations);
                    let mut applied_migrations = self.applied_migrations.lock().await;
                    if let Some(migration) = applied_migrations.get_mut(&latest_version) {
                        migration.status = MigrationStatus::RolledBack;
                        migration.rolled_back_at = Some(SystemTime::now());
                    }

                    return Ok(applied_migrations.remove(&latest_version));
                }
            }
        }

        Ok(None)
    }

    /// Find migration file by version
    async fn find_migration_file(&self, version: i64) -> Result<MigrationFile> {
        let mut entries = async_fs::read_dir(&self.migration_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(migration_file) = self.parse_migration_filename(filename)? {
                        if migration_file.version == version {
                            let content = async_fs::read_to_string(&path).await?;
                            return Ok(MigrationFile {
                                content,
                                checksum: Self::calculate_checksum(&content),
                                ..migration_file
                            });
                        }
                    }
                }
            }
        }

        Err(anyhow!("Migration file not found for version: {}", version))
    }

    /// Extract rollback SQL from migration content
    fn extract_rollback_sql(&self, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_down_section = false;
        let mut down_sql = Vec::new();

        for line in lines {
            let line = line.trim();

            if line.starts_with("-- DOWN") || line.contains("DOWN MIGRATION") {
                in_down_section = true;
                continue;
            }

            if line.starts_with("-- UP") || line.contains("UP MIGRATION") {
                in_down_section = false;
                continue;
            }

            if in_down_section && !line.is_empty() && !line.starts_with("--") {
                down_sql.push(line);
            }
        }

        if down_sql.is_empty() {
            None
        } else {
            Some(down_sql.join("\n"))
        }
    }

    /// Get migration status
    #[instrument(skip(self))]
    pub async fn get_status(&self) -> MigrationStatusReport {
        let applied_migrations = self.applied_migrations.lock().await;
        let pending_migrations = self.get_pending_migrations().await.unwrap_or_default();

        MigrationStatusReport {
            total_applied: applied_migrations.len(),
            total_pending: pending_migrations.len(),
            applied_migrations: applied_migrations.values().cloned().collect(),
            pending_migrations: pending_migrations
                .iter()
                .map(|m| m.to_migration())
                .collect(),
        }
    }
}

/// Migration file representation
#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: i64,
    pub name: String,
    pub description: String,
    pub checksum: String,
    pub path: PathBuf,
    pub content: String,
}

impl MigrationFile {
    /// Convert to Migration struct
    pub fn to_migration(&self) -> Migration {
        Migration {
            version: self.version,
            name: self.name.clone(),
            description: self.description.clone(),
            checksum: self.checksum.clone(),
            applied_at: None,
            rolled_back_at: None,
            status: MigrationStatus::Pending,
            execution_time_ms: None,
        }
    }
}

/// Migration execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub total_migrations: usize,
    pub successful_migrations: usize,
    pub failed_migrations: usize,
    pub total_time_ms: u64,
    pub migrations: Vec<Result<()>>,
}

/// Migration status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatusReport {
    pub total_applied: usize,
    pub total_pending: usize,
    pub applied_migrations: Vec<Migration>,
    pub pending_migrations: Vec<Migration>,
}

/// Migration error types
#[derive(thiserror::Error, Debug)]
pub enum MigrationError {
    #[error("Migration lock error: {0}")]
    LockError(String),

    #[error("Migration not found: {0}")]
    MigrationNotFoundError(String),

    #[error("Migration execution error: {0}")]
    ExecutionError(String),

    #[error("Migration file error: {0}")]
    FileError(String),

    #[error("Migration checksum mismatch: {0}")]
    ChecksumError(String),

    #[error("Migration transaction error: {0}")]
    TransactionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_migration_filename() {
        let manager =
            MigrationManager::new(MigrationConfig::default(), TempDir::new().unwrap().path())
                .unwrap();

        let result = manager.parse_migration_filename("V0000000000001__create_users_table.sql");
        assert!(result.is_ok());

        let migration_file = result.unwrap().unwrap();
        assert_eq!(migration_file.version, 1);
        assert_eq!(migration_file.name, "create_users_table");
    }

    #[test]
    fn test_parse_invalid_migration_filename() {
        let manager =
            MigrationManager::new(MigrationConfig::default(), TempDir::new().unwrap().path())
                .unwrap();

        let result = manager.parse_migration_filename("invalid_filename.sql");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_calculate_checksum() {
        let content = "CREATE TABLE users (id SERIAL PRIMARY KEY);";
        let checksum = MigrationManager::calculate_checksum(content);
        assert_eq!(checksum.len(), 64); // SHA256 hex length
    }

    #[tokio::test]
    async fn test_migration_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = MigrationConfig {
            migration_dir: temp_dir.path().to_path_buf(),
            lock_timeout_seconds: 300,
            enable_migration_locking: true,
        };

        let manager = MigrationManager::new(config, &temp_dir).unwrap();
        assert_eq!(manager.migration_dir, temp_dir.path());
    }

    #[test]
    fn test_migration_status() {
        assert_eq!(MigrationStatus::Pending, MigrationStatus::Pending);
        assert_eq!(MigrationStatus::Applied, MigrationStatus::Applied);
        assert_eq!(MigrationStatus::Failed, MigrationStatus::Failed);
    }

    #[test]
    fn test_migration_result() {
        let result = MigrationResult {
            total_migrations: 3,
            successful_migrations: 2,
            failed_migrations: 1,
            total_time_ms: 1500,
            migrations: vec![Ok(()), Ok(()), Err(anyhow!("error"))],
        };

        assert_eq!(result.total_migrations, 3);
        assert_eq!(result.successful_migrations, 2);
        assert_eq!(result.failed_migrations, 1);
    }
}

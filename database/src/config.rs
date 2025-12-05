//! # Database Configuration
//!
//! Configuration structures for database connections, caching, and performance settings.
//! Provides environment-based configuration with validation and defaults.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Database configuration for PostgreSQL connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection acquisition timeout
    pub acquire_timeout: Duration,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Connection max lifetime
    pub max_lifetime: Duration,
    /// Enable SSL/TLS for connections
    pub enable_ssl: bool,
    /// Connection timeout
    pub connect_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost:5432/trading".to_string()),
            max_connections: 20,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            enable_ssl: true,
            connect_timeout: Duration::from_secs(10),
        }
    }
}

/// Redis cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Command timeout
    pub command_timeout: Duration,
    /// Enable key eviction
    pub enable_eviction: bool,
    /// Maximum memory usage (in MB)
    pub max_memory_mb: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            command_timeout: Duration::from_secs(10),
            enable_eviction: true,
            max_memory_mb: 100,
        }
    }
}

/// Supabase configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase project URL
    pub project_url: String,
    /// Supabase anonymous key
    pub anon_key: String,
    /// Supabase service role key (if available)
    pub service_role_key: Option<String>,
    /// Database password for migrations
    pub database_password: Option<String>,
    /// Enable Row Level Security
    pub enable_rls: bool,
    /// JWT secret for custom authentication
    pub jwt_secret: Option<String>,
}

impl Default for SupabaseConfig {
    fn default() -> Self {
        Self {
            project_url: std::env::var("SUPABASE_URL")
                .unwrap_or_else(|_| "https://localhost:54321".to_string()),
            anon_key: std::env::var("SUPABASE_ANON_KEY")
                .expect("SUPABASE_ANON_KEY env variable must be set and not empty"),
            service_role_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
            database_password: std::env::var("SUPABASE_DB_PASSWORD").ok(),
            enable_rls: true,
            jwt_secret: std::env::var("SUPABASE_JWT_SECRET").ok(),
        }
    }
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Path to migration files
    pub migration_dir: String,
    /// Enable migration locking
    pub enable_locking: bool,
    /// Lock timeout duration
    pub lock_timeout: Duration,
    /// Enable transaction per migration
    pub transactional: bool,
    /// Migration table name
    pub table_name: String,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            migration_dir: "migrations".to_string(),
            enable_locking: true,
            lock_timeout: Duration::from_secs(60),
            transactional: true,
            table_name: "_sqlx_migrations".to_string(),
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// List of connection pools to initialize
    pub pools: Vec<ConnectionPool>,
    /// Global maximum number of connections allowed
    pub global_max_connections: u32,
    /// Failure threshold for circuit breaker
    pub circuit_breaker_failure_threshold: u32,
    /// Recovery timeout for circuit breaker
    pub circuit_breaker_recovery_timeout: Duration,
    /// Enable connection health monitoring
    pub enable_connection_monitoring: bool,
    /// Enable automatic failover
    pub enable_failover: bool,
}

/// Connection pool details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPool {
    pub name: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub health_check_interval: Duration,
    pub circuit_breaker_enabled: bool,
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_recovery_timeout: Duration,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub failover_enabled: bool,
    pub read_write_splitting: bool,
    pub endpoints: Vec<ConnectionEndpoint>,
}

/// Connection endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEndpoint {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub read_only: bool,
    pub weight: u32,
    pub priority: u32,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub command_timeout: Duration,
    pub retry_attempts: u32,
    pub health_check_interval: Duration,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRandom,
    LeastConnections,
    PriorityBased,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            pools: Vec::new(),
            global_max_connections: 100,
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_recovery_timeout: Duration::from_secs(30),
            enable_connection_monitoring: true,
            enable_failover: true,
        }
    }
}

/// Master configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseLayerConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Supabase configuration
    pub supabase: SupabaseConfig,
    /// Migration configuration
    pub migrations: MigrationConfig,
    /// Connection pool configuration
    pub connection_pool: ConnectionPoolConfig,
    /// Enable query logging
    pub enable_query_logging: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Environment (development, staging, production)
    pub environment: String,
}

impl Default for DatabaseLayerConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            supabase: SupabaseConfig::default(),
            migrations: MigrationConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
            enable_query_logging: false,
            enable_metrics: true,
            environment: "development".to_string(),
        }
    }
}

impl DatabaseLayerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let environment =
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let enable_query_logging = match environment.as_str() {
            "development" => true,
            "production" => false,
            _ => false,
        };

        Ok(Self {
            environment,
            enable_query_logging,
            ..Default::default()
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate database URL format
        if self.database.database_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }

        // Validate connection pool sizes
        if self.database.max_connections < self.database.min_connections {
            return Err(anyhow::anyhow!(
                "Max connections must be >= min connections"
            ));
        }

        // Validate Supabase configuration
        if self.supabase.project_url.is_empty() {
            return Err(anyhow::anyhow!("Supabase project URL cannot be empty"));
        }

        if self.supabase.anon_key.is_empty() {
            return Err(anyhow::anyhow!("Supabase anon key cannot be empty"));
        }

        // Validate cache configuration
        if self.cache.redis_url.is_empty() {
            return Err(anyhow::anyhow!("Redis URL cannot be empty"));
        }

        Ok(())
    }

    /// Get database connection string with SSL settings
    pub fn get_database_url(&self) -> &str {
        &self.database.database_url
    }

    /// Get Redis connection string
    pub fn get_redis_url(&self) -> &str {
        &self.cache.redis_url
    }

    /// Check if running in development environment
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    /// Check if running in production environment
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseLayerConfig::default();
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.cache.default_ttl.as_secs(), 3600);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = DatabaseLayerConfig::default();
        config.database.database_url = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_environment_detection() {
        let mut config = DatabaseLayerConfig::default();
        config.environment = "production".to_string();
        assert!(config.is_production());
        assert!(!config.is_development());
    }
}

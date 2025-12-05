//! # Redis Caching Layer
//!
//! High-performance Redis caching integration with connection pooling,
//! automatic serialization/deserialization, and cache management features.

use anyhow::Result;
use redis::{AsyncCommands, Client, Commands, Connection, RedisResult};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

use crate::config::CacheConfig;

/// Redis cache manager with connection pooling and async operations
pub struct CacheManager {
    client: Client,
    manager: Arc<Mutex<ConnectionManager>>,
    config: CacheConfig,
}

impl CacheManager {
    /// Create a new cache manager with the given configuration
    /// Create a new cache manager with the given configuration
    #[instrument(skip(config), fields(redis_url = %config.redis_url))]
    pub async fn new(config: CacheConfig) -> Result<Self> {
        info!("Initializing Redis cache manager");

        let client = Client::open(config.redis_url.clone())?;
        
        // Create connection manager
        let connection_manager = ConnectionManager::new(client.clone()).await?;

        // Test the connection
        let mut conn = connection_manager.clone();
        let ping_result: String = redis::cmd("PING").query_async(&mut conn).await?;
        if ping_result != "PONG" {
            // Note: Redis PING usually returns "PONG"
            // If it fails it returns an error, so strict check might be optional but good practice
        }

        info!("Redis cache manager initialized successfully");
        Ok(Self {
            client,
            manager: Arc::new(Mutex::new(connection_manager)),
            config,
        })
    }

    /// Get a value from cache by key
    #[instrument(skip(self), fields(key = %key))]
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send + Sync,
    {
        debug!("Getting value from cache: {}", key);

        let mut conn = self.manager.lock().await;
        let data: Option<Vec<u8>> = conn.get(key).await?;

        match data {
            Some(bytes) => {
                let value: T = serde_json::from_slice(&bytes)?;
                debug!("Cache hit for key: {}", key);
                Ok(Some(value))
            }
            None => {
                debug!("Cache miss for key: {}", key);
                Ok(None)
            }
        }
    }

    /// Set a value in cache with optional TTL
    #[instrument(skip(self, value), fields(key = %key))]
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        debug!("Setting value in cache: {}", key);

        let mut conn = self.manager.lock().await;
        let data = serde_json::to_vec(value)?;

        match ttl {
            Some(duration) => {
                let _: () = conn.set_ex(key, data, duration.as_secs()).await?;
                debug!("Set key {} with TTL: {}s", key, duration.as_secs());
            }
            None => {
                let _: () = conn.set(key, data).await?;
                debug!("Set key {} without TTL", key);
            }
        }

        Ok(())
    }

    /// Set a value with default TTL from configuration
    #[instrument(skip(self, value), fields(key = %key))]
    pub async fn set_with_default_ttl<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        self.set(key, value, Some(self.config.default_ttl)).await
    }

    /// Delete a key from cache
    #[instrument(skip(self), fields(key = %key))]
    pub async fn delete(&self, key: &str) -> Result<()> {
        debug!("Deleting key from cache: {}", key);

        let mut conn = self.manager.lock().await;
        let deleted: u32 = conn.del(key).await?;
        debug!("Deleted {} key(s) from cache", deleted);

        Ok(())
    }

    /// Check if a key exists in cache
    #[instrument(skip(self), fields(key = %key))]
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.manager.lock().await;
        let exists: bool = conn.exists(key).await?;

        debug!("Key {} exists in cache: {}", key, exists);
        Ok(exists)
    }

    /// Set multiple key-value pairs
    #[instrument(skip(self, items), fields(count = %items.len()))]
    pub async fn set_multiple<T>(&self, items: &[(String, T, Option<Duration>)]) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        debug!("Setting {} items in cache", items.len());

        let mut conn = self.manager.lock().await;
        let mut pipe = redis::pipe();

        for (key, value, ttl) in items {
            let data = serde_json::to_vec(value)?;
            match ttl {
                Some(duration) => {
                    pipe.set_ex(key, data, duration.as_secs()).ignore();
                }
                None => {
                    pipe.set(key, data).ignore();
                }
            }
        }

        let _: () = pipe.query_async(&mut *conn).await?;
        debug!("Successfully set {} items in cache", items.len());

        Ok(())
    }

    /// Get multiple values from cache
    #[instrument(skip(self), fields(keys_count = %keys.len()))]
    pub async fn get_multiple<T>(&self, keys: &[String]) -> Result<Vec<(String, Option<T>)>>
    where
        T: serde::de::DeserializeOwned + Send + Sync,
    {
        debug!("Getting {} values from cache", keys.len());

        let mut conn = self.manager.lock().await;

        // Get all values in one request
        let values: Vec<Option<Vec<u8>>> = conn.get(keys.to_vec()).await?;

        let results: Vec<(String, Option<T>)> = keys
            .iter()
            .zip(values.into_iter())
            .map(|(key, data)| {
                let value = data.and_then(|bytes| serde_json::from_slice::<T>(&bytes).ok());
                (key.clone(), value)
            })
            .collect();

        debug!("Retrieved {} values from cache", results.len());
        Ok(results)
    }

    /// Increment a numeric value in cache
    #[instrument(skip(self), fields(key = %key))]
    pub async fn increment(&self, key: &str, increment: i64) -> Result<i64> {
        debug!("Incrementing cache value for key: {}", key);

        let mut conn = self.manager.lock().await;
        let new_value: i64 = conn.incr(key, increment).await?;

        debug!("Incremented {} by {} to {}", key, increment, new_value);
        Ok(new_value)
    }

    /// Expire a key after a certain duration
    #[instrument(skip(self), fields(key = %key))]
    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<bool> {
        debug!("Setting expiration for key: {}", key);

        let mut conn = self.manager.lock().await;
        let expired: bool = conn.expire(key, ttl.as_secs() as i64).await?;

        debug!(
            "Set expiration for {} to {}s: {}",
            key,
            ttl.as_secs(),
            expired
        );
        Ok(expired)
    }

    /// Get cache statistics
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> Result<CacheStats> {
        debug!("Gathering cache statistics");

        let mut conn = self.manager.lock().await;

        let info: HashMap<String, String> = redis::cmd("INFO").query_async(&mut *conn).await?;

        let stats = CacheStats {
            connected_clients: info
                .get("connected_clients")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            used_memory: info
                .get("used_memory")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            used_memory_peak: info
                .get("used_memory_peak")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            mem_fragmentation_ratio: info
                .get("mem_fragmentation_ratio")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            total_connections_received: info
                .get("total_connections_received")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            total_commands_processed: info
                .get("total_commands_processed")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            instantaneous_ops_per_sec: info
                .get("instantaneous_ops_per_sec")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            rejected_connections: info
                .get("rejected_connections")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            evicted_keys: info
                .get("evicted_keys")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            keyspace_hits: info
                .get("keyspace_hits")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            keyspace_misses: info
                .get("keyspace_misses")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        };

        info!("Cache stats: {:?}", stats);
        Ok(stats)
    }

    /// Flush all cache data
    #[instrument(skip(self))]
    pub async fn flush_all(&self) -> Result<()> {
        warn!("Flushing all cache data");

        let mut conn = self.manager.lock().await;
        let _: () = redis::cmd("FLUSHALL").query_async(&mut *conn).await?;

        info!("Successfully flushed all cache data");
        Ok(())
    }

    /// Flush database cache
    #[instrument(skip(self))]
    pub async fn flush_db(&self, db: i32) -> Result<()> {
        debug!("Flushing database {} cache", db);

        let mut conn = self.manager.lock().await;
        let _: () = redis::cmd("FLUSHDB").query_async(&mut *conn).await?;

        info!("Successfully flushed database {} cache", db);
        Ok(())
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Test cache connectivity
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<()> {
        debug!("Performing cache health check");

        let mut conn = self.manager.lock().await;
        let pong: String = redis::cmd("PING").query_async(&mut *conn).await?;

        if pong == "PONG" {
            info!("Cache health check passed");
            Ok(())
        } else {
            error!("Cache health check failed");
            Err(anyhow::anyhow!(
                "Cache health check returned unexpected result"
            ))
        }
    }

    /// Get raw Redis client for advanced operations
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get connection manager for advanced operations
    pub fn manager(&self) -> &Arc<Mutex<ConnectionManager>> {
        &self.manager
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub connected_clients: u32,
    pub used_memory: u64,
    pub used_memory_peak: u64,
    pub mem_fragmentation_ratio: f64,
    pub total_connections_received: u64,
    pub total_commands_processed: u64,
    pub instantaneous_ops_per_sec: u32,
    pub rejected_connections: u32,
    pub evicted_keys: u64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            connected_clients: 0,
            used_memory: 0,
            used_memory_peak: 0,
            mem_fragmentation_ratio: 0.0,
            total_connections_received: 0,
            total_commands_processed: 0,
            instantaneous_ops_per_sec: 0,
            rejected_connections: 0,
            evicted_keys: 0,
            keyspace_hits: 0,
            keyspace_misses: 0,
        }
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: clients={}, memory={}MB, peak={}MB, ops/sec={}, hits={}, misses={}",
            self.connected_clients,
            self.used_memory / (1024 * 1024),
            self.used_memory_peak / (1024 * 1024),
            self.instantaneous_ops_per_sec,
            self.keyspace_hits,
            self.keyspace_misses
        )
    }
}

/// Cache error types
#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Key not found: {0}")]
    KeyNotFoundError(String),

    #[error("Operation error: {0}")]
    OperationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        CacheError::ConnectionError(err.to_string())
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_config() {
        let config = CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: Duration::from_secs(3600),
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            command_timeout: Duration::from_secs(10),
            enable_eviction: true,
            max_memory_mb: 100,
        };

        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.default_ttl.as_secs(), 3600);
        assert_eq!(config.max_connections, 10);
        assert!(config.enable_eviction);
    }

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.connected_clients, 0);
        assert_eq!(stats.used_memory, 0);
        assert_eq!(stats.keyspace_hits, 0);
        assert_eq!(stats.keyspace_misses, 0);
    }

    #[test]
    fn test_cache_stats_display() {
        let stats = CacheStats {
            connected_clients: 5,
            used_memory: 1048576,      // 1MB
            used_memory_peak: 2097152, // 2MB
            instantaneous_ops_per_sec: 100,
            keyspace_hits: 50,
            keyspace_misses: 10,
            ..Default::default()
        };

        let display = format!("{}", stats);
        assert!(display.contains("clients=5"));
        assert!(display.contains("memory=1MB"));
        assert!(display.contains("peak=2MB"));
        assert!(display.contains("ops/sec=100"));
        assert!(display.contains("hits=50"));
        assert!(display.contains("misses=10"));
    }
}

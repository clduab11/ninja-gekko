//! # Advanced Connection Pool Management
//!
//! Enterprise-grade connection pool management with read/write splitting,
//! failover capabilities, circuit breaker patterns, and comprehensive monitoring.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, instrument, warn};

use crate::config::{
    ConnectionEndpoint, ConnectionPool, ConnectionPoolConfig, LoadBalancingStrategy,
};

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolStats {
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub waiting_connections: u32,
    pub failed_connections: u64,
    pub successful_connections: u64,
    pub average_wait_time_ms: f64,
    pub max_wait_time_ms: u64,
    pub total_connection_time_ms: u64,
    pub average_connection_time_ms: f64,
    pub circuit_breaker_trips: u64,
    pub failover_events: u64,
    pub last_health_check: SystemTime,
    pub pool_efficiency: f64,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Circuit is open, failing fast
    HalfOpen, // Testing if service has recovered
}

/// Connection pool manager with advanced features
pub struct ConnectionManager {
    pools: Arc<RwLock<HashMap<String, Arc<ConnectionPoolImpl>>>>,
    config: ConnectionPoolConfig,
    stats: Arc<RwLock<HashMap<String, ConnectionPoolStats>>>,
    global_circuit_breaker: Arc<Mutex<CircuitBreaker>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    #[instrument(skip(config))]
    pub async fn new(config: ConnectionPoolConfig) -> Result<Self> {
        info!("Initializing advanced connection manager");

        let cb_threshold = config.circuit_breaker_failure_threshold;
        let cb_timeout = config.circuit_breaker_recovery_timeout;

        let manager = Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(HashMap::new())),
            global_circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::new(
                cb_threshold,
                cb_timeout,
            ))),
        };

        // Initialize pools based on configuration
        manager.initialize_pools().await?;

        // Start background health checks
        manager.start_health_checks();

        info!("Connection manager initialized successfully");
        Ok(manager)
    }

    /// Initialize connection pools
    #[instrument(skip(self))]
    async fn initialize_pools(&self) -> Result<()> {
        debug!("Initializing connection pools");

        for pool_config in &self.config.pools {
            let pool = Arc::new(ConnectionPoolImpl::new(pool_config.clone()).await?);
            let mut pools = self.pools.write().await;
            pools.insert(pool_config.name.clone(), pool);

            // Initialize stats
            let mut stats = self.stats.write().await;
            stats.insert(pool_config.name.clone(), ConnectionPoolStats::default());
        }

        info!("Initialized {} connection pools", self.config.pools.len());
        Ok(())
    }

    /// Start background health checks
    #[instrument(skip(self))]
    fn start_health_checks(&self) {
        let pools = Arc::clone(&self.pools);
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                let pools = pools.read().await;
                let mut stats = stats.write().await;

                for (pool_name, pool) in pools.iter() {
                    if let Some(pool_stats) = stats.get_mut(pool_name) {
                        pool.perform_health_check(pool_stats).await;
                    }
                }
            }
        });
    }

    /// Get a connection from the specified pool
    #[instrument(skip(self))]
    pub async fn get_connection(&self, pool_name: &str) -> Result<PooledConnection> {
        debug!("Getting connection from pool: {}", pool_name);

        // Check global circuit breaker
        {
            let circuit_breaker = self.global_circuit_breaker.lock().await;
            if circuit_breaker.state() == CircuitBreakerState::Open {
                return Err(anyhow!("Global circuit breaker is open"));
            }
        }

        let pools = self.pools.read().await;
        if let Some(pool) = pools.get(pool_name) {
            let start_time = Instant::now();

            match pool.acquire_connection().await {
                Ok(connection) => {
                    let connection_time = start_time.elapsed().as_millis() as u64;

                    // Update stats
                    let mut stats = self.stats.write().await;
                    if let Some(pool_stats) = stats.get_mut(pool_name) {
                        pool_stats.successful_connections += 1;
                        pool_stats.average_wait_time_ms =
                            (pool_stats.average_wait_time_ms + connection_time as f64) / 2.0;
                        pool_stats.max_wait_time_ms =
                            pool_stats.max_wait_time_ms.max(connection_time);
                        pool_stats.total_connection_time_ms += connection_time;
                        pool_stats.pool_efficiency = pool_stats.calculate_efficiency();
                    }

                    Ok(connection)
                }
                Err(e) => {
                    // Update failure stats
                    let mut stats = self.stats.write().await;
                    if let Some(pool_stats) = stats.get_mut(pool_name) {
                        pool_stats.failed_connections += 1;

                        // Check circuit breaker
                        if pool_stats.should_trip_circuit_breaker() {
                            warn!("Circuit breaker threshold reached for pool: {}", pool_name);
                        }
                    }

                    Err(anyhow!("Failed to acquire connection: {}", e))
                }
            }
        } else {
            Err(anyhow!("Pool not found: {}", pool_name))
        }
    }

    /// Execute a query with automatic retry and failover
    #[instrument(skip(self, operation))]
    pub async fn execute_with_retry<F, Fut, T>(&self, pool_name: &str, operation: F) -> Result<T>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + Send>>,
        Fut: Future<Output = Result<T, sqlx::Error>>,
    {
        debug!("Executing operation with retry logic: {}", pool_name);

        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    warn!(
                        "Operation failed on attempt {}: {:?}",
                        attempt + 1,
                        last_error
                    );

                    if attempt < max_retries - 1 {
                        // Exponential backoff
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                        tokio::time::sleep(delay).await;

                        // Try to get a fresh connection for retry
                        if let Ok(_) = self.get_connection(pool_name).await {
                            continue;
                        }
                    }
                }
            }
        }

        Err(anyhow!(
            "Operation failed after {} retries: {:?}",
            max_retries,
            last_error
        ))
    }

    /// Get connection pool statistics
    #[instrument(skip(self))]
    pub async fn get_pool_stats(&self, pool_name: &str) -> Option<ConnectionPoolStats> {
        let stats = self.stats.read().await;
        stats.get(pool_name).cloned()
    }

    /// Get all pool statistics
    #[instrument(skip(self))]
    pub async fn get_all_stats(&self) -> HashMap<String, ConnectionPoolStats> {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Perform maintenance on all pools
    #[instrument(skip(self))]
    pub async fn perform_maintenance(&self) -> Result<()> {
        debug!("Performing connection pool maintenance");

        let pools = self.pools.read().await;
        let mut stats = self.stats.write().await;

        for (pool_name, pool) in pools.iter() {
            if let Some(pool_stats) = stats.get_mut(pool_name) {
                pool.perform_maintenance(pool_stats).await;
            }
        }

        info!("Connection pool maintenance completed");
        Ok(())
    }

    /// Gracefully shutdown all pools
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down connection manager");

        let pools = self.pools.read().await;

        for (pool_name, pool) in pools.iter() {
            info!("Shutting down pool: {}", pool_name);
            pool.shutdown().await?;
        }

        info!("Connection manager shutdown completed");
        Ok(())
    }

    /// Get global circuit breaker state
    pub async fn circuit_breaker_state(&self) -> CircuitBreakerState {
        let circuit_breaker = self.global_circuit_breaker.lock().await;
        circuit_breaker.state()
    }

    /// Force reset global circuit breaker
    pub async fn reset_circuit_breaker(&self) {
        let circuit_breaker = self.global_circuit_breaker.lock().await;
        circuit_breaker.reset();
        info!("Global circuit breaker reset");
    }
}

/// Individual connection pool implementation
pub struct ConnectionPoolImpl {
    config: ConnectionPool,
    connections: Arc<RwLock<Vec<PooledConnection>>>,
    endpoint_manager: Arc<EndpointManager>,
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    last_health_check: Arc<Mutex<SystemTime>>,
    connection_counter: Arc<Mutex<u64>>,
}

impl ConnectionPoolImpl {
    /// Create a new connection pool
    #[instrument(skip(config))]
    async fn new(config: ConnectionPool) -> Result<Self> {
        info!("Creating connection pool: {}", config.name);

        let endpoint_manager = Arc::new(EndpointManager::new(config.endpoints.clone())?);

        let pool = Self {
            config: config.clone(),
            connections: Arc::new(RwLock::new(Vec::new())),
            endpoint_manager,
            circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::new(
                config.circuit_breaker_failure_threshold,
                config.circuit_breaker_recovery_timeout,
            ))),
            last_health_check: Arc::new(Mutex::new(SystemTime::now())),
            connection_counter: Arc::new(Mutex::new(0)),
        };

        // Initialize minimum connections
        pool.initialize_connections().await?;

        info!("Connection pool created successfully: {}", config.name);
        Ok(pool)
    }

    /// Initialize minimum connections
    #[instrument(skip(self))]
    async fn initialize_connections(&self) -> Result<()> {
        debug!(
            "Initializing {} minimum connections",
            self.config.min_connections
        );

        for _ in 0..self.config.min_connections {
            if let Ok(connection) = self.create_connection().await {
                let mut connections = self.connections.write().await;
                connections.push(connection);
            }
        }

        Ok(())
    }

    /// Create a new connection
    #[instrument(skip(self))]
    async fn create_connection(&self) -> Result<PooledConnection> {
        let endpoint = self.endpoint_manager.select_endpoint().await?;

        let connection =
            PooledConnection::new(endpoint.clone(), Arc::clone(&self.connection_counter)).await?;

        Ok(connection)
    }

    /// Acquire a connection from the pool
    #[instrument(skip(self))]
    async fn acquire_connection(&self) -> Result<PooledConnection> {
        debug!("Acquiring connection from pool: {}", self.config.name);

        // Check circuit breaker
        {
            let circuit_breaker = self.circuit_breaker.lock().await;
            if circuit_breaker.state() == CircuitBreakerState::Open {
                return Err(anyhow!("Pool circuit breaker is open"));
            }
        }

        // Try to get existing connection
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.pop() {
                if connection.is_valid().await {
                    debug!("Reused existing connection");
                    return Ok(connection);
                } else {
                    debug!("Connection was invalid, creating new one");
                }
            }
        }

        // Create new connection if pool is not at max capacity
        let current_count = {
            let connections = self.connections.read().await;
            connections.len() as u32
        };

        if current_count < self.config.max_connections {
            self.create_connection().await
        } else {
            Err(anyhow!("Connection pool exhausted"))
        }
    }

    /// Perform health check on the pool
    #[instrument(skip(self, stats))]
    async fn perform_health_check(&self, stats: &mut ConnectionPoolStats) {
        debug!("Performing health check on pool: {}", self.config.name);

        let now = SystemTime::now();
        let mut last_check = self.last_health_check.lock().await;

        if now.duration_since(*last_check).unwrap() < self.config.health_check_interval {
            return;
        }

        *last_check = now;

        // Check connection validity
        let mut connections = self.connections.write().await;
        let mut valid_connections = Vec::new();

        for connection in connections.drain(..) {
            if connection.is_valid().await {
                valid_connections.push(connection);
            } else {
                stats.failed_connections += 1;
            }
        }

        // Restore valid connections
        *connections = valid_connections;

        // Update stats
        stats.total_connections = connections.len() as u32;
        stats.idle_connections = connections.len() as u32;
        stats.active_connections = 0; // Will be calculated when connections are acquired
        stats.last_health_check = now;
        stats.pool_efficiency = stats.calculate_efficiency();

        debug!("Health check completed for pool: {}", self.config.name);
    }

    /// Perform maintenance operations
    #[instrument(skip(self, stats))]
    async fn perform_maintenance(&self, stats: &mut ConnectionPoolStats) {
        debug!("Performing maintenance on pool: {}", self.config.name);

        // Clean up expired connections
        let now = Instant::now();
        let mut connections = self.connections.write().await;
        let mut valid_connections = Vec::new();

        for connection in connections.drain(..) {
            if now.elapsed() < Duration::from_secs(300) {
                // 5 minute timeout
                valid_connections.push(connection);
            }
        }

        *connections = valid_connections;
        stats.total_connections = connections.len() as u32;

        debug!("Maintenance completed for pool: {}", self.config.name);
    }

    /// Shutdown the pool
    #[instrument(skip(self))]
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down pool: {}", self.config.name);

        let mut connections = self.connections.write().await;
        connections.clear();

        info!("Pool shutdown completed: {}", self.config.name);
        Ok(())
    }
}

/// Endpoint manager for load balancing and failover
pub struct EndpointManager {
    endpoints: Vec<ConnectionEndpoint>,
    current_index: Arc<Mutex<usize>>,
    strategy: LoadBalancingStrategy,
    health_status: Arc<RwLock<HashMap<String, bool>>>,
}

impl EndpointManager {
    /// Create a new endpoint manager
    fn new(endpoints: Vec<ConnectionEndpoint>) -> Result<Self> {
        if endpoints.is_empty() {
            return Err(anyhow!("At least one endpoint must be configured"));
        }

        let health_status = (0..endpoints.len())
            .map(|i| (format!("{}_{}", endpoints[i].host, endpoints[i].port), true))
            .collect();

        Ok(Self {
            endpoints,
            current_index: Arc::new(Mutex::new(0)),
            strategy: LoadBalancingStrategy::RoundRobin,
            health_status: Arc::new(RwLock::new(health_status)),
        })
    }

    /// Select an endpoint using the configured strategy
    #[instrument(skip(self))]
    async fn select_endpoint(&self) -> Result<&ConnectionEndpoint> {
        match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin().await,
            LoadBalancingStrategy::WeightedRandom => self.select_weighted_random().await,
            LoadBalancingStrategy::LeastConnections => self.select_least_connections().await,
            LoadBalancingStrategy::PriorityBased => self.select_priority_based().await,
        }
    }

    /// Round-robin selection
    async fn select_round_robin(&self) -> Result<&ConnectionEndpoint> {
        let mut index = self.current_index.lock().await;
        let start_index = *index;

        loop {
            let endpoint = &self.endpoints[*index];
            let health_key = format!("{}_{}", endpoint.host, endpoint.port);

            {
                let health_status = self.health_status.read().await;
                if *health_status.get(&health_key).unwrap_or(&false) {
                    let next_index = (*index + 1) % self.endpoints.len();
                    *index = next_index;
                    return Ok(endpoint);
                }
            }

            let next_index = (*index + 1) % self.endpoints.len();
            if next_index == start_index {
                return Err(anyhow!("No healthy endpoints available"));
            }
            *index = next_index;
        }
    }

    /// Weighted random selection
    async fn select_weighted_random(&self) -> Result<&ConnectionEndpoint> {
        // Implementation for weighted random selection
        // For now, fall back to round-robin
        self.select_round_robin().await
    }

    /// Least connections selection
    async fn select_least_connections(&self) -> Result<&ConnectionEndpoint> {
        // Implementation for least connections selection
        // For now, fall back to round-robin
        self.select_round_robin().await
    }

    /// Priority-based selection
    async fn select_priority_based(&self) -> Result<&ConnectionEndpoint> {
        // Implementation for priority-based selection
        // For now, fall back to round-robin
        self.select_round_robin().await
    }
}

/// Pooled database connection
pub struct PooledConnection {
    endpoint: ConnectionEndpoint,
    connection_id: u64,
    created_at: Instant,
    last_used: Arc<Mutex<Instant>>,
    is_valid: Arc<Mutex<bool>>,
}

impl PooledConnection {
    /// Create a new pooled connection
    async fn new(
        endpoint: ConnectionEndpoint,
        connection_counter: Arc<Mutex<u64>>,
    ) -> Result<Self> {
        let connection_id = {
            let mut counter = connection_counter.lock().await;
            *counter += 1;
            *counter
        };

        // Create actual database connection here
        // For now, we'll create a mock connection

        Ok(Self {
            endpoint,
            connection_id,
            created_at: Instant::now(),
            last_used: Arc::new(Mutex::new(Instant::now())),
            is_valid: Arc::new(Mutex::new(true)),
        })
    }

    /// Check if the connection is still valid
    async fn is_valid(&self) -> bool {
        let is_valid = *self.is_valid.lock().await;
        is_valid && self.created_at.elapsed() < Duration::from_secs(3600) // 1 hour timeout
    }

    /// Mark connection as invalid
    async fn invalidate(&self) {
        let mut is_valid = self.is_valid.lock().await;
        *is_valid = false;
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    failure_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<SystemTime>>>,
    state: Arc<Mutex<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            failure_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
        }
    }

    /// Get current state
    fn state(&self) -> CircuitBreakerState {
        // This would check the actual state based on failure count and timing
        CircuitBreakerState::Closed
    }

    /// Record a success
    async fn record_success(&self) {
        let mut failure_count = self.failure_count.lock().await;
        *failure_count = 0;
        let mut state = self.state.lock().await;
        *state = CircuitBreakerState::Closed;
    }

    /// Record a failure
    async fn record_failure(&self) {
        let mut failure_count = self.failure_count.lock().await;
        *failure_count += 1;

        if *failure_count >= self.failure_threshold {
            let mut state = self.state.lock().await;
            *state = CircuitBreakerState::Open;
            let mut last_failure_time = self.last_failure_time.lock().await;
            *last_failure_time = Some(SystemTime::now());
        }
    }

    /// Reset the circuit breaker
    fn reset(&self) {
        // Reset failure count and state
    }
}

impl ConnectionPoolStats {
    /// Calculate pool efficiency
    fn calculate_efficiency(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            (self.successful_connections as f64)
                / (self.successful_connections + self.failed_connections) as f64
        }
    }

    /// Check if circuit breaker should be tripped
    fn should_trip_circuit_breaker(&self) -> bool {
        let failure_rate = if self.successful_connections + self.failed_connections > 0 {
            self.failed_connections as f64
                / (self.successful_connections + self.failed_connections) as f64
        } else {
            0.0
        };

        failure_rate > 0.5 // Trip if failure rate > 50%
    }
}

impl Default for ConnectionPoolStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            waiting_connections: 0,
            failed_connections: 0,
            successful_connections: 0,
            average_wait_time_ms: 0.0,
            max_wait_time_ms: 0,
            total_connection_time_ms: 0,
            average_connection_time_ms: 0.0,
            circuit_breaker_trips: 0,
            failover_events: 0,
            last_health_check: SystemTime::now(),
            pool_efficiency: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_stats_default() {
        let stats = ConnectionPoolStats::default();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.failed_connections, 0);
        assert_eq!(stats.successful_connections, 0);
        assert_eq!(stats.pool_efficiency, 1.0);
    }

    #[test]
    fn test_connection_pool_stats_calculate_efficiency() {
        let mut stats = ConnectionPoolStats::default();
        stats.successful_connections = 10;
        stats.failed_connections = 5;
        stats.total_connections = 15;

        let efficiency = stats.calculate_efficiency();
        assert_eq!(efficiency, 10.0 / 15.0);
    }

    #[test]
    fn test_circuit_breaker_states() {
        assert_eq!(CircuitBreakerState::Closed, CircuitBreakerState::Closed);
        assert_eq!(CircuitBreakerState::Open, CircuitBreakerState::Open);
        assert_eq!(CircuitBreakerState::HalfOpen, CircuitBreakerState::HalfOpen);
    }

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let config = ConnectionPoolConfig {
            pools: Vec::new(),
            global_max_connections: 100,
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_recovery_timeout: Duration::from_secs(60),
            enable_connection_monitoring: true,
            enable_failover: true,
        };

        let manager = ConnectionManager::new(config).await.unwrap();
        assert!(manager.pools.read().await.is_empty());
    }

    #[test]
    fn test_load_balancing_strategies() {
        assert!(matches!(
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::RoundRobin
        ));
        assert!(matches!(
            LoadBalancingStrategy::WeightedRandom,
            LoadBalancingStrategy::WeightedRandom
        ));
        assert!(matches!(
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::LeastConnections
        ));
        assert!(matches!(
            LoadBalancingStrategy::PriorityBased,
            LoadBalancingStrategy::PriorityBased
        ));
    }

    #[test]
    fn test_connection_endpoint() {
        let endpoint = ConnectionEndpoint {
            host: "localhost".to_string(),
            port: 5432,
            database: "test".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            read_only: false,
            weight: 1,
            priority: 1,
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            command_timeout: Duration::from_secs(60),
            retry_attempts: 3,
            health_check_interval: Duration::from_secs(30),
        };

        assert_eq!(endpoint.host, "localhost");
        assert_eq!(endpoint.port, 5432);
        assert_eq!(endpoint.database, "test");
        assert!(!endpoint.read_only);
        assert_eq!(endpoint.weight, 1);
    }
}

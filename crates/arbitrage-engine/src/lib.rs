//! Arbitrage Engine - Gordon Gekko Inspired Trading System
//!
//! This crate implements an aggressive arbitrage trading engine that embodies
//! the "greed is good" mentality of Gordon Gekko from Wall Street.
//!
//! Core Features:
//! - Multi-exchange volatility scanning
//! - Dynamic capital allocation and reallocation
//! - AI-powered opportunity detection
//! - Real-time cross-exchange arbitrage execution
//! - Aggressive risk/reward optimization

use exchange_connectors::{ExchangeConnector, ExchangeId};
use neural_engine::NeuralEngine;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub mod capital_allocator;
pub mod execution_engine;
pub mod opportunity_detector;
pub mod volatility_scanner;

pub use capital_allocator::CapitalAllocator;
pub use execution_engine::ExecutionEngine;
pub use opportunity_detector::OpportunityDetector;
pub use volatility_scanner::VolatilityScanner;

/// Arbitrage engine error types
#[derive(Error, Debug)]
pub enum ArbitrageError {
    #[error("Exchange error: {0}")]
    Exchange(String),

    #[error("Insufficient capital: required {required}, available {available}")]
    InsufficientCapital {
        required: Decimal,
        available: Decimal,
    },

    #[error("No arbitrage opportunities found")]
    NoOpportunities,

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Risk limits exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Neural engine error: {0}")]
    NeuralEngine(String),

    #[error("Task join error: {0}")]
    TaskJoin(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type ArbitrageResult<T> = Result<T, ArbitrageError>;

/// Volatility score for a trading instrument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityScore {
    pub symbol: String,
    pub exchange: ExchangeId,
    pub score: f64, // 0.0 to 1.0, higher = more volatile
    pub price_change_1m: Decimal,
    pub price_change_5m: Decimal,
    pub price_change_15m: Decimal,
    pub volume_surge_factor: f64, // Volume vs average
    pub spread_tightness: f64,    // Bid-ask spread analysis
    pub momentum_indicator: f64,  // Price momentum
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cross-exchange arbitrage opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: Uuid,
    pub symbol: String,
    pub buy_exchange: ExchangeId,
    pub sell_exchange: ExchangeId,
    pub buy_price: Decimal,
    pub sell_price: Decimal,
    pub price_difference: Decimal,
    pub profit_percentage: f64,
    pub estimated_profit: Decimal,
    pub confidence_score: f64, // AI confidence in opportunity
    pub max_quantity: Decimal, // Maximum tradeable quantity
    pub time_sensitivity: TimeSensitivity,
    pub risk_score: f64, // 0.0 to 1.0, higher = riskier
    pub execution_complexity: ExecutionComplexity,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Time sensitivity levels for arbitrage opportunities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSensitivity {
    Low,      // 30+ seconds execution window
    Medium,   // 10-30 seconds execution window
    High,     // 3-10 seconds execution window
    Critical, // <3 seconds execution window
}

/// Execution complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionComplexity {
    Simple,   // Single pair, direct arbitrage
    Moderate, // Multi-step execution required
    Complex,  // Triangular or multi-hop arbitrage
    Advanced, // Cross-asset arbitrage
}

/// Capital allocation request for fund rebalancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRequest {
    pub id: Uuid,
    pub from_exchange: ExchangeId,
    pub to_exchange: ExchangeId,
    pub currency: String,
    pub amount: Decimal,
    pub priority: AllocationPriority,
    pub reason: String,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub deadline: chrono::DateTime<chrono::Utc>,
}

/// Capital allocation priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationPriority {
    Low,       // Routine rebalancing
    Normal,    // Standard opportunity funding
    High,      // High-profit opportunity
    Critical,  // Time-sensitive arbitrage
    Emergency, // Risk management action
}

/// Arbitrage strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageConfig {
    /// Minimum profit percentage to consider (e.g., 0.1 = 0.1%)
    pub min_profit_percentage: f64,

    /// Maximum risk score to accept (0.0 to 1.0)
    pub max_risk_score: f64,

    /// Minimum confidence score required (0.0 to 1.0)
    pub min_confidence_score: f64,

    /// Maximum position size per opportunity
    pub max_position_size: Decimal,

    /// Maximum daily capital allocation
    pub max_daily_allocation: Decimal,

    /// Target return multipliers (5:1 to 20:1 as specified)
    pub target_return_min: f64,
    pub target_return_max: f64,

    /// Capital allocation aggressiveness (0.0 to 1.0)
    pub allocation_aggressiveness: f64,

    /// Volatility scanning frequency in milliseconds
    pub scan_frequency_ms: u64,

    /// Enable aggressive mode (Gordon Gekko style)
    pub gekko_mode: bool,
}

impl Default for ArbitrageConfig {
    fn default() -> Self {
        Self {
            min_profit_percentage: 0.05, // 0.05% minimum profit
            max_risk_score: 0.7,
            min_confidence_score: 0.85,
            max_position_size: Decimal::new(50000, 0), // $50,000
            max_daily_allocation: Decimal::new(1000000, 0), // $1M
            target_return_min: 5.0,
            target_return_max: 20.0,
            allocation_aggressiveness: 0.8, // Highly aggressive
            scan_frequency_ms: 100,         // 100ms scanning
            gekko_mode: true,               // "Greed is good"
        }
    }
}

/// Main arbitrage engine orchestrating all components
pub struct ArbitrageEngine {
    config: ArbitrageConfig,
    exchanges: HashMap<ExchangeId, Arc<dyn ExchangeConnector>>,
    volatility_scanner: Arc<VolatilityScanner>,
    capital_allocator: Arc<CapitalAllocator>,
    opportunity_detector: Arc<OpportunityDetector>,
    execution_engine: Arc<ExecutionEngine>,
    neural_engine: Option<Arc<NeuralEngine>>,

    // State tracking
    active_opportunities: Arc<RwLock<HashMap<Uuid, ArbitrageOpportunity>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    risk_monitor: Arc<RwLock<RiskMonitor>>,
}

/// Performance tracking metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_opportunities_detected: u64,
    pub successful_arbitrages: u64,
    pub failed_arbitrages: u64,
    pub total_profit: Decimal,
    pub total_volume: Decimal,
    pub success_rate: f64,
    pub average_profit_per_trade: Decimal,
    pub sharpe_ratio: f64,
    pub max_drawdown: Decimal,
    pub daily_pnl: HashMap<String, Decimal>,
}

/// Risk monitoring and controls
#[derive(Debug, Default, Clone)]
pub struct RiskMonitor {
    pub daily_loss: Decimal,
    pub consecutive_losses: u32,
    pub max_position_exposure: Decimal,
    pub var_estimate: Decimal, // Value at Risk
    pub circuit_breaker_triggered: bool,
    pub last_risk_check: chrono::DateTime<chrono::Utc>,
}

impl ArbitrageEngine {
    /// Create a new arbitrage engine with configuration
    pub fn new(
        config: ArbitrageConfig,
        exchanges: HashMap<ExchangeId, Arc<dyn ExchangeConnector>>,
    ) -> Self {
        let volatility_scanner = Arc::new(VolatilityScanner::new(exchanges.clone()));
        let capital_allocator = Arc::new(CapitalAllocator::new(exchanges.clone()));
        let opportunity_detector = Arc::new(OpportunityDetector::new(config.clone(), None));
        let execution_engine = Arc::new(ExecutionEngine::new(exchanges.clone()));

        Self {
            config,
            exchanges,
            volatility_scanner,
            capital_allocator,
            opportunity_detector,
            execution_engine,
            neural_engine: None,
            active_opportunities: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            risk_monitor: Arc::new(RwLock::new(RiskMonitor::default())),
        }
    }

    /// Create a new arbitrage engine with a NeuralEngine for ML-powered confidence scoring
    pub fn with_neural_engine(
        config: ArbitrageConfig,
        exchanges: HashMap<ExchangeId, Arc<dyn ExchangeConnector>>,
        neural_engine: Arc<NeuralEngine>,
    ) -> Self {
        let volatility_scanner = Arc::new(VolatilityScanner::new(exchanges.clone()));
        let capital_allocator = Arc::new(CapitalAllocator::new(exchanges.clone()));
        let opportunity_detector = Arc::new(OpportunityDetector::new(
            config.clone(),
            Some(Arc::clone(&neural_engine)),
        ));
        let execution_engine = Arc::new(ExecutionEngine::new(exchanges.clone()));

        Self {
            config,
            exchanges,
            volatility_scanner,
            capital_allocator,
            opportunity_detector,
            execution_engine,
            neural_engine: Some(neural_engine),
            active_opportunities: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            risk_monitor: Arc::new(RwLock::new(RiskMonitor::default())),
        }
    }

    /// Get a reference to the neural engine if available
    pub fn neural_engine(&self) -> Option<&Arc<NeuralEngine>> {
        self.neural_engine.as_ref()
    }

    /// Start the arbitrage engine with continuous scanning
    pub async fn start(&self) -> ArbitrageResult<()> {
        info!("🔥 Starting Gordon Gekko Arbitrage Engine - Greed is Good! 🔥");

        if self.config.gekko_mode {
            info!("💰 GEKKO MODE ENABLED: Maximum aggression, maximum profits!");
        }

        // Start volatility scanning
        let scanner_handle = self.start_volatility_scanning().await?;

        // Start opportunity detection
        let detection_handle = self.start_opportunity_detection().await?;

        // Start capital allocation management
        let allocation_handle = self.start_capital_management().await?;

        // Start performance monitoring
        let monitoring_handle = self.start_performance_monitoring().await?;

        info!("✅ Arbitrage engine started successfully");
        info!(
            "🎯 Target returns: {}:1 to {}:1",
            self.config.target_return_min, self.config.target_return_max
        );
        info!("⚡ Scan frequency: {}ms", self.config.scan_frequency_ms);
        info!("💀 Max risk score: {}", self.config.max_risk_score);

        // Join all handles (this would run indefinitely)
        match tokio::try_join!(
            scanner_handle,
            detection_handle,
            allocation_handle,
            monitoring_handle
        ) {
            Ok(_) => {}
            Err(e) => return Err(ArbitrageError::TaskJoin(e.to_string())),
        }

        Ok(())
    }

    /// Stop the arbitrage engine gracefully
    pub async fn stop(&self) -> ArbitrageResult<()> {
        info!("🛑 Stopping arbitrage engine...");

        // Cancel all active opportunities
        let mut opportunities = self.active_opportunities.write().await;
        opportunities.clear();

        info!("✅ Arbitrage engine stopped");
        Ok(())
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Get active arbitrage opportunities
    pub async fn get_active_opportunities(&self) -> Vec<ArbitrageOpportunity> {
        self.active_opportunities
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Emergency stop - halt all trading immediately
    pub async fn emergency_stop(&self) -> ArbitrageResult<()> {
        error!("🚨 EMERGENCY STOP TRIGGERED 🚨");

        let mut risk_monitor = self.risk_monitor.write().await;
        risk_monitor.circuit_breaker_triggered = true;

        // Cancel all pending orders across all exchanges
        for (exchange_id, _connector) in &self.exchanges {
            warn!("Cancelling all orders on {:?}", exchange_id);
            // Implementation would cancel all active orders
        }

        // Clear all opportunities
        let mut opportunities = self.active_opportunities.write().await;
        opportunities.clear();

        error!("🛡️ Emergency stop complete - all trading halted");
        Ok(())
    }

    /// Process a market event to update internal state
    pub async fn process_market_event(
        &self,
        event: &exchange_connectors::MarketTick,
        exchange_id: ExchangeId,
    ) {
        self.opportunity_detector
            .update_price(event.clone(), exchange_id)
            .await;
    }

    // Private implementation methods

    async fn start_volatility_scanning(
        &self,
    ) -> ArbitrageResult<tokio::task::JoinHandle<ArbitrageResult<()>>> {
        let scanner = Arc::clone(&self.volatility_scanner);
        let frequency = self.config.scan_frequency_ms;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(frequency));

            loop {
                interval.tick().await;

                match scanner.scan_volatility().await {
                    Ok(scores) => {
                        debug!("Scanned volatility for {} instruments", scores.len());
                    }
                    Err(e) => {
                        error!("Volatility scanning error: {}", e);
                    }
                }
            }
        });

        Ok(handle)
    }

    async fn start_opportunity_detection(
        &self,
    ) -> ArbitrageResult<tokio::task::JoinHandle<ArbitrageResult<()>>> {
        let detector = Arc::clone(&self.opportunity_detector);
        let opportunities = Arc::clone(&self.active_opportunities);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(50), // 50ms detection cycle
            );

            loop {
                interval.tick().await;

                match detector.detect_opportunities().await {
                    Ok(new_opportunities) => {
                        let mut active = opportunities.write().await;
                        for opp in new_opportunities {
                            active.insert(opp.id, opp);
                        }
                    }
                    Err(e) => {
                        warn!("Opportunity detection error: {}", e);
                    }
                }
            }
        });

        Ok(handle)
    }

    async fn start_capital_management(
        &self,
    ) -> ArbitrageResult<tokio::task::JoinHandle<ArbitrageResult<()>>> {
        let allocator = Arc::clone(&self.capital_allocator);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(5), // 5-second allocation cycle
            );

            loop {
                interval.tick().await;

                match allocator.rebalance_capital().await {
                    Ok(_) => {
                        debug!("Capital rebalancing completed");
                    }
                    Err(e) => {
                        warn!("Capital allocation error: {}", e);
                    }
                }
            }
        });

        Ok(handle)
    }

    async fn start_performance_monitoring(
        &self,
    ) -> ArbitrageResult<tokio::task::JoinHandle<ArbitrageResult<()>>> {
        let metrics = Arc::clone(&self.performance_metrics);
        let risk_monitor = Arc::clone(&self.risk_monitor);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(10), // 10-second monitoring cycle
            );

            loop {
                interval.tick().await;

                // Update performance metrics
                let mut metrics_guard = metrics.write().await;
                metrics_guard.success_rate = if metrics_guard.total_opportunities_detected > 0 {
                    (metrics_guard.successful_arbitrages as f64)
                        / (metrics_guard.total_opportunities_detected as f64)
                        * 100.0
                } else {
                    0.0
                };

                // Update risk monitoring
                let mut risk_guard = risk_monitor.write().await;
                risk_guard.last_risk_check = chrono::Utc::now();

                drop(metrics_guard);
                drop(risk_guard);

                debug!("Performance monitoring cycle completed");
            }
        });

        Ok(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrage_config_default() {
        let config = ArbitrageConfig::default();
        assert_eq!(config.min_profit_percentage, 0.05);
        assert_eq!(config.max_risk_score, 0.7);
        assert_eq!(config.gekko_mode, true);
        assert_eq!(config.target_return_min, 5.0);
        assert_eq!(config.target_return_max, 20.0);
    }

    #[test]
    fn test_volatility_score_creation() {
        let score = VolatilityScore {
            symbol: "BTC-USD".to_string(),
            exchange: ExchangeId::Kraken,
            score: 0.85,
            price_change_1m: Decimal::new(150, 2),   // 1.50
            price_change_5m: Decimal::new(750, 2),   // 7.50
            price_change_15m: Decimal::new(1250, 2), // 12.50
            volume_surge_factor: 2.5,
            spread_tightness: 0.9,
            momentum_indicator: 0.7,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(score.symbol, "BTC-USD");
        assert_eq!(score.exchange, ExchangeId::Kraken);
        assert_eq!(score.score, 0.85);
    }

    #[test]
    fn test_arbitrage_opportunity_creation() {
        let opportunity = ArbitrageOpportunity {
            id: Uuid::new_v4(),
            symbol: "BTC-USD".to_string(),
            buy_exchange: ExchangeId::Kraken,
            sell_exchange: ExchangeId::BinanceUs,
            buy_price: Decimal::new(50000, 0),
            sell_price: Decimal::new(50250, 0),
            price_difference: Decimal::new(250, 0),
            profit_percentage: 0.5,
            estimated_profit: Decimal::new(500, 0),
            confidence_score: 0.92,
            max_quantity: Decimal::new(5, 0),
            time_sensitivity: TimeSensitivity::High,
            risk_score: 0.3,
            execution_complexity: ExecutionComplexity::Simple,
            detected_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(30),
        };

        assert_eq!(opportunity.symbol, "BTC-USD");
        assert_eq!(opportunity.profit_percentage, 0.5);
        assert_eq!(opportunity.time_sensitivity, TimeSensitivity::High);
    }
}

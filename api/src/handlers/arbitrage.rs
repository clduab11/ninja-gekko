//! Arbitrage API handlers for Gordon Gekko trading system
//!
//! This module provides HTTP endpoints for arbitrage operations including
//! opportunity management, balance queries, volatility tracking, and performance metrics.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    error::ApiResult,
    models::ApiResponse,
    AppState,
};

// Re-export types from arbitrage engine
pub use arbitrage_engine::{
    ArbitrageOpportunity, VolatilityScore, ArbitrageConfig, 
    AllocationRequest, AllocationPriority, PerformanceMetrics
};
pub use exchange_connectors::{ExchangeId, TransferUrgency};

/// Request to start arbitrage strategy
#[derive(Debug, Deserialize)]
pub struct StartArbitrageRequest {
    pub strategy_name: String,
    pub config: ArbitrageConfig,
    pub exchanges: Vec<ExchangeId>,
    pub symbols: Vec<String>,
}

/// Request to stop arbitrage strategy
#[derive(Debug, Deserialize)]
pub struct StopArbitrageRequest {
    pub strategy_name: String,
    pub reason: String,
}

/// Query parameters for opportunities
#[derive(Debug, Deserialize)]
pub struct OpportunityQuery {
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub min_profit_percentage: Option<f64>,
    pub min_confidence: Option<f64>,
    pub limit: Option<usize>,
}

/// Query parameters for volatility scores
#[derive(Debug, Deserialize)]
pub struct VolatilityQuery {
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub min_score: Option<f64>,
    pub limit: Option<usize>,
}

/// Response for arbitrage strategy status
#[derive(Debug, Serialize)]
pub struct ArbitrageStrategyStatus {
    pub strategy_name: String,
    pub is_active: bool,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub opportunities_detected: u64,
    pub successful_trades: u64,
    pub total_profit: rust_decimal::Decimal,
    pub success_rate: f64,
    pub current_config: ArbitrageConfig,
}

/// Start an arbitrage strategy
pub async fn start_arbitrage_strategy(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StartArbitrageRequest>,
) -> ApiResult<Json<ApiResponse<ArbitrageStrategyStatus>>> {
    info!("🚀 Starting arbitrage strategy: {}", request.strategy_name);
    info!("Gekko Mode: {}, Aggression: {:.0}%", 
          request.config.gekko_mode, 
          request.config.allocation_aggressiveness * 100.0);

    // In a real implementation, this would:
    // 1. Validate the configuration
    // 2. Initialize the arbitrage engine with the config
    // 3. Start the strategy
    // 4. Store the strategy state

    let status = ArbitrageStrategyStatus {
        strategy_name: request.strategy_name,
        is_active: true,
        started_at: Some(chrono::Utc::now()),
        opportunities_detected: 0,
        successful_trades: 0,
        total_profit: rust_decimal::Decimal::ZERO,
        success_rate: 0.0,
        current_config: request.config,
    };

    info!("✅ Arbitrage strategy started successfully");
    Ok(Json(ApiResponse::success(status)))
}

/// Stop an arbitrage strategy
pub async fn stop_arbitrage_strategy(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StopArbitrageRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    info!("🛑 Stopping arbitrage strategy: {} ({})", 
          request.strategy_name, request.reason);

    // In a real implementation, this would:
    // 1. Find the active strategy
    // 2. Gracefully shut down the arbitrage engine
    // 3. Cancel pending orders
    // 4. Update strategy state

    let message = format!("Strategy '{}' stopped successfully", request.strategy_name);
    info!("✅ {}", message);
    
    Ok(Json(ApiResponse::success(message)))
}

/// Get current arbitrage opportunities
pub async fn get_arbitrage_opportunities(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OpportunityQuery>,
) -> ApiResult<Json<ApiResponse<Vec<ArbitrageOpportunity>>>> {
    info!("📊 Fetching arbitrage opportunities");

    // Simulate arbitrage opportunities
    let opportunities = generate_mock_opportunities(&query);

    info!("Found {} arbitrage opportunities", opportunities.len());
    Ok(Json(ApiResponse::success(opportunities)))
}

/// Get volatility scores for instruments
pub async fn get_volatility_scores(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VolatilityQuery>,
) -> ApiResult<Json<ApiResponse<Vec<VolatilityScore>>>> {
    info!("📈 Fetching volatility scores");

    // Simulate volatility scores
    let scores = generate_mock_volatility_scores(&query);

    info!("Found {} volatility scores", scores.len());
    Ok(Json(ApiResponse::success(scores)))
}

/// Get arbitrage performance metrics
pub async fn get_arbitrage_performance(
    State(state): State<Arc<AppState>>,
    Path(strategy_name): Path<String>,
) -> ApiResult<Json<ApiResponse<PerformanceMetrics>>> {
    info!("📊 Fetching performance metrics for strategy: {}", strategy_name);

    // Simulate performance metrics
    let metrics = PerformanceMetrics {
        total_opportunities_detected: 1247,
        successful_arbitrages: 1156,
        failed_arbitrages: 91,
        total_profit: rust_decimal::Decimal::new(487650, 2), // $4,876.50
        total_volume: rust_decimal::Decimal::new(12450000, 2), // $124,500
        success_rate: 92.7,
        average_profit_per_trade: rust_decimal::Decimal::new(422, 2), // $4.22
        sharpe_ratio: 2.84,
        max_drawdown: rust_decimal::Decimal::new(125, 2), // $1.25
        daily_pnl: std::collections::HashMap::new(),
    };

    info!("📈 Performance: {:.1}% success rate, ${} total profit", 
          metrics.success_rate, metrics.total_profit);

    Ok(Json(ApiResponse::success(metrics)))
}

/// Get real-time balance distribution across exchanges
pub async fn get_balance_distribution(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    info!("💰 Fetching balance distribution across exchanges");

    // Simulate balance distribution
    let distribution = json!({
        "total_portfolio_value": 1250000.00,
        "exchanges": {
            "kraken": {
                "USD": {"available": 425000.00, "reserved": 25000.00},
                "BTC": {"available": 8.5, "reserved": 0.5, "usd_value": 510000.00},
                "ETH": {"available": 125.0, "reserved": 5.0, "usd_value": 312500.00}
            },
            "binance_us": {
                "USD": {"available": 375000.00, "reserved": 15000.00},
                "BTC": {"available": 6.2, "reserved": 0.3, "usd_value": 372000.00},
                "ETH": {"available": 90.0, "reserved": 2.0, "usd_value": 225000.00}
            },
            "oanda": {
                "USD": {"available": 150000.00, "reserved": 5000.00},
                "EUR": {"available": 45000.00, "usd_value": 48600.00},
                "GBP": {"available": 35000.00, "usd_value": 43750.00}
            }
        },
        "allocation_strategy": "aggressive",
        "last_rebalanced": chrono::Utc::now()
    });

    Ok(Json(ApiResponse::success(distribution)))
}

/// Execute emergency capital reallocation (Gekko mode)
pub async fn emergency_capital_reallocation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmergencyReallocationRequest>,
) -> ApiResult<Json<ApiResponse<EmergencyReallocationResponse>>> {
    warn!("🚨 EMERGENCY CAPITAL REALLOCATION TRIGGERED 🚨");
    warn!("Target: {:?}, Currency: {}, Percentage: {}%", 
          request.target_exchange, request.currency, request.percentage * 100.0);

    // In a real implementation, this would:
    // 1. Validate the reallocation request
    // 2. Check available balances
    // 3. Execute transfers with emergency priority
    // 4. Monitor transfer progress

    let response = EmergencyReallocationResponse {
        reallocation_id: Uuid::new_v4(),
        initiated_at: chrono::Utc::now(),
        estimated_completion: chrono::Utc::now() + chrono::Duration::minutes(15),
        transfers_initiated: 3,
        total_amount_reallocated: rust_decimal::Decimal::new(500000, 0), // $500,000
        status: "processing".to_string(),
    };

    warn!("💀 Emergency reallocation initiated: {}", response.reallocation_id);
    Ok(Json(ApiResponse::success(response)))
}

/// Request types for emergency reallocation
#[derive(Debug, Deserialize)]
pub struct EmergencyReallocationRequest {
    pub target_exchange: ExchangeId,
    pub currency: String,
    pub percentage: f64, // 0.0 to 1.0
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct EmergencyReallocationResponse {
    pub reallocation_id: Uuid,
    pub initiated_at: chrono::DateTime<chrono::Utc>,
    pub estimated_completion: chrono::DateTime<chrono::Utc>,
    pub transfers_initiated: u32,
    pub total_amount_reallocated: rust_decimal::Decimal,
    pub status: String,
}

/// Emergency shutdown request
#[derive(Debug, Deserialize)]
pub struct EmergencyShutdownRequest {
    pub reason: String,
    pub cancel_orders: bool,
    pub save_state: bool,
}

/// Emergency shutdown response
#[derive(Debug, Serialize)]
pub struct EmergencyShutdownResponse {
    pub shutdown_id: Uuid,
    pub initiated_at: chrono::DateTime<chrono::Utc>,
    pub orders_cancelled: u32,
    pub positions_closed: u32,
    pub state_saved: bool,
    pub status: String,
}

/// Risk status response
#[derive(Debug, Serialize)]
pub struct RiskStatusResponse {
    pub circuit_breaker_triggered: bool,
    pub daily_loss: rust_decimal::Decimal,
    pub max_daily_loss: rust_decimal::Decimal,
    pub consecutive_losses: u32,
    pub max_consecutive_losses: u32,
    pub current_drawdown_percent: f64,
    pub max_drawdown_percent: f64,
    pub api_error_count: u32,
    pub last_exchange_heartbeat: chrono::DateTime<chrono::Utc>,
    pub risk_score: f64,
    pub trading_halted: bool,
    pub halt_reason: Option<String>,
}

/// Execute emergency shutdown - halt all trading immediately
pub async fn emergency_shutdown(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<EmergencyShutdownRequest>,
) -> ApiResult<Json<ApiResponse<EmergencyShutdownResponse>>> {
    warn!("🚨🚨🚨 EMERGENCY SHUTDOWN INITIATED 🚨🚨🚨");
    warn!("Reason: {}", request.reason);
    warn!("Cancel Orders: {}, Save State: {}", request.cancel_orders, request.save_state);

    // In a real implementation, this would:
    // 1. Trigger circuit breaker across all exchanges
    // 2. Cancel all pending orders
    // 3. Close all open positions (optional)
    // 4. Save current state to database
    // 5. Send critical alerts to all configured channels
    // 6. Disconnect from exchanges
    // 7. Log all actions for audit trail

    let response = EmergencyShutdownResponse {
        shutdown_id: Uuid::new_v4(),
        initiated_at: chrono::Utc::now(),
        orders_cancelled: 15, // Mock value
        positions_closed: 3,  // Mock value
        state_saved: request.save_state,
        status: "shutdown_complete".to_string(),
    };

    warn!("🛡️ Emergency shutdown complete: {}", response.shutdown_id);
    warn!("Orders cancelled: {}, Positions closed: {}", 
          response.orders_cancelled, response.positions_closed);

    Ok(Json(ApiResponse::success(response)))
}

/// Get current risk status and circuit breaker state
pub async fn get_risk_status(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<RiskStatusResponse>>> {
    info!("📊 Fetching risk status");

    // In a real implementation, this would query the RiskMonitor
    let status = RiskStatusResponse {
        circuit_breaker_triggered: false,
        daily_loss: rust_decimal::Decimal::new(125050, 2), // $1,250.50
        max_daily_loss: rust_decimal::Decimal::new(500000, 2), // $5,000
        consecutive_losses: 1,
        max_consecutive_losses: 5,
        current_drawdown_percent: 0.85,
        max_drawdown_percent: 2.0,
        api_error_count: 2,
        last_exchange_heartbeat: chrono::Utc::now() - chrono::Duration::seconds(5),
        risk_score: 0.25,
        trading_halted: false,
        halt_reason: None,
    };

    Ok(Json(ApiResponse::success(status)))
}

/// Manually trigger circuit breaker
#[derive(Debug, Deserialize)]
pub struct CircuitBreakerRequest {
    pub reason: String,
    pub duration_minutes: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CircuitBreakerResponse {
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub resume_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reason: String,
    pub status: String,
}

pub async fn trigger_circuit_breaker(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CircuitBreakerRequest>,
) -> ApiResult<Json<ApiResponse<CircuitBreakerResponse>>> {
    warn!("⚡ Circuit breaker manually triggered: {}", request.reason);

    let resume_at = request.duration_minutes.map(|mins| {
        chrono::Utc::now() + chrono::Duration::minutes(mins as i64)
    });

    let response = CircuitBreakerResponse {
        triggered_at: chrono::Utc::now(),
        resume_at,
        reason: request.reason,
        status: "triggered".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Reset circuit breaker and resume trading
pub async fn reset_circuit_breaker(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<String>>> {
    info!("✅ Circuit breaker reset requested");

    // In a real implementation, verify conditions are safe before resetting
    
    Ok(Json(ApiResponse::success("Circuit breaker reset. Trading resumed.".to_string())))
}

// Constants for mock data generation
const DEFAULT_MOCK_SYMBOL: &str = "BTC-USD";
const MOCK_SYMBOLS: [&str; 5] = ["BTC-USD", "ETH-USD", "ADA-USD", "SOL-USD", "AVAX-USD"];

// Helper functions for generating mock data

fn generate_mock_opportunities(query: &OpportunityQuery) -> Vec<ArbitrageOpportunity> {
    let limit = query.limit.unwrap_or(10);
    let symbol = query.symbol.as_deref().unwrap_or(DEFAULT_MOCK_SYMBOL);
    
    (0..limit).map(|i| {
        ArbitrageOpportunity {
            id: Uuid::new_v4(),
            symbol: symbol.to_string(),
            buy_exchange: ExchangeId::Kraken,
            sell_exchange: ExchangeId::BinanceUs,
            buy_price: rust_decimal::Decimal::new(49850 + i as i64 * 10, 0),
            sell_price: rust_decimal::Decimal::new(50125 + i as i64 * 12, 0),
            price_difference: rust_decimal::Decimal::new(275 + i as i64 * 2, 0),
            profit_percentage: 0.55 + (i as f64 * 0.01),
            estimated_profit: rust_decimal::Decimal::new(550 + i as i64 * 10, 0),
            confidence_score: 0.92 - (i as f64 * 0.01),
            max_quantity: rust_decimal::Decimal::new(5 + i as i64, 0),
            time_sensitivity: arbitrage_engine::TimeSensitivity::High,
            risk_score: 0.15 + (i as f64 * 0.02),
            execution_complexity: arbitrage_engine::ExecutionComplexity::Simple,
            detected_at: chrono::Utc::now() - chrono::Duration::seconds(i as i64 * 10),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(30 - i as i64 * 2),
        }
    }).collect()
}

fn generate_mock_volatility_scores(query: &VolatilityQuery) -> Vec<VolatilityScore> {
    let limit = query.limit.unwrap_or(20);
    let exchanges = [ExchangeId::Kraken, ExchangeId::BinanceUs];
    
    (0..limit).map(|i| {
        let symbol = query.symbol.as_deref()
            .unwrap_or(MOCK_SYMBOLS[i % MOCK_SYMBOLS.len()]);
        let exchange = exchanges[i % exchanges.len()];
        
        VolatilityScore {
            symbol: symbol.to_string(),
            exchange,
            score: 0.95 - (i as f64 * 0.02),
            price_change_1m: rust_decimal::Decimal::new(150 - i as i64 * 5, 2),
            price_change_5m: rust_decimal::Decimal::new(750 - i as i64 * 20, 2),
            price_change_15m: rust_decimal::Decimal::new(1250 - i as i64 * 35, 2),
            volume_surge_factor: 2.5 - (i as f64 * 0.1),
            spread_tightness: 0.9 - (i as f64 * 0.02),
            momentum_indicator: 0.75 - (i as f64 * 0.015),
            timestamp: chrono::Utc::now() - chrono::Duration::seconds(i as i64 * 60),
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_opportunities() {
        let query = OpportunityQuery {
            exchange: None,
            symbol: None,
            min_profit_percentage: None,
            min_confidence: None,
            limit: Some(5),
        };
        
        let opportunities = generate_mock_opportunities(&query);
        assert_eq!(opportunities.len(), 5);
        assert!(opportunities.iter().all(|o| o.confidence_score > 0.8));
    }

    #[test]
    fn test_generate_mock_volatility_scores() {
        let query = VolatilityQuery {
            exchange: None,
            symbol: None,
            min_score: None,
            limit: Some(10),
        };
        
        let scores = generate_mock_volatility_scores(&query);
        assert_eq!(scores.len(), 10);
        assert!(scores.iter().all(|s| s.score > 0.5));
    }
}
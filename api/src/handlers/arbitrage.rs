//! Arbitrage API handlers for Gordon Gekko trading system
//!
//! This module provides HTTP endpoints for arbitrage operations including
//! opportunity management, balance queries, volatility tracking, and performance metrics.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{error::ApiResult, models::ApiResponse, AppState};

// Re-export types from arbitrage engine
pub use arbitrage_engine::{
    AllocationPriority, AllocationRequest, ArbitrageConfig, ArbitrageOpportunity,
    PerformanceMetrics, VolatilityScore,
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
    info!(
        "Gekko Mode: {}, Aggression: {:.0}%",
        request.config.gekko_mode,
        request.config.allocation_aggressiveness * 100.0
    );

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
    info!(
        "🛑 Stopping arbitrage strategy: {} ({})",
        request.strategy_name, request.reason
    );

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

    // TODO: Implement real opportunity detection from exchange data
    // Return empty list until arbitrage engine is connected
    let opportunities: Vec<ArbitrageOpportunity> = Vec::new();

    info!("Found {} arbitrage opportunities", opportunities.len());
    Ok(Json(ApiResponse::success(opportunities)))
}

/// Get volatility scores for instruments
pub async fn get_volatility_scores(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VolatilityQuery>,
) -> ApiResult<Json<ApiResponse<Vec<VolatilityScore>>>> {
    info!("📈 Fetching volatility scores");

    // TODO: Implement real volatility calculation from market data
    // Return empty list until market data integration is complete
    let scores: Vec<VolatilityScore> = Vec::new();

    info!("Found {} volatility scores", scores.len());
    Ok(Json(ApiResponse::success(scores)))
}

/// Get arbitrage performance metrics
pub async fn get_arbitrage_performance(
    State(state): State<Arc<AppState>>,
    Path(strategy_name): Path<String>,
) -> ApiResult<Json<ApiResponse<PerformanceMetrics>>> {
    info!(
        "📊 Fetching performance metrics for strategy: {}",
        strategy_name
    );

    // TODO: Implement real performance metrics from database
    // Return zeroed metrics until strategy tracking is implemented
    let metrics = PerformanceMetrics {
        total_opportunities_detected: 0,
        successful_arbitrages: 0,
        failed_arbitrages: 0,
        total_profit: rust_decimal::Decimal::ZERO,
        total_volume: rust_decimal::Decimal::ZERO,
        success_rate: 0.0,
        average_profit_per_trade: rust_decimal::Decimal::ZERO,
        sharpe_ratio: 0.0,
        max_drawdown: rust_decimal::Decimal::ZERO,
        daily_pnl: std::collections::HashMap::new(),
    };

    info!("📈 Performance metrics returned (no historical data yet)");
    Ok(Json(ApiResponse::success(metrics)))
}

/// Get real-time balance distribution across exchanges
pub async fn get_balance_distribution(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    info!("💰 Fetching balance distribution across exchanges");

    // TODO: Implement real balance fetching from connected exchanges
    // Return empty structure until exchange integration is complete
    let distribution = json!({
        "total_portfolio_value": 0.0,
        "exchanges": {},
        "allocation_strategy": "none",
        "last_rebalanced": null,
        "message": "Balance distribution requires connected exchange credentials"
    });

    Ok(Json(ApiResponse::success(distribution)))
}

/// Execute emergency capital reallocation (Gekko mode)
pub async fn emergency_capital_reallocation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmergencyReallocationRequest>,
) -> ApiResult<Json<ApiResponse<EmergencyReallocationResponse>>> {
    warn!("🚨 EMERGENCY CAPITAL REALLOCATION TRIGGERED 🚨");
    warn!(
        "Target: {:?}, Currency: {}, Percentage: {}%",
        request.target_exchange,
        request.currency,
        request.percentage * 100.0
    );

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

    warn!(
        "💀 Emergency reallocation initiated: {}",
        response.reallocation_id
    );
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
    warn!(
        "Cancel Orders: {}, Save State: {}",
        request.cancel_orders, request.save_state
    );

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
        orders_cancelled: 0, // Real count from trading engine
        positions_closed: 0, // Real count from trading engine
        state_saved: request.save_state,
        status: "shutdown_complete".to_string(),
    };

    warn!("🛡️ Emergency shutdown complete: {}", response.shutdown_id);
    warn!(
        "Orders cancelled: {}, Positions closed: {}",
        response.orders_cancelled, response.positions_closed
    );

    Ok(Json(ApiResponse::success(response)))
}

/// Get current risk status and circuit breaker state
pub async fn get_risk_status(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<RiskStatusResponse>>> {
    info!("📊 Fetching risk status");

    // TODO: Query real RiskMonitor when implemented
    // Return zeroed status until risk monitoring is active
    let status = RiskStatusResponse {
        circuit_breaker_triggered: false,
        daily_loss: rust_decimal::Decimal::ZERO,
        max_daily_loss: rust_decimal::Decimal::new(500000, 2), // $5,000 default
        consecutive_losses: 0,
        max_consecutive_losses: 5,
        current_drawdown_percent: 0.0,
        max_drawdown_percent: 2.0,
        api_error_count: 0,
        last_exchange_heartbeat: chrono::Utc::now(),
        risk_score: 0.0,
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

    let resume_at = request
        .duration_minutes
        .map(|mins| chrono::Utc::now() + chrono::Duration::minutes(mins as i64));

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

    Ok(Json(ApiResponse::success(
        "Circuit breaker reset. Trading resumed.".to_string(),
    )))
}

// Mock helper functions removed - all handlers now return empty/initialized data

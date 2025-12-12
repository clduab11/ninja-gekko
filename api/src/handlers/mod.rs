//! HTTP request handlers for the Ninja Gekko API
//!
//! This module contains all the HTTP endpoint handlers organized by functionality:
//! - Authentication utilities (login, logout, token refresh)
//! - Trade management (CRUD operations for trades)
//! - Portfolio management (portfolio, positions, performance)
//! - Market data retrieval (current data, historical data)
//! - Strategy management (trading strategies, execution, backtesting)
//! - Utility endpoints (health check, API info)

use crate::models::ApiResponse;
use axum::response::Json;
use serde_json::json;

pub mod accounts;
pub mod arbitrage;
pub mod auth_utils;
pub mod chat;
pub mod intel;
pub mod intel_rss;
pub mod market_data;
pub mod orchestrator;
pub mod portfolio;
pub mod strategies;
pub mod trades;

// Re-export all handler functions
pub use arbitrage::{
    emergency_capital_reallocation, emergency_shutdown, get_arbitrage_opportunities,
    get_arbitrage_performance, get_balance_distribution, get_risk_status, get_volatility_scores,
    reset_circuit_breaker, start_arbitrage_strategy, stop_arbitrage_strategy,
    trigger_circuit_breaker,
};
pub use chat::{
    get_chat_history, get_models, get_news_headlines, get_persona, pause_trading, research_sonar,
    send_message, summon_swarm, update_persona,
};
pub use market_data::{get_batch_market_data, get_historical_data, get_market_data};
pub use portfolio::{get_performance_metrics, get_portfolio, get_position, get_positions};
pub use strategies::{
    create_strategy, delete_strategy, execute_strategy, get_strategy, list_strategies,
    update_strategy,
};
pub use trades::{create_trade, delete_trade, get_trade, list_trades, update_trade};

/// Health check endpoint
///
/// Returns the current health status of the API server and its dependencies.
/// This endpoint is used for monitoring and load balancer health checks.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "service": "ninja-gekko-api"
    }))
}

/// API information endpoint
///
/// Returns general information about the API including available endpoints,
/// version information, and supported features.
pub async fn api_info() -> Json<ApiResponse<serde_json::Value>> {
    let info = json!({
        "name": "Ninja Gekko Trading API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "High-performance REST API for autonomous trading operations",
        "endpoints": {
            "health": "/health",
            "auth": {
                "login": "/api/v1/auth/login",
                "refresh": "/api/v1/auth/refresh",
                "logout": "/api/v1/auth/logout"
            },
            "trades": {
                "list": "/api/v1/trades",
                "create": "/api/v1/trades",
                "get": "/api/v1/trades/{id}",
                "update": "/api/v1/trades/{id}",
                "delete": "/api/v1/trades/{id}"
            },
            "portfolio": {
                "overview": "/api/v1/portfolio",
                "positions": "/api/v1/portfolio/positions",
                "position": "/api/v1/portfolio/positions/{symbol}",
                "performance": "/api/v1/portfolio/performance"
            },
            "market_data": {
                "overview": "/api/v1/market-data",
                "symbol": "/api/v1/market-data/{symbol}",
                "history": "/api/v1/market-data/{symbol}/history"
            },
            "strategies": {
                "list": "/api/v1/strategies",
                "create": "/api/v1/strategies",
                "get": "/api/v1/strategies/{id}",
                "update": "/api/v1/strategies/{id}",
                "delete": "/api/v1/strategies/{id}",
                "execute": "/api/v1/strategies/{id}/execute"
            },
            "websocket": "/api/v1/ws"
        },
        "features": [
            "REST API",
            "WebSocket real-time updates",
            "JWT Authentication",
            "Rate limiting",
            "CORS support",
            "Comprehensive error handling"
        ],
        "documentation": "/api/v1/docs"
    });

    Json(ApiResponse::success(info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.0.get("status").unwrap(), "healthy");
    }

    #[tokio::test]
    async fn test_api_info() {
        let response = api_info().await;
        assert!(response.0.success);
        assert!(response.0.data.is_some());
    }
}

//! Portfolio management endpoint handlers
//!
//! This module provides HTTP handlers for portfolio-related operations including
//! retrieving portfolio information, positions, and performance metrics.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    error::{ApiError, ApiResult},
    models::{
        AllocationResponse, ApiResponse, PaginatedResponse, PaginationParams,
        PerformanceMetricsResponse, PortfolioHistoryResponse, PortfolioResponse,
        PortfolioSummaryRequest, PositionResponse, RebalanceRequest, RebalanceResponse,
        RiskMetricsResponse,
    },
    AppState,
};

/// Get complete portfolio information
pub async fn get_portfolio(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<PortfolioResponse>>> {
    info!("Retrieving complete portfolio information");

    match state.portfolio_manager.get_portfolio().await {
        Ok(portfolio) => Ok(Json(ApiResponse::success(portfolio))),
        Err(e) => {
            warn!("Failed to retrieve portfolio: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve portfolio: {}", e),
            })
        }
    }
}

/// Get portfolio summary with optional filtering
pub async fn get_portfolio_summary(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PortfolioSummaryRequest>,
) -> ApiResult<Json<ApiResponse<PortfolioResponse>>> {
    info!("Retrieving portfolio summary with params: {:?}", params);

    match state.portfolio_manager.get_portfolio_summary(params).await {
        Ok(summary) => Ok(Json(ApiResponse::success(summary))),
        Err(e) => {
            warn!("Failed to retrieve portfolio summary: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve portfolio summary: {}", e),
            })
        }
    }
}

/// Get positions with pagination and filtering
pub async fn get_positions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<PositionResponse>>> {
    info!("Retrieving positions with pagination: {:?}", params);

    match state.portfolio_manager.get_positions(params).await {
        Ok(positions) => Ok(Json(positions)),
        Err(e) => {
            warn!("Failed to retrieve positions: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve positions: {}", e),
            })
        }
    }
}

/// Get specific position by symbol
pub async fn get_position(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<PositionResponse>>> {
    info!("Retrieving position for symbol: {}", symbol);

    match state.portfolio_manager.get_position(&symbol).await {
        Ok(Some(position)) => Ok(Json(ApiResponse::success(position))),
        Ok(None) => Err(ApiError::NotFound {
            resource: format!("Position for symbol {}", symbol),
        }),
        Err(e) => {
            warn!("Failed to retrieve position for {}: {}", symbol, e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve position: {}", e),
            })
        }
    }
}

/// Get portfolio performance metrics
pub async fn get_performance_metrics(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<PerformanceMetricsResponse>>> {
    info!("Retrieving portfolio performance metrics");

    match state.portfolio_manager.get_performance_metrics().await {
        Ok(metrics) => Ok(Json(ApiResponse::success(metrics))),
        Err(e) => {
            warn!("Failed to retrieve performance metrics: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve performance metrics: {}", e),
            })
        }
    }
}

/// Get portfolio allocation breakdown
pub async fn get_allocation_breakdown(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<Vec<AllocationResponse>>>> {
    info!("Retrieving portfolio allocation breakdown");

    match state.portfolio_manager.get_allocation_breakdown().await {
        Ok(allocations) => Ok(Json(ApiResponse::success(allocations))),
        Err(e) => {
            warn!("Failed to retrieve allocation breakdown: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve allocation breakdown: {}", e),
            })
        }
    }
}

/// Rebalance portfolio based on target allocations
pub async fn rebalance_portfolio(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RebalanceRequest>,
) -> ApiResult<Json<ApiResponse<RebalanceResponse>>> {
    info!("Rebalancing portfolio with request: {:?}", request);

    match state.portfolio_manager.rebalance_portfolio(request).await {
        Ok(rebalance_result) => Ok(Json(ApiResponse::success(rebalance_result))),
        Err(e) => {
            warn!("Failed to rebalance portfolio: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to rebalance portfolio: {}", e),
            })
        }
    }
}

/// Get portfolio historical data
pub async fn get_portfolio_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<PortfolioHistoryResponse>>> {
    info!("Retrieving portfolio history with pagination: {:?}", params);

    match state.portfolio_manager.get_portfolio_history(params).await {
        Ok(history) => Ok(Json(history)),
        Err(e) => {
            warn!("Failed to retrieve portfolio history: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve portfolio history: {}", e),
            })
        }
    }
}

/// Get portfolio risk metrics
pub async fn get_risk_metrics(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<RiskMetricsResponse>>> {
    info!("Retrieving portfolio risk metrics");

    match state.portfolio_manager.get_risk_metrics().await {
        Ok(metrics) => Ok(Json(ApiResponse::success(metrics))),
        Err(e) => {
            warn!("Failed to retrieve risk metrics: {}", e);
            Err(ApiError::Portfolio {
                message: format!("Failed to retrieve risk metrics: {}", e),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use std::sync::Arc;

    #[tokio::test]
    #[ignore]
    async fn test_get_portfolio_success() {
        let state = Arc::new(
            AppState::new(crate::config::ApiConfig::default())
                .await
                .unwrap(),
        );
        let result = get_portfolio(State(state)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_portfolio_summary_success() {
        let state = Arc::new(
            AppState::new(crate::config::ApiConfig::default())
                .await
                .unwrap(),
        );
        let params = PortfolioSummaryRequest::default();
        let result = get_portfolio_summary(State(state), Query(params)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_positions_success() {
        let state = Arc::new(
            AppState::new(crate::config::ApiConfig::default())
                .await
                .unwrap(),
        );
        let params = PaginationParams::default();
        let result = get_positions(State(state), Query(params)).await;

        assert!(result.is_ok());
    }
}

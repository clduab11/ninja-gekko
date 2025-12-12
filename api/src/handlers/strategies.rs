//! Trading strategy endpoint handlers
//!
//! This module provides HTTP handlers for trading strategy operations including
//! strategy management, execution, backtesting, and performance analysis.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    error::{ApiError, ApiResult},
    models::{
        ApiResponse, BacktestRequest, BacktestResponse, CreateStrategyRequest,
        DetailedStrategyPerformance, PaginatedResponse, PaginationParams, StrategyExecutionRequest,
        StrategyExecutionResponse, StrategyOptimizationRequest, StrategyOptimizationResponse,
        StrategyResponse, UpdateStrategyRequest,
    },
    AppState,
};

/// Get all available trading strategies
pub async fn list_strategies(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<StrategyResponse>>> {
    info!("Retrieving strategies with pagination: {:?}", params);

    // Validate pagination parameters
    let mut params = params;
    if let Err(e) = params.validate() {
        return Err(ApiError::Validation {
            message: e,
            field: Some("pagination".to_string()),
        });
    }

    match state.strategy_manager.list_strategies(params).await {
        Ok(strategies) => Ok(Json(strategies)),
        Err(e) => {
            warn!("Failed to list strategies: {}", e);
            Err(ApiError::Strategy {
                message: format!("Failed to list strategies: {}", e),
            })
        }
    }
}

/// Get a specific trading strategy by ID
pub async fn get_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<StrategyResponse>>> {
    info!("Retrieving strategy: {}", strategy_id);

    match state.strategy_manager.get_strategy(&strategy_id).await {
        Ok(Some(strategy)) => Ok(Json(ApiResponse::success(strategy))),
        Ok(None) => Err(ApiError::NotFound {
            resource: format!("Strategy with ID {}", strategy_id),
        }),
        Err(e) => {
            warn!("Failed to retrieve strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy {
                message: format!("Failed to retrieve strategy: {}", e),
            })
        }
    }
}

/// Create a new trading strategy
pub async fn create_strategy(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateStrategyRequest>,
) -> ApiResult<Json<ApiResponse<StrategyResponse>>> {
    info!("Creating new strategy: {}", request.name);

    // Validate the request
    if let Err(e) = request.validate() {
        return Err(ApiError::Validation {
            message: e,
            field: Some("strategy".to_string()),
        });
    }

    match state.strategy_manager.create_strategy(request).await {
        Ok(strategy) => Ok(Json(ApiResponse::success(strategy))),
        Err(e) => {
            warn!("Failed to create strategy: {}", e);
            Err(ApiError::Strategy {
                message: format!("Failed to create strategy: {}", e),
            })
        }
    }
}

/// Update an existing trading strategy
pub async fn update_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<UpdateStrategyRequest>,
) -> ApiResult<Json<ApiResponse<StrategyResponse>>> {
    info!("Updating strategy: {}", strategy_id);

    match state
        .strategy_manager
        .update_strategy(&strategy_id, request)
        .await
    {
        Ok(strategy) => Ok(Json(ApiResponse::success(strategy))),
        Err(e) => {
            warn!("Failed to update strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy {
                message: format!("Failed to update strategy: {}", e),
            })
        }
    }
}

/// Delete a trading strategy
pub async fn delete_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    info!("Deleting strategy: {}", strategy_id);

    match state.strategy_manager.delete_strategy(&strategy_id).await {
        Ok(_) => {
            let response = json!({
                "message": "Strategy deleted successfully",
                "strategy_id": strategy_id
            });

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to delete strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy {
                message: format!("Failed to delete strategy: {}", e),
            })
        }
    }
}

/// Execute a trading strategy
pub async fn execute_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<StrategyExecutionRequest>,
) -> ApiResult<Json<ApiResponse<StrategyExecutionResponse>>> {
    info!("Executing strategy: {}", strategy_id);

    match state
        .strategy_manager
        .execute_strategy(&strategy_id, request)
        .await
    {
        Ok(execution_result) => Ok(Json(ApiResponse::success(execution_result))),
        Err(e) => {
            warn!("Failed to execute strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy {
                message: format!("Failed to execute strategy: {}", e),
            })
        }
    }
}

/// Get strategy execution history
pub async fn get_strategy_executions(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<StrategyExecutionResponse>>> {
    info!(
        "Retrieving execution history for strategy: {} with params: {:?}",
        strategy_id, params
    );

    match state
        .strategy_manager
        .get_execution_history(&strategy_id, params)
        .await
    {
        Ok(executions) => Ok(Json(executions)),
        Err(e) => {
            warn!(
                "Failed to retrieve execution history for {}: {}",
                strategy_id, e
            );
            Err(ApiError::Strategy {
                message: format!("Failed to retrieve execution history: {}", e),
            })
        }
    }
}

/// Backtest a trading strategy
pub async fn backtest_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<BacktestRequest>,
) -> ApiResult<Json<ApiResponse<BacktestResponse>>> {
    info!("Backtesting strategy: {}", strategy_id);

    match state
        .strategy_manager
        .backtest_strategy(&strategy_id, request)
        .await
    {
        Ok(backtest_result) => Ok(Json(ApiResponse::success(backtest_result))),
        Err(e) => {
            warn!("Failed to backtest strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy {
                message: format!("Failed to backtest strategy: {}", e),
            })
        }
    }
}

/// Optimize strategy parameters
pub async fn optimize_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<StrategyOptimizationRequest>,
) -> ApiResult<Json<ApiResponse<StrategyOptimizationResponse>>> {
    info!("Optimizing strategy: {}", strategy_id);

    match state
        .strategy_manager
        .optimize_strategy(&strategy_id, request)
        .await
    {
        Ok(optimization_result) => Ok(Json(ApiResponse::success(optimization_result))),
        Err(e) => {
            warn!("Failed to optimize strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy {
                message: format!("Failed to optimize strategy: {}", e),
            })
        }
    }
}

/// Get strategy performance metrics
pub async fn get_strategy_performance(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<DetailedStrategyPerformance>>> {
    info!(
        "Retrieving detailed performance for strategy: {}",
        strategy_id
    );

    match state
        .strategy_manager
        .get_detailed_performance(&strategy_id)
        .await
    {
        Ok(performance) => Ok(Json(ApiResponse::success(performance))),
        Err(e) => {
            warn!("Failed to retrieve performance for {}: {}", strategy_id, e);
            Err(ApiError::Strategy {
                message: format!("Failed to retrieve strategy performance: {}", e),
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
    async fn test_list_strategies_success() {
        let state = Arc::new(
            AppState::new(crate::config::ApiConfig::default())
                .await
                .unwrap(),
        );
        let params = PaginationParams::default();
        let result = list_strategies(State(state), Query(params)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_strategy_success() {
        let state = Arc::new(
            AppState::new(crate::config::ApiConfig::default())
                .await
                .unwrap(),
        );
        let result = get_strategy(State(state), Path("test-strategy".to_string())).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_strategy_success() {
        let state = Arc::new(
            AppState::new(crate::config::ApiConfig::default())
                .await
                .unwrap(),
        );
        let request = CreateStrategyRequest {
            name: "Test Strategy".to_string(),
            description: Some("Test strategy description".to_string()),
            parameters: std::collections::HashMap::new(),
            is_active: Some(true),
            account_ids: Some(vec!["test-account".to_string()]),
        };
        let result = create_strategy(State(state), Json(request)).await;

        assert!(result.is_ok());
    }
}

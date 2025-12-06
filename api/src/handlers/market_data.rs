//! Market data endpoint handlers
//!
//! This module provides HTTP handlers for market data operations including
//! real-time price feeds, historical data, and market indicators.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    error::{ApiError, ApiResult},
    models::{
        ApiResponse, PaginationParams, PaginatedResponse,
        MarketDataResponse, MarketDataRequest, MarketDataPoint,
        MarketDataWithIndicators, SymbolInfo, SearchSymbolsRequest,
        MarketOverview, StreamSubscriptionResponse, MarketStatistics,
    },
    AppState,
};

/// Get current market data for a specific symbol
pub async fn get_market_data(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<MarketDataResponse>>> {
    info!("Retrieving market data for symbol: {}", symbol);

    match state.market_data_service.get_latest_data(&symbol).await {
        Ok(data) => {
            Ok(Json(ApiResponse::success(data)))
        }
        Err(e) => {
            warn!("Failed to retrieve market data for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve market data: {}", e) })
        }
    }
}

/// Get market data for multiple symbols
pub async fn get_batch_market_data(
    State(state): State<Arc<AppState>>,
    Query(request): Query<MarketDataRequest>,
) -> ApiResult<Json<ApiResponse<Vec<MarketDataResponse>>>> {
    info!("Retrieving batch market data for symbols: {:?}", request.symbols);

    if request.symbols.is_empty() {
        return Err(ApiError::Validation { message: "Symbols list cannot be empty".to_string(), field: Some("symbols".to_string()) });
    }

    match state.market_data_service.get_batch_data(&request.symbols).await {
        Ok(data_list) => {
            Ok(Json(ApiResponse::success(data_list)))
        }
        Err(e) => {
            warn!("Failed to retrieve batch market data: {}", e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve batch market data: {}", e) })
        }
    }
}

/// Get historical market data for a symbol
pub async fn get_historical_data(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<MarketDataPoint>>> {
    info!("Retrieving historical data for symbol: {} with params: {:?}", symbol, params);

    // Validate pagination parameters
    let mut params = params;
    if let Err(e) = params.validate() {
        return Err(ApiError::Validation { message: e, field: Some("pagination".to_string()) });
    }

    match state.market_data_service.get_historical_data(&symbol, params).await {
        Ok(history) => {
            // PaginatedResponse is already returned by the service
            Ok(Json(history))
        }
        Err(e) => {
            warn!("Failed to retrieve historical data for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve historical data: {}", e) })
        }
    }
}

/// Get price history with technical indicators
pub async fn get_price_with_indicators(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<MarketDataWithIndicators>>> {
    info!("Retrieving price data with indicators for symbol: {}", symbol);

    match state.market_data_service.get_data_with_indicators(&symbol, params).await {
        Ok(data) => {
            Ok(Json(ApiResponse::success(data)))
        }
        Err(e) => {
            warn!("Failed to retrieve price with indicators for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve price with indicators: {}", e) })
        }
    }
}

/// Search for symbols based on query
pub async fn search_symbols(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchSymbolsRequest>,
) -> ApiResult<Json<ApiResponse<Vec<SymbolInfo>>>> {
    info!("Searching symbols with query: {}", params.query);

    if params.query.trim().is_empty() {
        return Err(ApiError::Validation { message: "Search query cannot be empty".to_string(), field: Some("query".to_string()) });
    }

    match state.market_data_service.search_symbols(&params.query, params.limit).await {
        Ok(symbols) => {
            Ok(Json(ApiResponse::success(symbols)))
        }
        Err(e) => {
            warn!("Failed to search symbols: {}", e);
            Err(ApiError::MarketData { message: format!("Failed to search symbols: {}", e) })
        }
    }
}

/// Get market overview with top gainers, losers, and volume leaders
pub async fn get_market_overview(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<MarketOverview>>> {
    info!("Retrieving market overview");

    match state.market_data_service.get_market_overview().await {
        Ok(overview) => {
            Ok(Json(ApiResponse::success(overview)))
        }
        Err(e) => {
            warn!("Failed to retrieve market overview: {}", e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve market overview: {}", e) })
        }
    }
}

/// Get real-time price stream for a symbol (WebSocket upgrade)
pub async fn get_price_stream(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<StreamSubscriptionResponse>>> {
    info!("Starting price stream for symbol: {}", symbol);

    match state.market_data_service.subscribe_to_price_stream(&symbol).await {
        Ok(subscription) => {
            Ok(Json(ApiResponse::success(subscription)))
        }
        Err(e) => {
            warn!("Failed to start price stream for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to start price stream: {}", e) })
        }
    }
}

/// Get market statistics for a symbol
pub async fn get_market_statistics(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<MarketStatistics>>> {
    info!("Retrieving market statistics for symbol: {}", symbol);

    match state.market_data_service.get_market_statistics(&symbol).await {
        Ok(stats) => {
            Ok(Json(ApiResponse::success(stats)))
        }
        Err(e) => {
            warn!("Failed to retrieve market statistics for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve market statistics: {}", e) })
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
    async fn test_get_market_data_success() {
        let state = Arc::new(AppState::new(crate::config::ApiConfig::default()).await.unwrap());
        let result = get_market_data(State(state), Path("AAPL".to_string())).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_batch_market_data_success() {
        let state = Arc::new(AppState::new(crate::config::ApiConfig::default()).await.unwrap());
        let request = MarketDataRequest {
            symbols: vec!["AAPL".to_string(), "GOOGL".to_string()],
            include_history: Some(false),
            history_limit: None,
        };
        let result = get_batch_market_data(State(state), Query(request)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_search_symbols_success() {
        let state = Arc::new(AppState::new(crate::config::ApiConfig::default()).await.unwrap());
        let params = SearchSymbolsRequest {
            query: "Apple".to_string(),
            asset_class: None,
            limit: Some(10),
        };
        let result = search_symbols(State(state), Query(params)).await;

        assert!(result.is_ok());
    }
}
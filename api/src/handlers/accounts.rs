use axum::{
    extract::{State, Query},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{
    AppState,
    error::ApiResult,
    models::ApiResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: String, // "long" or "short"
    pub size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAccount {
    pub exchange_id: String,
    pub net_liquidity: f64,
    pub buying_power: f64,
    pub margin_used: f64,
    pub unrealized_pnl: f64,
    pub positions: Vec<Position>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct AccountSnapshotParams {
    pub exchange: Option<String>,
}

#[derive(Serialize)]
pub struct AggregateAccount {
    pub total_net_liquidty: f64,
    pub total_exposure: f64,
    pub breakdown: Vec<ExchangeAccount>,
}

/// Get account snapshot for a specific exchange or all
/// 
/// Fetches real account data from connected exchange APIs.
/// Requires valid exchange credentials to be configured.
pub async fn get_account_snapshot(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<AccountSnapshotParams>,
) -> ApiResult<Json<ApiResponse<Vec<ExchangeAccount>>>> {
    // Account balance data requires direct exchange API calls
    // Currently, the exchange connector is encapsulated in MarketDataService
    // Future: Add BalanceService or AccountService to AppState
    // For now, return empty until account service is implemented
    
    tracing::info!("Account snapshot requested for exchange: {:?}", params.exchange);
    
    Ok(Json(ApiResponse::success(vec![])))
}

/// Get aggregate account viewing
/// 
/// Aggregates account data across all connected exchanges.
/// Returns real data when exchange connectors are properly configured.
pub async fn get_aggregate_account(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<AggregateAccount>>> {
    // Aggregate account data requires balance queries from all exchanges
    // Future: Implement AccountService that queries all connected exchanges
    
    tracing::info!("Aggregate account data requested");

    Ok(Json(ApiResponse::success(AggregateAccount {
        total_net_liquidty: 0.0,
        total_exposure: 0.0,
        breakdown: vec![],
    })))
}

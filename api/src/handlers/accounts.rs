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
use exchange_connectors::ExchangeId;

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
pub async fn get_account_snapshot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccountSnapshotParams>,
) -> ApiResult<Json<ApiResponse<Vec<ExchangeAccount>>>> {
    // In real impl, fetch from portfolio_manager or exchange connectors
    // For now, mock responses based on requested exchange
    
    let now = chrono::Utc::now();
    let mut accounts = Vec::new();

    let exchanges = match params.exchange.as_deref() {
        Some(ex) => vec![ex],
        None => vec!["binance_us", "kraken", "oanda"],
    };

    for ex in exchanges {
        let (net_liq, positions) = match ex {
            "binance_us" => (
                50000.0,
                vec![
                    Position {
                        symbol: "BTC/USD".to_string(),
                        side: "long".to_string(),
                        size: 0.5,
                        entry_price: 90000.0,
                        current_price: 92000.0,
                        unrealized_pnl: 1000.0,
                    }
                ]
            ),
            "kraken" => (
                25000.0,
                vec![
                    Position {
                        symbol: "ETH/USD".to_string(),
                        side: "long".to_string(),
                        size: 10.0,
                        entry_price: 3000.0,
                        current_price: 3100.0,
                        unrealized_pnl: 1000.0,
                    }
                ]
            ),
            "oanda" => (
                10000.0,
                vec![
                    Position {
                        symbol: "EUR/USD".to_string(),
                        side: "short".to_string(),
                        size: 50000.0,
                        entry_price: 1.0850,
                        current_price: 1.0820,
                        unrealized_pnl: 150.0,
                    }
                ]
            ),
            _ => (0.0, vec![]),
        };

        if ex == "binance_us" || ex == "kraken" || ex == "oanda" {
             accounts.push(ExchangeAccount {
                exchange_id: ex.to_string(),
                net_liquidity: net_liq,
                buying_power: net_liq * 0.5, // Mock data
                margin_used: net_liq * 0.2, // Mock data
                unrealized_pnl: positions.iter().map(|p| p.unrealized_pnl).sum(),
                positions,
                updated_at: now,
            });
        }
    }

    Ok(Json(ApiResponse::success(accounts)))
}

/// Get aggregate account viewing
pub async fn get_aggregate_account(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<AggregateAccount>>> {
    // Re-use logic or call aggregation service
    // For mock, just sum up hardcoded values
    
    let binance = ExchangeAccount {
        exchange_id: "binance_us".to_string(),
        net_liquidity: 50000.0,
        buying_power: 25000.0,
        margin_used: 10000.0,
        unrealized_pnl: 1000.0,
        positions: vec![],
        updated_at: chrono::Utc::now(),
    };
     let kraken = ExchangeAccount {
        exchange_id: "kraken".to_string(),
        net_liquidity: 25000.0,
        buying_power: 12500.0,
        margin_used: 5000.0,
        unrealized_pnl: 1000.0,
        positions: vec![],
        updated_at: chrono::Utc::now(),
    };
    
    let total_liq = binance.net_liquidity + kraken.net_liquidity;
    let total_exposure = binance.margin_used + kraken.margin_used; // Mock exposure logic

    Ok(Json(ApiResponse::success(AggregateAccount {
        total_net_liquidty: total_liq,
        total_exposure: total_exposure,
        breakdown: vec![binance, kraken],
    })))
}

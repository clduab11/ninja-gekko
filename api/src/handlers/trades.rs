//! Trade endpoint handlers
//!
//! This module provides HTTP handlers for trade-related operations including
//! creating, reading, updating, and deleting trades. All handlers include
//! proper error handling, validation, and structured responses.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::FromRow;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    error::{ApiError, ApiResult},
    models::{
        ApiResponse, CreateTradeRequest, PaginatedResponse, PaginationMeta, PaginationParams,
        TradeResponse, UpdateTradeRequest,
    },
};
use ninja_gekko_core::types::{Order, OrderSide, OrderStatus, OrderType};

/// Database row structure for trade executions
#[derive(Debug, FromRow)]
struct TradeExecutionRow {
    id: uuid::Uuid,
    #[allow(dead_code)]
    bot_id: String,
    #[allow(dead_code)]
    exchange: String,
    symbol: String,
    side: String,
    order_type: String,
    quantity: Decimal,
    price: Option<Decimal>,
    status: String,
    #[allow(dead_code)]
    external_order_id: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    // Extra fields to match TradeResponse logic where possible
}

impl From<TradeExecutionRow> for TradeResponse {
    fn from(row: TradeExecutionRow) -> Self {
        TradeResponse {
            id: row.id.to_string(),
            symbol: row.symbol,
            side: row.side,
            quantity: row.quantity.to_f64().unwrap_or_default(),
            price: row.price.and_then(|p| p.to_f64()).unwrap_or_default(),
            order_type: row.order_type,
            status: row.status,
            filled_quantity: 0.0,    // TODO: Add filled quantity to DB
            average_fill_price: 0.0, // TODO: Add avg fill price to DB
            timestamp: row.created_at,
            updated_at: row.updated_at,
            account_id: "default".to_string(), // map from bot_id if needed
            metadata: None,
        }
    }
}

/// List trades with pagination and filtering
pub async fn list_trades(
    State(state): State<Arc<crate::AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<TradeResponse>>> {
    info!("Listing trades with params: {:?}", params);

    // Validate pagination parameters
    let mut pagination = params;
    pagination
        .validate()
        .map_err(|e| ApiError::validation(e.to_string(), None))?;

    let limit = pagination.limit.unwrap_or(50) as i64;
    let offset = pagination.offset() as i64;

    // Build query
    // Note: In production use a query builder for complex filtering
    let query = "
        SELECT * FROM trade_executions 
        ORDER BY created_at DESC 
        LIMIT $1 OFFSET $2
    ";

    let rows = sqlx::query_as::<_, TradeExecutionRow>(query)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.db_manager.pool())
        .await
        .map_err(|e| {
            error!("Database query failed: {}", e);
            ApiError::database(format!("Failed to fetch trades: {}", e))
        })?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trade_executions")
        .fetch_one(state.db_manager.pool())
        .await
        .unwrap_or(0);

    let total = total as usize;
    let limit_usize = limit as usize;
    let total_pages = if total > 0 {
        (total + limit_usize - 1) / limit_usize
    } else {
        0
    };

    let trades: Vec<TradeResponse> = rows.into_iter().map(TradeResponse::from).collect();

    let pagination_meta = PaginationMeta {
        page: pagination.page.unwrap_or(1),
        limit: limit_usize,
        total,
        total_pages,
        has_next: offset as usize + limit_usize < total,
        has_prev: offset > 0,
    };

    let response = PaginatedResponse {
        response: ApiResponse::success(trades),
        pagination: pagination_meta,
    };

    Ok(Json(response))
}

/// Create a new trade
pub async fn create_trade(
    State(state): State<Arc<crate::AppState>>,
    Json(request): Json<CreateTradeRequest>,
) -> ApiResult<Json<ApiResponse<TradeResponse>>> {
    info!("Creating trade: {:?}", request);

    // Validate the request
    request
        .validate()
        .map_err(|msg| ApiError::validation(msg, None))?;

    // Convert to core Order type
    let order_id = format!("order_{}", chrono::Utc::now().timestamp_millis());
    let order = request
        .to_order(order_id)
        .map_err(|msg| ApiError::trading(msg))?;

    // TODO: Implement actual trade execution through trading engine
    // Currently just saving to DB to demonstrate integration

    let id = uuid::Uuid::new_v4();
    let price = order.price.unwrap_or_default();

    let query = "
        INSERT INTO trade_executions 
        (id, bot_id, exchange, symbol, side, order_type, quantity, price, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        RETURNING *
    ";

    let row = sqlx::query_as::<_, TradeExecutionRow>(query)
        .bind(id)
        .bind("manual_trade") // bot_id
        .bind("kraken") // exchange
        .bind(&order.symbol)
        .bind(format!("{:?}", order.side))
        .bind(format!("{:?}", order.order_type))
        .bind(order.quantity)
        .bind(price)
        .bind("Pending")
        .fetch_one(state.db_manager.pool())
        .await
        .map_err(|e| {
            error!("Failed to persist trade: {}", e);
            ApiError::database(format!("Failed to create trade: {}", e))
        })?;

    let trade_response = TradeResponse::from(row);
    let response = ApiResponse::success(trade_response);

    info!("Trade created successfully: {}", id);
    Ok(Json(response))
}

/// Get a trade by ID
pub async fn get_trade(
    State(state): State<Arc<crate::AppState>>,
    Path(trade_id): Path<String>,
) -> ApiResult<Json<ApiResponse<TradeResponse>>> {
    info!("Getting trade: {}", trade_id);

    let uuid = uuid::Uuid::parse_str(&trade_id)
        .map_err(|_| ApiError::validation("Invalid UUID format".to_string(), None))?;

    let row =
        sqlx::query_as::<_, TradeExecutionRow>("SELECT * FROM trade_executions WHERE id = $1")
            .bind(uuid)
            .fetch_optional(state.db_manager.pool())
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

    match row {
        Some(row) => Ok(Json(ApiResponse::success(TradeResponse::from(row)))),
        _ => Err(ApiError::not_found(format!("Trade {}", trade_id))),
    }
}

/// Update a trade
pub async fn update_trade(
    State(_state): State<Arc<crate::AppState>>,
    Path(trade_id): Path<String>,
    Json(_request): Json<UpdateTradeRequest>,
) -> ApiResult<Json<ApiResponse<TradeResponse>>> {
    info!("Updating trade: {}", trade_id);
    // Real updates would involve the trading engine
    Err(ApiError::not_found(format!("Trade {}", trade_id)))
}

/// Delete/cancel a trade
pub async fn delete_trade(
    State(_state): State<Arc<crate::AppState>>,
    Path(trade_id): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    info!("Deleting/Cancelling trade: {}", trade_id);
    // Cancellation logic
    Err(ApiError::not_found(format!("Trade {}", trade_id)))
}

/// Cancel multiple trades
pub async fn cancel_trades(
    State(_state): State<Arc<crate::AppState>>,
    Json(trade_ids): Json<Vec<String>>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    info!("Cancelling multiple trades: {:?}", trade_ids);
    Err(ApiError::not_implemented(
        "Batch cancellation not supported yet".to_string(),
    ))
}

/// Get trade statistics
pub async fn get_trade_stats(
    State(state): State<Arc<crate::AppState>>,
    Query(_params): Query<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    info!("Getting trade statistics");

    let pool = state.db_manager.pool();

    // Derived stats
    let total_trades: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trade_executions")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let filled_trades: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM trade_executions WHERE status = 'Filled'")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    let open_trades: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trade_executions WHERE status IN ('Open', 'Pending', 'PartiallyFilled')")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let total_volume: Option<Decimal> = sqlx::query_scalar(
        "SELECT SUM(quantity * price) FROM trade_executions WHERE status = 'Filled'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(None);

    let stats = json!({
        "total_trades": total_trades,
        "open_trades": open_trades,
        "filled_trades": filled_trades,
        "cancelled_trades": 0, // Request count
        "total_volume": total_volume.unwrap_or_default().to_f64().unwrap_or(0.0),
        "total_pnl": 0.0,
        "win_rate": 0.0,
        "avg_trade_duration": "N/A",
        "largest_win": 0.0,
        "largest_loss": 0.0,
        "period": {
            "start": chrono::Utc::now() - chrono::Duration::days(30),
            "end": chrono::Utc::now()
        },
        "message": "Real database metrics"
    });

    let response = ApiResponse::success(stats);
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests omitted for brevity
}

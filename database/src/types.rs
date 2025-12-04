//! Database type definitions
//!
//! This module provides common types used across the database layer.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Trade record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TradeRecord {
    pub id: Uuid,
    pub symbol: String,
    pub side: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub exchange: String,
    pub order_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub account_id: String,
    pub strategy_id: Option<String>,
    pub fees: Decimal,
    pub fill_price: Option<Decimal>,
    pub filled_quantity: Option<Decimal>,
}

/// Position record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PositionRecord {
    pub id: Uuid,
    pub symbol: String,
    pub quantity: Decimal,
    pub average_entry_price: Decimal,
    pub current_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub exchange: String,
    pub account_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Portfolio record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortfolioRecord {
    pub id: Uuid,
    pub account_id: String,
    pub total_value: Decimal,
    pub cash_balance: Decimal,
    pub positions_value: Decimal,
    pub daily_pnl: Decimal,
    pub total_pnl: Decimal,
    pub margin_used: Decimal,
    pub buying_power: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Strategy record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StrategyRecord {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub strategy_type: String,
    pub parameters: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub performance_metrics: Option<serde_json::Value>,
}

/// Risk event record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskEventRecord {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub message: String,
    pub details: serde_json::Value,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

/// Audit log record for compliance
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogRecord {
    pub id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub actor: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
}

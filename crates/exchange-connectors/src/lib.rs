//! Exchange Connectors for Ninja Gekko
//!
//! This crate provides unified exchange connectors for:
//! - Coinbase Pro/Advanced Trade API
//! - Binance.us API  
//! - OANDA v20 REST API
//!
//! All connectors implement the `ExchangeConnector` trait for consistent interface.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

pub mod binance_us;
pub mod coinbase;
pub mod credentials;
pub mod oanda;

/// Exchange connector error types
#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Exchange API error: {code} - {message}")]
    Api { code: String, message: String },

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance {
        required: Decimal,
        available: Decimal,
    },

    #[error("Symbol not supported: {0}")]
    UnsupportedSymbol(String),

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type ExchangeResult<T> = Result<T, ExchangeError>;

/// Unique identifiers for exchanges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExchangeId {
    Coinbase,
    BinanceUs,
    Oanda,
}

/// Trading pair representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradingPair {
    pub base: String,
    pub quote: String,
    pub symbol: String, // e.g., "BTC-USD", "EUR_USD"
}

/// Order side enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

/// Order status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Order representation for exchange APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeOrder {
    pub id: String,
    pub exchange_id: ExchangeId,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub status: OrderStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub fills: Vec<Fill>,
}

/// Fill/execution representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub id: String,
    pub order_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub fee: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Account balance representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub currency: String,
    pub available: Decimal,
    pub total: Decimal,
    pub hold: Decimal,
}

/// Market data tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
    pub volume_24h: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// WebSocket market data stream message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamMessage {
    Tick(MarketTick),
    OrderUpdate(ExchangeOrder),
    Error(String),
    Ping,
    Pong,
}

/// Fund transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub id: Uuid,
    pub from_exchange: ExchangeId,
    pub to_exchange: ExchangeId,
    pub currency: String,
    pub amount: Decimal,
    pub urgency: TransferUrgency,
}

/// Transfer urgency levels for capital allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferUrgency {
    Low,      // Standard processing time
    Normal,   // Priority processing
    High,     // Expedited processing (higher fees)
    Critical, // Emergency reallocation
}

/// Main exchange connector trait
#[async_trait]
pub trait ExchangeConnector: Send + Sync {
    /// Get exchange identifier
    fn exchange_id(&self) -> ExchangeId;

    /// Connect to the exchange
    async fn connect(&mut self) -> ExchangeResult<()>;

    /// Disconnect from the exchange
    async fn disconnect(&mut self) -> ExchangeResult<()>;

    /// Check if connected and authenticated
    async fn is_connected(&self) -> bool;

    /// Get supported trading pairs
    async fn get_trading_pairs(&self) -> ExchangeResult<Vec<TradingPair>>;

    /// Get account balances
    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>>;

    /// Place an order
    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> ExchangeResult<ExchangeOrder>;

    /// Cancel an order
    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder>;

    /// Get order status
    async fn get_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder>;

    /// Get market data
    async fn get_market_data(&self, symbol: &str) -> ExchangeResult<MarketTick>;

    /// Start WebSocket stream for market data
    async fn start_market_stream(
        &self,
        symbols: Vec<String>,
    ) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>>;

    /// Start WebSocket stream for order updates
    async fn start_order_stream(&self) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>>;

    /// Transfer funds (for supported exchanges)
    async fn transfer_funds(&self, request: TransferRequest) -> ExchangeResult<String>;

    /// Get transfer status
    async fn get_transfer_status(&self, transfer_id: &str) -> ExchangeResult<TransferStatus>;
}

/// Transfer status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Exchange configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub exchange_id: ExchangeId,
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>, // For Coinbase
    pub sandbox: bool,
    pub rate_limit_requests_per_second: u32,
    pub websocket_url: Option<String>,
    pub rest_api_url: Option<String>,
}

/// Rate limiter for API calls
pub struct RateLimiter {
    governor: governor::RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        use governor::{Quota, RateLimiter as GovernorRateLimiter};
        use std::num::NonZeroU32;

        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        let limiter = GovernorRateLimiter::direct(quota);

        Self { governor: limiter }
    }

    pub async fn acquire(&self) -> ExchangeResult<()> {
        self.governor.until_ready().await;
        Ok(())
    }
}

/// Utility functions for exchange implementations
pub mod utils {
    use super::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    /// Generate HMAC-SHA256 signature for API authentication
    pub fn hmac_sha256_signature(secret: &str, message: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();
        base64::encode(result.into_bytes())
    }

    /// Generate timestamp for API calls
    pub fn timestamp() -> String {
        chrono::Utc::now().timestamp().to_string()
    }

    /// Convert decimal to string with proper precision
    pub fn decimal_to_string(value: Decimal, precision: u32) -> String {
        format!("{:.precision$}", value, precision = precision as usize)
    }
}

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core trading types and data structures for the Ninja Gekko trading system.
///
/// This module defines the fundamental types used throughout the trading engine
/// including orders, positions, executions, and market data structures.
use crate::error::{TradingError, TradingResult};

// Re-export arbitrage types for integration
pub use arbitrage_engine::{
    AllocationPriority, AllocationRequest, ArbitrageConfig, ArbitrageOpportunity,
    ExecutionComplexity, TimeSensitivity, VolatilityScore,
};

pub use exchange_connectors::{ExchangeId, TransferRequest, TransferStatus, TransferUrgency};

/// Unique identifier for trading entities
pub type OrderId = Uuid;
pub type ExecutionId = Uuid;
pub type PositionId = Uuid;
pub type PortfolioId = Uuid;

/// Trading symbols and identifiers
pub type Symbol = String;
pub type Exchange = String;
pub type AccountId = String;

/// Core order structure representing a trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub id: OrderId,

    /// Trading symbol (e.g., "AAPL", "BTCUSD")
    pub symbol: Symbol,

    /// Type of order (Market, Limit, Stop, etc.)
    pub order_type: OrderType,

    /// Buy or Sell side
    pub side: OrderSide,

    /// Order quantity
    pub quantity: Decimal,

    /// Order price (None for market orders)
    pub price: Option<Decimal>,

    /// Current order status
    pub status: OrderStatus,

    /// Timestamp when order was created
    pub timestamp: DateTime<Utc>,

    /// Account identifier
    pub account_id: AccountId,

    /// Additional order metadata
    pub metadata: HashMap<String, String>,
}

impl Order {
    /// Creates a new order with the specified parameters
    pub fn new(
        symbol: Symbol,
        order_type: OrderType,
        side: OrderSide,
        quantity: Decimal,
        price: Option<Decimal>,
        account_id: AccountId,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            order_type,
            side,
            quantity,
            price,
            status: OrderStatus::Pending,
            timestamp: Utc::now(),
            account_id,
            metadata: HashMap::new(),
        }
    }

    /// Calculates the order value (price * quantity)
    pub fn value(&self) -> TradingResult<Decimal> {
        match self.price {
            Some(price) => Ok(price * self.quantity),
            None => Err(TradingError::OrderValidation(
                "Cannot calculate value for market order without price".into(),
            )),
        }
    }

    /// Checks if the order is still active (can be modified/cancelled)
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Pending | OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }

    /// Checks if the order is completed (filled or cancelled)
    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected
        )
    }
}

/// Supported order types for trading operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Execute immediately at best available price
    Market,

    /// Execute only at specified price or better
    Limit,

    /// Execute when market reaches specified price
    Stop,

    /// Stop order that converts to limit order
    StopLimit,

    /// Hidden order with visible portion that refreshes
    Iceberg,

    /// Time-weighted average price execution algorithm
    TWAP,

    /// Volume-weighted average price execution algorithm
    VWAP,
}

impl OrderType {
    /// Returns true if the order type supports price specification
    pub fn requires_price(&self) -> bool {
        !matches!(self, OrderType::Market)
    }

    /// Returns true if the order type is algorithmic
    pub fn is_algorithmic(&self) -> bool {
        matches!(self, OrderType::TWAP | OrderType::VWAP | OrderType::Iceberg)
    }
}

/// Buy or sell side of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    /// Returns the opposite side of the order
    pub fn opposite(&self) -> Self {
        match self {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }
}

/// Current status of an order in the trading system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order received but not yet processed
    Pending,

    /// Order accepted and waiting for execution
    Open,

    /// Order partially filled
    PartiallyFilled,

    /// Order completely filled
    Filled,

    /// Order cancelled by user
    Cancelled,

    /// Order rejected by system
    Rejected,
}

/// Execution record for a completed trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    /// Unique execution identifier
    pub id: ExecutionId,

    /// Order that generated this execution
    pub order_id: OrderId,

    /// Symbol that was traded
    pub symbol: Symbol,

    /// Side of the execution
    pub side: OrderSide,

    /// Quantity executed
    pub quantity: Decimal,

    /// Price at which the execution occurred
    pub price: Decimal,

    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,

    /// Exchange where execution occurred
    pub exchange: Exchange,

    /// Fees associated with the execution
    pub fees: Decimal,

    /// Additional execution metadata
    pub metadata: HashMap<String, String>,
}

impl Execution {
    /// Creates a new execution record
    pub fn new(
        order_id: OrderId,
        symbol: Symbol,
        side: OrderSide,
        quantity: Decimal,
        price: Decimal,
        exchange: Exchange,
        fees: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            order_id,
            symbol,
            side,
            quantity,
            price,
            timestamp: Utc::now(),
            exchange,
            fees,
            metadata: HashMap::new(),
        }
    }

    /// Calculates the total value of the execution (quantity * price)
    pub fn total_value(&self) -> Decimal {
        self.quantity * self.price
    }

    /// Calculates the net value after fees
    pub fn net_value(&self) -> Decimal {
        self.total_value() - self.fees
    }
}

/// Position representing current holdings in a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Unique position identifier
    pub id: PositionId,

    /// Account holding the position
    pub account_id: AccountId,

    /// Symbol being held
    pub symbol: Symbol,

    /// Current quantity held (positive for long, negative for short)
    pub quantity: Decimal,

    /// Average price paid for the position
    pub average_price: Decimal,

    /// Unrealized profit/loss
    pub unrealized_pnl: Decimal,

    /// Realized profit/loss from closed portions
    pub realized_pnl: Decimal,

    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl Position {
    /// Creates a new position
    pub fn new(account_id: AccountId, symbol: Symbol, quantity: Decimal, price: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            symbol,
            quantity,
            average_price: price,
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            last_update: Utc::now(),
        }
    }

    /// Updates the position with new execution data
    pub fn update_from_execution(&mut self, execution: &Execution) {
        let old_quantity = self.quantity;
        let old_cost = self.average_price * old_quantity;

        // Update quantity
        self.quantity += execution.quantity;

        // Update average price using weighted average
        if self.quantity != Decimal::ZERO {
            let new_cost = execution.price * execution.quantity;
            let total_cost = old_cost + new_cost;
            self.average_price = total_cost / self.quantity;
        }

        self.last_update = Utc::now();
    }

    /// Calculates the current market value of the position
    pub fn market_value(&self, current_price: Decimal) -> Decimal {
        self.quantity * current_price
    }

    /// Calculates the current unrealized P&L
    pub fn calculate_unrealized_pnl(&self, current_price: Decimal) -> Decimal {
        (current_price - self.average_price) * self.quantity
    }

    /// Checks if the position is closed (zero quantity)
    pub fn is_closed(&self) -> bool {
        self.quantity.is_zero()
    }
}

/// Portfolio representing all positions for an account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// Unique portfolio identifier
    pub id: PortfolioId,

    /// Account identifier
    pub account_id: AccountId,

    /// All positions in the portfolio
    pub positions: HashMap<Symbol, Position>,

    /// Total portfolio value
    pub total_value: Decimal,

    /// Total unrealized P&L
    pub total_unrealized_pnl: Decimal,

    /// Total realized P&L
    pub total_realized_pnl: Decimal,

    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl Portfolio {
    /// Creates a new empty portfolio
    pub fn new(account_id: AccountId) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            positions: HashMap::new(),
            total_value: Decimal::ZERO,
            total_unrealized_pnl: Decimal::ZERO,
            total_realized_pnl: Decimal::ZERO,
            last_update: Utc::now(),
        }
    }

    /// Updates the portfolio with a new execution
    pub fn update_from_execution(&mut self, execution: &Execution) {
        let position = self
            .positions
            .entry(execution.symbol.clone())
            .or_insert_with(|| {
                Position::new(
                    self.account_id.clone(),
                    execution.symbol.clone(),
                    Decimal::ZERO,
                    execution.price,
                )
            });

        position.update_from_execution(execution);
        self.update_totals();
    }

    /// Updates portfolio totals based on current positions
    fn update_totals(&mut self) {
        self.total_value = Decimal::ZERO;
        self.total_unrealized_pnl = Decimal::ZERO;
        self.total_realized_pnl = Decimal::ZERO;

        for position in self.positions.values() {
            self.total_value += position.market_value(position.average_price);
            self.total_unrealized_pnl += position.unrealized_pnl;
            self.total_realized_pnl += position.realized_pnl;
        }

        self.last_update = Utc::now();
    }

    /// Gets a specific position by symbol
    pub fn get_position(&self, symbol: &Symbol) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Gets a mutable reference to a position
    pub fn get_position_mut(&mut self, symbol: &Symbol) -> Option<&mut Position> {
        self.positions.get_mut(symbol)
    }
}

/// Market data structure for price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    /// Trading symbol
    pub symbol: Symbol,

    /// Best bid price
    pub bid: Decimal,

    /// Best ask price
    pub ask: Decimal,

    /// Last traded price
    pub last_price: Decimal,

    /// 24h volume
    pub volume_24h: Decimal,

    /// Timestamp of the data
    pub timestamp: DateTime<Utc>,
}

impl MarketData {
    /// Creates new market data
    pub fn new(
        symbol: Symbol,
        bid: Decimal,
        ask: Decimal,
        last_price: Decimal,
        volume_24h: Decimal,
    ) -> Self {
        Self {
            symbol,
            bid,
            ask,
            last_price,
            volume_24h,
            timestamp: Utc::now(),
        }
    }

    /// Calculates the spread between bid and ask
    pub fn spread(&self) -> Decimal {
        self.ask - self.bid
    }

    /// Calculates the mid price
    pub fn mid_price(&self) -> Decimal {
        (self.bid + self.ask) / Decimal::TWO
    }
}

/// Trading platform/venue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPlatform {
    /// Platform identifier
    pub id: String,

    /// Platform name
    pub name: String,

    /// Supported symbols
    pub supported_symbols: Vec<Symbol>,

    /// Fee structure
    pub fee_structure: FeeStructure,

    /// API rate limits
    pub rate_limits: RateLimits,

    /// Connection status
    pub connected: bool,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Fee structure for a trading platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    /// Maker fee (negative means rebate)
    pub maker_fee: Decimal,

    /// Taker fee
    pub taker_fee: Decimal,

    /// Withdrawal fees per symbol
    pub withdrawal_fees: HashMap<Symbol, Decimal>,
}

impl Default for FeeStructure {
    fn default() -> Self {
        Self {
            maker_fee: Decimal::new(-1, 4), // -0.0001 (0.01% rebate)
            taker_fee: Decimal::new(1, 3),  // 0.001 (0.1%)
            withdrawal_fees: HashMap::new(),
        }
    }
}

/// Rate limiting configuration for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Requests per second
    pub requests_per_second: u32,

    /// Requests per minute
    pub requests_per_minute: u32,

    /// Requests per hour
    pub requests_per_hour: u32,

    /// Burst limit
    pub burst_limit: u32,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            requests_per_minute: 600,
            requests_per_hour: 36000,
            burst_limit: 20,
        }
    }
}

/// Circuit breaker configuration for risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Maximum position size per symbol
    pub max_position_size: Decimal,

    /// Maximum portfolio Value at Risk
    pub max_portfolio_var: Decimal,

    /// Stop loss threshold as percentage
    pub stop_loss_threshold: Decimal,

    /// Circuit breaker thresholds
    pub thresholds: CircuitBreakerThresholds,
}

/// Circuit breaker thresholds for different risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerThresholds {
    /// Maximum drawdown percentage before circuit breaker
    pub max_drawdown_pct: Decimal,

    /// Maximum loss per day
    pub max_daily_loss: Decimal,

    /// Maximum consecutive losses
    pub max_consecutive_losses: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_position_size: Decimal::new(100000, 0), // $100,000
            max_portfolio_var: Decimal::new(50000, 0),  // $50,000
            stop_loss_threshold: Decimal::new(5, 2),    // 5%
            thresholds: CircuitBreakerThresholds {
                max_drawdown_pct: Decimal::new(10, 2),  // 10%
                max_daily_loss: Decimal::new(10000, 0), // $10,000
                max_consecutive_losses: 5,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let order = Order::new(
            "AAPL".to_string(),
            OrderType::Limit,
            OrderSide::Buy,
            Decimal::new(100, 0),
            Some(Decimal::new(15000, 2)),
            "test_account".to_string(),
        );

        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.quantity, Decimal::new(100, 0));
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(order.is_active());
        assert!(!order.is_completed());
    }

    #[test]
    fn test_order_value_calculation() {
        let order = Order::new(
            "AAPL".to_string(),
            OrderType::Limit,
            OrderSide::Buy,
            Decimal::new(100, 0),
            Some(Decimal::new(15000, 2)),
            "test_account".to_string(),
        );

        let expected_value = Decimal::new(1500000, 2); // 100 * 150.00
        assert_eq!(order.value().unwrap(), expected_value);
    }

    #[test]
    fn test_position_update() {
        let mut position = Position::new(
            "test_account".to_string(),
            "AAPL".to_string(),
            Decimal::new(100, 0),
            Decimal::new(15000, 2),
        );

        let execution = Execution::new(
            Uuid::new_v4(),
            "AAPL".to_string(),
            OrderSide::Buy,
            Decimal::new(50, 0),
            Decimal::new(15200, 2),
            "NASDAQ".to_string(),
            Decimal::new(100, 2),
        );

        position.update_from_execution(&execution);

        assert_eq!(position.quantity, Decimal::new(150, 0));
        assert_eq!(position.average_price.round_dp(2), Decimal::new(15067, 2)); // Weighted average rounded
    }

    #[test]
    fn test_portfolio_operations() {
        let mut portfolio = Portfolio::new("test_account".to_string());

        let execution = Execution::new(
            Uuid::new_v4(),
            "AAPL".to_string(),
            OrderSide::Buy,
            Decimal::new(100, 0),
            Decimal::new(15000, 2),
            "NASDAQ".to_string(),
            Decimal::new(100, 2),
        );

        portfolio.update_from_execution(&execution);

        assert_eq!(portfolio.positions.len(), 1);
        assert!(portfolio.get_position(&"AAPL".to_string()).is_some());
    }
}

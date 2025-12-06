use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fmt;
use tokio::sync::RwLock;

use crate::error::{TradingError, TradingResult};
use crate::types::{
    AccountId, Execution, Order, OrderId, OrderSide, OrderStatus, OrderType, Symbol,
};

/// Order management system for handling order lifecycle, validation, and execution.
///
/// The OrderManager is responsible for:
/// - Order validation and creation
/// - Order state management and updates
/// - Order matching and execution
/// - Order persistence and retrieval
/// - Risk checks and compliance
pub struct OrderManager {
    /// Active orders indexed by order ID
    orders: RwLock<HashMap<OrderId, Order>>,

    /// Order book for matching orders
    order_book: RwLock<OrderBook>,

    /// Risk manager for validation
    risk_manager: Box<dyn RiskValidator + Send + Sync>,

    /// Fee calculator for execution costs
    fee_calculator: Box<dyn FeeCalculator + Send + Sync>,
}

impl fmt::Debug for OrderManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderManager").finish_non_exhaustive()
    }
}

impl OrderManager {
    /// Creates a new OrderManager with the specified risk manager and fee calculator
    pub fn new(
        risk_manager: Box<dyn RiskValidator + Send + Sync>,
        fee_calculator: Box<dyn FeeCalculator + Send + Sync>,
    ) -> Self {
        Self {
            orders: RwLock::new(HashMap::new()),
            order_book: RwLock::new(OrderBook::new()),
            risk_manager,
            fee_calculator,
        }
    }

    /// Submits a new order for processing
    pub async fn submit_order(
        &self,
        symbol: Symbol,
        order_type: OrderType,
        side: OrderSide,
        quantity: Decimal,
        price: Option<Decimal>,
        account_id: AccountId,
    ) -> TradingResult<OrderId> {
        // Create the order
        let order = Order::new(symbol, order_type, side, quantity, price, account_id);

        // Validate the order
        self.validate_order(&order).await?;

        // Store the order
        let order_id = order.id;
        let mut orders = self.orders.write().await;
        orders.insert(order_id, order);

        // Add to order book if it's a limit order
        if matches!(order_type, OrderType::Limit) {
            let mut order_book = self.order_book.write().await;
            order_book.add_order(order_id, &orders[&order_id]);
        }

        Ok(order_id)
    }

    /// Cancels an existing order
    pub async fn cancel_order(&self, order_id: OrderId) -> TradingResult<()> {
        let mut orders = self.orders.write().await;

        if let Some(order) = orders.get_mut(&order_id) {
            if !order.is_active() {
                return Err(TradingError::OrderValidation(format!(
                    "Order {} is not in an active state",
                    order_id
                )));
            }

            order.status = OrderStatus::Cancelled;

            // Remove from order book if it's a limit order
            if matches!(order.order_type, OrderType::Limit) {
                let mut order_book = self.order_book.write().await;
                order_book.remove_order(order_id);
            }

            Ok(())
        } else {
            Err(TradingError::OrderNotFound(order_id.to_string()))
        }
    }

    /// Gets an order by ID
    pub async fn get_order(&self, order_id: OrderId) -> TradingResult<Order> {
        let orders = self.orders.read().await;
        orders
            .get(&order_id)
            .cloned()
            .ok_or_else(|| TradingError::OrderNotFound(order_id.to_string()))
    }

    /// Lists all orders for an account
    pub async fn list_orders(&self, account_id: AccountId) -> TradingResult<Vec<Order>> {
        let orders = self.orders.read().await;
        let account_orders: Vec<Order> = orders
            .values()
            .filter(|order| order.account_id == account_id)
            .cloned()
            .collect();
        Ok(account_orders)
    }

    /// Processes market data updates and executes matching orders
    pub async fn process_market_data(
        &self,
        symbol: Symbol,
        price: Decimal,
    ) -> TradingResult<Vec<Execution>> {
        let mut executions = Vec::new();

        // Check for limit orders that can be executed at the new price
        let mut orders_to_remove = Vec::new();
        {
            let order_book = self.order_book.write().await;
            let mut orders = self.orders.write().await;

            // Get matching orders for this symbol
            let matching_orders = order_book.get_matching_orders(symbol.clone(), price);

            for order_id in matching_orders {
                if let Some(order) = orders.get_mut(&order_id) {
                    // Execute the order
                    let execution = self.execute_order(order, price).await?;
                    executions.push(execution);

                    // Check if order is fully filled
                    if order.status == OrderStatus::Filled {
                        orders_to_remove.push(order_id);
                    }
                }
            }
        }

        // Clean up filled orders from order book
        for order_id in orders_to_remove {
            let mut order_book = self.order_book.write().await;
            order_book.remove_order(order_id);
        }

        Ok(executions)
    }

    /// Validates an order before submission
    async fn validate_order(&self, order: &Order) -> TradingResult<()> {
        // Basic validation
        if order.quantity <= Decimal::ZERO {
            return Err(TradingError::OrderValidation(
                "Order quantity must be positive".into(),
            ));
        }

        if let Some(price) = order.price {
            if price <= Decimal::ZERO {
                return Err(TradingError::OrderValidation(
                    "Order price must be positive".into(),
                ));
            }
        }

        if order.order_type.requires_price() && order.price.is_none() {
            return Err(TradingError::OrderValidation(format!(
                "{:?} orders must have a price",
                order.order_type
            )));
        }

        // Risk validation
        let account_orders = self.list_orders(order.account_id.clone()).await?;
        let orders_slice: &[Order] = &account_orders;
        self.risk_manager
            .validate_order(order, orders_slice)
            .await?;

        Ok(())
    }

    /// Executes an order at the given price
    async fn execute_order(&self, order: &mut Order, price: Decimal) -> TradingResult<Execution> {
        // Calculate fees
        let fees = self.fee_calculator.calculate_fees(order, price);

        // Create execution record
        let execution = Execution::new(
            order.id,
            order.symbol.clone(),
            order.side,
            order.quantity,
            price,
            "SIMULATED".to_string(), // TODO: Get actual exchange
            fees,
        );

        // Update order status
        order.status = OrderStatus::Filled;

        Ok(execution)
    }
}

/// Order book for matching buy and sell orders
#[derive(Debug)]
pub struct OrderBook {
    /// Buy orders sorted by price (highest first)
    buy_orders: HashMap<Symbol, Vec<(OrderId, Decimal)>>,

    /// Sell orders sorted by price (lowest first)
    sell_orders: HashMap<Symbol, Vec<(OrderId, Decimal)>>,
}

impl OrderBook {
    /// Creates a new empty order book
    pub fn new() -> Self {
        Self {
            buy_orders: HashMap::new(),
            sell_orders: HashMap::new(),
        }
    }

    /// Adds an order to the order book
    pub fn add_order(&mut self, order_id: OrderId, order: &Order) {
        if let Some(price) = order.price {
            let orders = match order.side {
                OrderSide::Buy => &mut self.buy_orders,
                OrderSide::Sell => &mut self.sell_orders,
            };

            orders
                .entry(order.symbol.clone())
                .or_insert_with(Vec::new)
                .push((order_id, price));
        }
    }

    /// Removes an order from the order book
    pub fn remove_order(&mut self, order_id: OrderId) {
        for orders in self.buy_orders.values_mut() {
            orders.retain(|(id, _)| *id != order_id);
        }

        for orders in self.sell_orders.values_mut() {
            orders.retain(|(id, _)| *id != order_id);
        }
    }

    /// Gets orders that can be matched at the given price
    pub fn get_matching_orders(&self, symbol: Symbol, price: Decimal) -> Vec<OrderId> {
        let mut matching_orders = Vec::new();

        // Check buy orders (match if price >= order price)
        if let Some(buy_orders) = self.buy_orders.get(&symbol) {
            for (order_id, order_price) in buy_orders {
                if price <= *order_price {
                    matching_orders.push(*order_id);
                }
            }
        }

        // Check sell orders (match if price <= order price)
        if let Some(sell_orders) = self.sell_orders.get(&symbol) {
            for (order_id, order_price) in sell_orders {
                if price >= *order_price {
                    matching_orders.push(*order_id);
                }
            }
        }

        matching_orders
    }
}

/// Trait for order validation and risk management
#[async_trait]
pub trait RiskValidator: Send + Sync {
    /// Validates an order against risk limits
    async fn validate_order(&self, order: &Order, existing_orders: &[Order]) -> TradingResult<()>;
}

/// Default implementation of risk validator
pub struct DefaultRiskValidator {
    /// Maximum order size per symbol
    max_order_size: Decimal,

    /// Maximum position size per symbol
    max_position_size: Decimal,

    /// Maximum portfolio exposure
    max_portfolio_exposure: Decimal,
}

impl DefaultRiskValidator {
    /// Creates a new default risk validator
    pub fn new(
        max_order_size: Decimal,
        max_position_size: Decimal,
        max_portfolio_exposure: Decimal,
    ) -> Self {
        Self {
            max_order_size,
            max_position_size,
            max_portfolio_exposure,
        }
    }
}

#[async_trait]
impl RiskValidator for DefaultRiskValidator {
    async fn validate_order(&self, order: &Order, existing_orders: &[Order]) -> TradingResult<()> {
        // Check order size limits
        if order.quantity > self.max_order_size {
            return Err(TradingError::OrderValidation(format!(
                "Order size {} exceeds maximum allowed {}",
                order.quantity, self.max_order_size
            )));
        }

        // Calculate current exposure for this symbol
        let symbol_exposure: Decimal = existing_orders
            .iter()
            .filter(|o| o.symbol == order.symbol && o.is_active())
            .map(|o| o.quantity)
            .sum();

        if symbol_exposure + order.quantity > self.max_position_size {
            return Err(TradingError::OrderValidation(format!(
                "Position size would exceed limit for symbol {}",
                order.symbol
            )));
        }

        // Calculate total portfolio exposure
        let total_exposure: Decimal = existing_orders
            .iter()
            .filter(|o| o.is_active())
            .map(|o| o.quantity)
            .sum();

        if total_exposure + order.quantity > self.max_portfolio_exposure {
            return Err(TradingError::OrderValidation(
                "Portfolio exposure would exceed maximum allowed".into(),
            ));
        }

        Ok(())
    }
}

/// Trait for fee calculation
pub trait FeeCalculator: Send + Sync {
    /// Calculates fees for an order execution
    fn calculate_fees(&self, order: &Order, execution_price: Decimal) -> Decimal;
}

/// Default fee calculator implementation
pub struct DefaultFeeCalculator {
    /// Maker fee rate (can be negative for rebates)
    maker_fee_rate: Decimal,

    /// Taker fee rate
    taker_fee_rate: Decimal,
}

impl DefaultFeeCalculator {
    /// Creates a new default fee calculator
    pub fn new(maker_fee_rate: Decimal, taker_fee_rate: Decimal) -> Self {
        Self {
            maker_fee_rate,
            taker_fee_rate,
        }
    }
}

impl FeeCalculator for DefaultFeeCalculator {
    fn calculate_fees(&self, order: &Order, execution_price: Decimal) -> Decimal {
        let order_value = order.quantity * execution_price;

        // Use maker fees for limit orders, taker fees for market orders
        let fee_rate = if matches!(order.order_type, OrderType::Limit) {
            self.maker_fee_rate
        } else {
            self.taker_fee_rate
        };

        order_value * fee_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_order_submission() {
        let risk_manager = Box::new(DefaultRiskValidator::new(
            Decimal::new(1000, 0),
            Decimal::new(5000, 0),
            Decimal::new(10000, 0),
        ));

        let fee_calculator = Box::new(DefaultFeeCalculator::new(
            Decimal::new(-1, 4), // -0.0001 (0.01% rebate)
            Decimal::new(1, 3),  // 0.001 (0.1%)
        ));

        let order_manager = OrderManager::new(risk_manager, fee_calculator);

        let order_id = order_manager
            .submit_order(
                "AAPL".to_string(),
                OrderType::Limit,
                OrderSide::Buy,
                Decimal::new(100, 0),
                Some(Decimal::new(15000, 2)),
                "test_account".to_string(),
            )
            .await
            .unwrap();

        let order = order_manager.get_order(order_id).await.unwrap();
        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.status, OrderStatus::Pending);
    }

    #[tokio::test]
    async fn test_order_cancellation() {
        let risk_manager = Box::new(DefaultRiskValidator::new(
            Decimal::new(1000, 0),
            Decimal::new(5000, 0),
            Decimal::new(10000, 0),
        ));

        let fee_calculator = Box::new(DefaultFeeCalculator::new(
            Decimal::new(-1, 4),
            Decimal::new(1, 3),
        ));

        let order_manager = OrderManager::new(risk_manager, fee_calculator);

        let order_id = order_manager
            .submit_order(
                "AAPL".to_string(),
                OrderType::Limit,
                OrderSide::Buy,
                Decimal::new(100, 0),
                Some(Decimal::new(15000, 2)),
                "test_account".to_string(),
            )
            .await
            .unwrap();

        order_manager.cancel_order(order_id).await.unwrap();

        let order = order_manager.get_order(order_id).await.unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_risk_validation() {
        let risk_manager = Box::new(DefaultRiskValidator::new(
            Decimal::new(100, 0),  // Max order size
            Decimal::new(500, 0),  // Max position size
            Decimal::new(1000, 0), // Max portfolio exposure
        ));

        let order = Order::new(
            "AAPL".to_string(),
            OrderType::Limit,
            OrderSide::Buy,
            Decimal::new(200, 0), // This exceeds the max order size
            Some(Decimal::new(15000, 2)),
            "test_account".to_string(),
        );

        let result = risk_manager.validate_order(&order, &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fee_calculation() {
        let fee_calculator = DefaultFeeCalculator::new(
            Decimal::new(-1, 4), // -0.0001 (0.01% rebate)
            Decimal::new(1, 3),  // 0.001 (0.1%)
        );

        let order = Order::new(
            "AAPL".to_string(),
            OrderType::Limit, // Should use maker fees
            OrderSide::Buy,
            Decimal::new(100, 0),
            Some(Decimal::new(15000, 2)),
            "test_account".to_string(),
        );

        let fees = fee_calculator.calculate_fees(&order, Decimal::new(15000, 2));
        let expected_fees = Decimal::new(1500000, 2) * Decimal::new(-1, 4); // Negative = rebate
        assert_eq!(fees, expected_fees);
    }
}

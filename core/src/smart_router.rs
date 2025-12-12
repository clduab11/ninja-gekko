use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::error::{TradingError, TradingResult};
use crate::types::{Execution, MarketData, Order, Symbol, TradingPlatform};

/// Smart Order Router for optimal venue selection and execution.
///
/// The SmartOrderRouter analyzes multiple trading venues and selects the optimal
/// execution venue based on:
/// - Order characteristics (size, type, urgency)
/// - Venue liquidity and depth
/// - Transaction fees and costs
/// - Execution speed and reliability
/// - Market conditions and spreads
/// - Regulatory and compliance factors
#[derive(Debug)]
pub struct SmartOrderRouter {
    /// Available trading platforms/venues
    platforms: RwLock<Vec<TradingPlatform>>,

    /// Market data for all symbols
    market_data: RwLock<HashMap<Symbol, MarketData>>,

    /// Routing rules and strategies
    routing_rules: RwLock<RoutingRules>,

    /// Performance metrics for venues
    venue_metrics: RwLock<HashMap<String, VenueMetrics>>,
}

impl SmartOrderRouter {
    /// Creates a new SmartOrderRouter with empty platform list
    pub fn new() -> Self {
        Self {
            platforms: RwLock::new(Vec::new()),
            market_data: RwLock::new(HashMap::new()),
            routing_rules: RwLock::new(RoutingRules::default()),
            venue_metrics: RwLock::new(HashMap::new()),
        }
    }

    /// Adds a trading platform to the router
    pub async fn add_platform(&self, platform: TradingPlatform) {
        let mut platforms = self.platforms.write().await;
        platforms.push(platform);
    }

    /// Removes a trading platform from the router
    pub async fn remove_platform(&self, platform_id: &str) -> TradingResult<()> {
        let mut platforms = self.platforms.write().await;
        let initial_len = platforms.len();

        platforms.retain(|p| p.id != platform_id);

        if platforms.len() == initial_len {
            return Err(TradingError::PlatformNotFound(platform_id.to_string()));
        }

        Ok(())
    }

    /// Updates market data for a symbol
    pub async fn update_market_data(&self, market_data: MarketData) {
        let mut data = self.market_data.write().await;
        data.insert(market_data.symbol.clone(), market_data);
    }

    /// Routes an order to the optimal venue for execution
    pub async fn route_order(&self, order: &mut Order) -> TradingResult<ExecutionResult> {
        // Get available platforms for this symbol
        let platforms = self.get_available_platforms(&order.symbol).await?;

        if platforms.is_empty() {
            return Err(TradingError::NoAvailablePlatforms(order.symbol.clone()));
        }

        // Score each platform based on order characteristics
        let mut scored_platforms = Vec::new();
        for platform in &platforms {
            let score = self.score_platform(platform, order).await?;
            scored_platforms.push((platform, score));
        }

        // Sort by score (highest first)
        scored_platforms.sort_by(|a, b| {
            b.1.total_score
                .partial_cmp(&a.1.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select the best platform
        let (selected_platform, score) = &scored_platforms[0];

        // Update venue metrics
        self.update_venue_metrics(selected_platform, score.total_score)
            .await;

        // Execute the order on the selected platform
        let execution = self.execute_on_platform(selected_platform, order).await?;

        Ok(ExecutionResult {
            platform_id: selected_platform.id.clone(),
            platform_name: selected_platform.name.clone(),
            execution,
            routing_score: score.total_score,
            alternatives: scored_platforms
                .into_iter()
                .skip(1)
                .map(|(p, s)| (p.id.clone(), s.total_score))
                .collect(),
        })
    }

    /// Gets available platforms that support the given symbol
    async fn get_available_platforms(
        &self,
        symbol: &Symbol,
    ) -> TradingResult<Vec<TradingPlatform>> {
        let platforms = self.platforms.read().await;
        let available_platforms: Vec<TradingPlatform> = platforms
            .iter()
            .filter(|p| p.supported_symbols.contains(symbol) && p.connected)
            .cloned()
            .collect();

        if available_platforms.is_empty() {
            return Err(TradingError::NoAvailablePlatforms(symbol.clone()));
        }

        Ok(available_platforms)
    }

    /// Scores a platform for order execution
    async fn score_platform(
        &self,
        platform: &TradingPlatform,
        order: &Order,
    ) -> TradingResult<PlatformScore> {
        let routing_rules = self.routing_rules.read().await;
        let market_data = self.market_data.read().await;
        let venue_metrics = self.venue_metrics.read().await;

        // Base scoring components
        let mut score_components = PlatformScoreComponents::default();

        // 1. Liquidity scoring (40% weight)
        if let Some(data) = market_data.get(&order.symbol) {
            score_components.liquidity_score = self.score_liquidity(platform, order, data).await?;
        }

        // 2. Cost scoring (30% weight)
        score_components.cost_score = self.score_costs(platform, order)?;

        // 3. Speed scoring (15% weight)
        if let Some(metrics) = venue_metrics.get(&platform.id) {
            score_components.speed_score = self.score_speed(metrics);
        }

        // 4. Reliability scoring (15% weight)
        if let Some(metrics) = venue_metrics.get(&platform.id) {
            score_components.reliability_score = self.score_reliability(metrics);
        }

        // Apply routing rules
        routing_rules.apply_rules(&mut score_components, order);

        // Calculate total weighted score
        let total_score = score_components.calculate_total_score();

        Ok(PlatformScore {
            total_score,
            components: score_components,
        })
    }

    /// Scores platform liquidity based on order book depth and volume
    async fn score_liquidity(
        &self,
        platform: &TradingPlatform,
        order: &Order,
        market_data: &MarketData,
    ) -> TradingResult<Decimal> {
        // Higher volume and tighter spreads indicate better liquidity
        let volume_score = (market_data.volume_24h / Decimal::new(1000000, 0)) // Normalize to millions
            .min(Decimal::new(1, 0)); // Cap at 1.0

        let spread_score = if market_data.spread() > Decimal::ZERO {
            Decimal::new(1, 0) / (Decimal::new(1, 0) + market_data.spread())
        } else {
            Decimal::new(1, 0)
        };

        // Weight volume more heavily than spread (70/30)
        let liquidity_score = (volume_score * Decimal::new(7, 1)
            + spread_score * Decimal::new(3, 1))
            / Decimal::new(10, 0);

        Ok(liquidity_score)
    }

    /// Scores platform costs including fees and slippage
    fn score_costs(&self, platform: &TradingPlatform, order: &Order) -> TradingResult<Decimal> {
        // Calculate effective fees based on order type
        let fee_rate = if matches!(order.order_type, crate::types::OrderType::Limit) {
            platform.fee_structure.maker_fee
        } else {
            platform.fee_structure.taker_fee
        };

        // Lower fees = higher score (inverse relationship)
        // Assume 0.002 (0.2%) as baseline fee rate
        let baseline_fee = Decimal::new(2, 3); // 0.002
        let cost_score = if fee_rate < Decimal::ZERO {
            // Rebates (negative fees) get maximum score
            Decimal::new(1, 0)
        } else {
            // Positive fees score inversely to fee rate
            (baseline_fee / (baseline_fee + fee_rate)).max(Decimal::new(1, 1)) // Minimum score of 0.1
        };

        Ok(cost_score)
    }

    /// Scores platform execution speed
    fn score_speed(&self, metrics: &VenueMetrics) -> Decimal {
        // Score based on average execution time (lower is better)
        let avg_time_ms = metrics.average_execution_time;
        if avg_time_ms <= 0.0 {
            return Decimal::new(1, 0); // Perfect score if no data
        }

        // Score = 1000ms / (1000ms + execution_time)
        // This gives score of 0.5 for 1000ms execution time, approaches 1.0 for faster execution
        Decimal::new(1000, 0) / (Decimal::new(1000, 0) + Decimal::new(avg_time_ms as i64, 0))
    }

    /// Scores platform reliability based on historical performance
    fn score_reliability(&self, metrics: &VenueMetrics) -> Decimal {
        // Score based on success rate (higher is better)
        let success_rate = metrics.success_rate;
        if success_rate >= 1.0 {
            Decimal::new(1, 0)
        } else if success_rate <= 0.0 {
            Decimal::ZERO
        } else {
            Decimal::new(success_rate as i64, 0)
        }
    }

    /// Updates performance metrics for a venue
    async fn update_venue_metrics(&self, platform: &TradingPlatform, score: Decimal) {
        let mut metrics = self.venue_metrics.write().await;
        let venue_metrics = metrics
            .entry(platform.id.clone())
            .or_insert_with(VenueMetrics::default);

        // Update running average score
        let alpha = Decimal::new(1, 1); // Learning rate (0.1)
        venue_metrics.average_score =
            venue_metrics.average_score * (Decimal::new(1, 0) - alpha) + score * alpha;

        // Update execution count
        venue_metrics.execution_count += 1;
    }

    /// Executes an order on the specified platform
    async fn execute_on_platform(
        &self,
        platform: &TradingPlatform,
        order: &mut Order,
    ) -> TradingResult<Execution> {
        // This is a placeholder for actual platform execution
        // In a real implementation, this would connect to the exchange API

        // For simulation purposes, we'll create a mock execution
        let execution_price = if let Some(price) = order.price {
            price
        } else {
            // Use market price if available
            if let Some(market_data) = self.market_data.read().await.get(&order.symbol) {
                market_data.mid_price()
            } else {
                // Fallback to a reasonable price
                Decimal::new(10000, 2) // $100.00
            }
        };

        let fees = platform.fee_structure.taker_fee * order.quantity * execution_price;

        Ok(Execution::new(
            order.id,
            order.symbol.clone(),
            order.side,
            order.quantity,
            execution_price,
            platform.id.clone(),
            fees,
        ))
    }

    /// Gets current market data for a symbol
    pub async fn get_market_data(&self, symbol: &Symbol) -> Option<MarketData> {
        let data = self.market_data.read().await;
        data.get(symbol).cloned()
    }

    /// Gets available platforms
    pub async fn get_platforms(&self) -> Vec<TradingPlatform> {
        let platforms = self.platforms.read().await;
        platforms.clone()
    }

    /// Gets routing rules
    pub async fn get_routing_rules(&self) -> RoutingRules {
        let rules = self.routing_rules.read().await;
        rules.clone()
    }

    /// Updates routing rules
    pub async fn update_routing_rules(&self, rules: RoutingRules) {
        let mut current_rules = self.routing_rules.write().await;
        *current_rules = rules;
    }
}

/// Result of order routing and execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// ID of the selected platform
    pub platform_id: String,

    /// Name of the selected platform
    pub platform_name: String,

    /// Execution details
    pub execution: Execution,

    /// Score that determined platform selection
    pub routing_score: Decimal,

    /// Alternative platforms with their scores
    pub alternatives: Vec<(String, Decimal)>,
}

/// Result of arbitrage opportunity routing
#[derive(Debug, Clone)]
pub struct ArbitrageExecutionPlan {
    pub opportunity_id: uuid::Uuid,
    pub buy_platform: TradingPlatform,
    pub sell_platform: TradingPlatform,
    pub execution_size: Decimal,
    pub estimated_profit: Decimal,
    pub risk_score: f64,
    pub execution_deadline: chrono::DateTime<chrono::Utc>,
    pub contingency_plans: Vec<ContingencyPlan>,
}

/// Contingency plan for arbitrage execution
#[derive(Debug, Clone)]
pub struct ContingencyPlan {
    pub trigger_condition: String,
    pub alternative_action: String,
    pub fallback_platforms: Vec<TradingPlatform>,
}

/// Platform scoring components
#[derive(Debug, Clone)]
pub struct PlatformScoreComponents {
    pub liquidity_score: Decimal,
    pub cost_score: Decimal,
    pub speed_score: Decimal,
    pub reliability_score: Decimal,
}

impl Default for PlatformScoreComponents {
    fn default() -> Self {
        Self {
            liquidity_score: Decimal::ZERO,
            cost_score: Decimal::ZERO,
            speed_score: Decimal::ZERO,
            reliability_score: Decimal::ZERO,
        }
    }
}

impl PlatformScoreComponents {
    /// Calculates the total weighted score
    pub fn calculate_total_score(&self) -> Decimal {
        // Weighted scoring: Liquidity 40%, Cost 30%, Speed 15%, Reliability 15%
        self.liquidity_score * Decimal::new(4, 1)
            + self.cost_score * Decimal::new(3, 1)
            + self.speed_score * Decimal::new(15, 2)
            + self.reliability_score * Decimal::new(15, 2)
    }
}

/// Final platform score
#[derive(Debug, Clone)]
pub struct PlatformScore {
    /// Total weighted score
    pub total_score: Decimal,

    /// Individual component scores
    pub components: PlatformScoreComponents,
}

/// Routing rules for customizing venue selection
#[derive(Debug, Clone, Default)]
pub struct RoutingRules {
    /// Platform preferences by symbol
    platform_preferences: HashMap<Symbol, Vec<String>>,

    /// Cost sensitivity (0-1, higher = more cost-sensitive)
    cost_sensitivity: Decimal,

    /// Speed sensitivity (0-1, higher = more speed-sensitive)
    speed_sensitivity: Decimal,

    /// Minimum score threshold for venue selection
    minimum_score_threshold: Decimal,
}

impl RoutingRules {
    /// Creates new routing rules with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets platform preferences for a symbol (ordered by preference)
    pub fn set_platform_preferences(&mut self, symbol: Symbol, platforms: Vec<String>) {
        self.platform_preferences.insert(symbol, platforms);
    }

    /// Sets cost sensitivity
    pub fn set_cost_sensitivity(&mut self, sensitivity: Decimal) {
        self.cost_sensitivity = sensitivity.min(Decimal::new(1, 0)).max(Decimal::ZERO);
    }

    /// Sets speed sensitivity
    pub fn set_speed_sensitivity(&mut self, sensitivity: Decimal) {
        self.speed_sensitivity = sensitivity.min(Decimal::new(1, 0)).max(Decimal::ZERO);
    }

    /// Sets minimum score threshold
    pub fn set_minimum_score_threshold(&mut self, threshold: Decimal) {
        self.minimum_score_threshold = threshold.min(Decimal::new(1, 0)).max(Decimal::ZERO);
    }

    /// Applies routing rules to platform scoring
    pub fn apply_rules(&self, components: &mut PlatformScoreComponents, order: &Order) {
        // Apply symbol-specific platform preferences
        if let Some(preferred_platforms) = self.platform_preferences.get(&order.symbol) {
            // This would be implemented to boost scores of preferred platforms
            // For now, we'll leave this as a placeholder for future enhancement
        }

        // Apply cost sensitivity adjustments
        if self.cost_sensitivity > Decimal::new(5, 1) {
            // High cost sensitivity
            components.cost_score = components.cost_score * Decimal::new(12, 1);
            // Boost cost scoring
        }

        // Apply speed sensitivity adjustments
        if self.speed_sensitivity > Decimal::new(5, 1) {
            // High speed sensitivity
            components.speed_score = components.speed_score * Decimal::new(12, 1);
            // Boost speed scoring
        }

        // Ensure scores stay within bounds
        components.liquidity_score = components.liquidity_score.min(Decimal::new(1, 0));
        components.cost_score = components.cost_score.min(Decimal::new(1, 0));
        components.speed_score = components.speed_score.min(Decimal::new(1, 0));
        components.reliability_score = components.reliability_score.min(Decimal::new(1, 0));
    }
}

/// Performance metrics for trading venues
#[derive(Debug, Clone, Default)]
pub struct VenueMetrics {
    /// Average execution time in milliseconds
    pub average_execution_time: f64,

    /// Success rate (0-1)
    pub success_rate: f64,

    /// Total number of executions
    pub execution_count: u64,

    /// Average routing score
    pub average_score: Decimal,

    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl VenueMetrics {
    /// Updates execution time with new measurement
    pub fn update_execution_time(&mut self, execution_time_ms: f64) {
        // Calculate running average
        let total_time = self.average_execution_time * self.execution_count as f64;
        self.execution_count += 1;
        self.average_execution_time =
            (total_time + execution_time_ms) / self.execution_count as f64;
        self.last_update = Utc::now();
    }

    /// Updates success rate
    pub fn update_success_rate(&mut self, success: bool) {
        let current_rate = self.success_rate;
        let total_attempts = self.execution_count;
        self.execution_count += 1;

        // Calculate new success rate
        self.success_rate = (current_rate * (total_attempts - 1) as f64
            + if success { 1.0 } else { 0.0 })
            / self.execution_count as f64;
        self.last_update = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_smart_order_router_creation() {
        let router = SmartOrderRouter::new();
        assert!(router.get_platforms().await.is_empty());
    }

    #[tokio::test]
    async fn test_platform_management() {
        let router = SmartOrderRouter::new();

        let platform = TradingPlatform {
            id: "test_platform".to_string(),
            name: "Test Exchange".to_string(),
            supported_symbols: vec!["AAPL".to_string()],
            fee_structure: crate::types::FeeStructure::default(),
            rate_limits: crate::types::RateLimits::default(),
            connected: true,
            metadata: HashMap::new(),
        };

        router.add_platform(platform).await;
        let platforms = router.get_platforms().await;

        assert_eq!(platforms.len(), 1);
        assert_eq!(platforms[0].id, "test_platform");
    }

    #[tokio::test]
    async fn test_market_data_update() {
        let router = SmartOrderRouter::new();

        let market_data = MarketData::new(
            "AAPL".to_string(),
            Decimal::new(15000, 2),   // $150.00 bid
            Decimal::new(15002, 2),   // $150.02 ask
            Decimal::new(15001, 2),   // $150.01 last
            Decimal::new(1000000, 0), // 1M volume
        );

        router.update_market_data(market_data).await;
        let retrieved_data = router.get_market_data(&"AAPL".to_string()).await.unwrap();

        assert_eq!(retrieved_data.symbol, "AAPL");
        assert_eq!(retrieved_data.bid, Decimal::new(15000, 2));
    }

    #[tokio::test]
    async fn test_routing_rules() {
        let mut rules = RoutingRules::new();
        rules.set_cost_sensitivity(Decimal::new(8, 1)); // High cost sensitivity
        rules.set_platform_preferences("AAPL".to_string(), vec!["platform1".to_string()]);

        assert_eq!(rules.cost_sensitivity, Decimal::new(8, 1));
        assert_eq!(
            rules
                .platform_preferences
                .get(&"AAPL".to_string())
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_platform_score_calculation() {
        let mut components = PlatformScoreComponents::default();
        components.liquidity_score = Decimal::new(8, 1); // 0.8
        components.cost_score = Decimal::new(7, 1); // 0.7
        components.speed_score = Decimal::new(9, 1); // 0.9
        components.reliability_score = Decimal::new(6, 1); // 0.6

        let total_score = components.calculate_total_score();
        // Expected: (0.8 * 0.4) + (0.7 * 0.3) + (0.9 * 0.15) + (0.6 * 0.15)
        //         = 0.32 + 0.21 + 0.135 + 0.09 = 0.755
        assert_eq!(total_score, Decimal::new(755, 3));
    }
}

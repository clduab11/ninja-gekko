//! Momentum-based trading strategy
//!
//! This strategy generates signals when price momentum exceeds configurable thresholds.
//! Uses a simple rate-of-change approach to measure momentum.

use std::time::Instant;

use event_bus::{Priority, SignalEventPayload, StrategySignal};
use exchange_connectors::ExchangeId;
use ninja_gekko_core::types::{OrderSide, OrderType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{debug, info};
use uuid::Uuid;

use crate::traits::{
    MarketSnapshot, StrategyContext, StrategyDecision, StrategyError, StrategyExecutor,
    StrategyInitContext, StrategyMetrics,
};

/// Configuration for momentum strategy
#[derive(Debug, Clone)]
pub struct MomentumConfig {
    /// Number of snapshots to look back for momentum calculation
    pub lookback_periods: usize,
    /// Minimum momentum percentage to trigger a signal (e.g., 0.01 = 1%)
    pub momentum_threshold: Decimal,
    /// Base position size in units
    pub base_position_size: Decimal,
    /// Target exchange for orders
    pub target_exchange: ExchangeId,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            lookback_periods: 5,
            momentum_threshold: dec!(0.005), // 0.5% momentum threshold
            base_position_size: dec!(0.1),
            target_exchange: ExchangeId::BinanceUs,
        }
    }
}

/// Momentum-based trading strategy
///
/// Generates buy signals when prices show strong upward momentum
/// and sell signals when downward momentum exceeds threshold.
pub struct MomentumStrategy {
    name: String,
    strategy_id: Uuid,
    account_id: String,
    config: MomentumConfig,
    initialized: bool,
}

impl MomentumStrategy {
    /// Create a new momentum strategy with configuration
    pub fn new(name: impl Into<String>, config: MomentumConfig) -> Self {
        Self {
            name: name.into(),
            strategy_id: Uuid::new_v4(),
            account_id: String::new(),
            config,
            initialized: false,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(name: impl Into<String>) -> Self {
        Self::new(name, MomentumConfig::default())
    }

    /// Calculate momentum as rate of change
    fn calculate_momentum(&self, snapshots: &[MarketSnapshot]) -> Option<Decimal> {
        if snapshots.len() < 2 {
            return None;
        }

        let lookback = self.config.lookback_periods.min(snapshots.len() - 1);
        let current = snapshots.last()?;
        let previous = snapshots.get(snapshots.len().saturating_sub(lookback + 1))?;

        if previous.last == Decimal::ZERO {
            return None;
        }

        // Rate of change: (current - previous) / previous
        let change = (current.last - previous.last) / previous.last;
        Some(change)
    }

    /// Generate signal based on momentum
    fn generate_signal(&self, snapshot: &MarketSnapshot, momentum: Decimal) -> Option<StrategySignal> {
        let (side, confidence) = if momentum > self.config.momentum_threshold {
            // Strong upward momentum - buy signal
            let confidence = (momentum / self.config.momentum_threshold)
                .min(dec!(1.0))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.5);
            (OrderSide::Buy, confidence)
        } else if momentum < -self.config.momentum_threshold {
            // Strong downward momentum - sell signal
            let confidence = (-momentum / self.config.momentum_threshold)
                .min(dec!(1.0))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.5);
            (OrderSide::Sell, confidence)
        } else {
            // Momentum within threshold - no signal
            return None;
        };

        Some(StrategySignal {
            exchange: Some(self.config.target_exchange.clone()),
            symbol: snapshot.symbol.clone(),
            side,
            order_type: OrderType::Market,
            quantity: self.config.base_position_size,
            limit_price: None,
            confidence,
            metadata: Default::default(),
        })
    }
}

impl StrategyExecutor<8> for MomentumStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn initialize(&mut self, ctx: StrategyInitContext<'_>) -> Result<(), StrategyError> {
        self.strategy_id = ctx.strategy_id;
        self.account_id = ctx.account_id.to_string();
        self.initialized = true;
        info!(
            strategy = %self.name,
            id = %self.strategy_id,
            "Momentum strategy initialized"
        );
        Ok(())
    }

    fn evaluate(&mut self, ctx: StrategyContext<'_, 8>) -> Result<StrategyDecision, StrategyError> {
        let start = Instant::now();
        let mut signals = Vec::new();
        let mut logs = Vec::new();

        let snapshots = ctx.snapshots();
        
        if snapshots.is_empty() {
            return Ok(StrategyDecision {
                signals,
                logs: vec!["No market snapshots available".to_string()],
                metrics: StrategyMetrics {
                    evaluation_latency: start.elapsed(),
                },
            });
        }

        // Calculate momentum for primary symbol
        if let Some(momentum) = self.calculate_momentum(snapshots) {
            debug!(
                strategy = %self.name,
                momentum = %momentum,
                threshold = %self.config.momentum_threshold,
                "Calculated momentum"
            );

            if let Some(latest) = snapshots.last() {
                if let Some(signal) = self.generate_signal(latest, momentum) {
                    info!(
                        strategy = %self.name,
                        symbol = %signal.symbol,
                        side = ?signal.side,
                        quantity = %signal.quantity,
                        confidence = %signal.confidence,
                        "Generated momentum signal"
                    );

                    let payload = SignalEventPayload {
                        strategy_id: self.strategy_id,
                        account_id: self.account_id.clone(),
                        priority: if signal.confidence > 0.8 {
                            Priority::High
                        } else {
                            Priority::Normal
                        },
                        signal,
                    };
                    signals.push(payload);
                    logs.push(format!(
                        "Momentum {:.4} triggered signal",
                        momentum
                    ));
                } else {
                    logs.push(format!(
                        "Momentum {:.4} within threshold, no signal",
                        momentum
                    ));
                }
            }
        }

        Ok(StrategyDecision {
            signals,
            logs,
            metrics: StrategyMetrics {
                evaluation_latency: start.elapsed(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ninja_gekko_core::types::AccountId;
    use chrono::Utc;

    fn create_snapshots(prices: &[Decimal]) -> [MarketSnapshot; 8] {
        let mut arr: [MarketSnapshot; 8] = std::array::from_fn(|_| MarketSnapshot {
            symbol: "BTC-USD".to_string(),
            bid: dec!(0),
            ask: dec!(0),
            last: dec!(0),
            timestamp: Utc::now(),
        });
        
        for (i, price) in prices.iter().take(8).enumerate() {
            arr[i] = MarketSnapshot {
                symbol: "BTC-USD".to_string(),
                bid: *price - dec!(10),
                ask: *price + dec!(10),
                last: *price,
                timestamp: Utc::now(),
            };
        }
        arr
    }

    #[test]
    fn test_momentum_strategy_creation() {
        let strategy = MomentumStrategy::with_defaults("test-momentum");
        assert_eq!(strategy.name(), "test-momentum");
        assert!(!strategy.initialized);
    }

    #[test]
    fn test_momentum_calculation() {
        let strategy = MomentumStrategy::with_defaults("test");
        let prices = [
            dec!(100), dec!(101), dec!(102), dec!(103),
            dec!(104), dec!(105), dec!(106), dec!(107),
        ];
        let snapshots = create_snapshots(&prices);
        
        let momentum = strategy.calculate_momentum(&snapshots);
        assert!(momentum.is_some());
        // 7% increase over 5 periods
        let m = momentum.unwrap();
        assert!(m > dec!(0));
    }

    #[test]
    fn test_upward_momentum_generates_buy() {
        let config = MomentumConfig {
            momentum_threshold: dec!(0.01),
            ..Default::default()
        };
        let strategy = MomentumStrategy::new("test", config);
        
        let snapshot = MarketSnapshot {
            symbol: "BTC-USD".to_string(),
            bid: dec!(990),
            ask: dec!(1010),
            last: dec!(1000),
            timestamp: Utc::now(),
        };
        
        // 5% momentum > 1% threshold = buy
        let signal = strategy.generate_signal(&snapshot, dec!(0.05));
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().side, OrderSide::Buy);
    }

    #[test]
    fn test_downward_momentum_generates_sell() {
        let config = MomentumConfig {
            momentum_threshold: dec!(0.01),
            ..Default::default()
        };
        let strategy = MomentumStrategy::new("test", config);
        
        let snapshot = MarketSnapshot {
            symbol: "BTC-USD".to_string(),
            bid: dec!(990),
            ask: dec!(1010),
            last: dec!(1000),
            timestamp: Utc::now(),
        };
        
        // -5% momentum > 1% threshold = sell
        let signal = strategy.generate_signal(&snapshot, dec!(-0.05));
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().side, OrderSide::Sell);
    }

    #[test]
    fn test_no_signal_within_threshold() {
        let config = MomentumConfig {
            momentum_threshold: dec!(0.01),
            ..Default::default()
        };
        let strategy = MomentumStrategy::new("test", config);
        
        let snapshot = MarketSnapshot {
            symbol: "BTC-USD".to_string(),
            bid: dec!(990),
            ask: dec!(1010),
            last: dec!(1000),
            timestamp: Utc::now(),
        };
        
        // 0.5% momentum < 1% threshold = no signal
        let signal = strategy.generate_signal(&snapshot, dec!(0.005));
        assert!(signal.is_none());
    }
}

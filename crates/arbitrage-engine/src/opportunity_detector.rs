//! Opportunity Detector - AI-Powered Arbitrage Opportunity Detection
//!
//! This module implements sophisticated arbitrage opportunity detection using
//! AI/ML models to identify profitable cross-exchange trading opportunities.

use crate::{
    ArbitrageConfig, ArbitrageOpportunity, ArbitrageResult, ExecutionComplexity,
    TimeSensitivity,
};
use chrono::Utc;
use exchange_connectors::{ExchangeId, MarketTick};
use neural_engine::{MarketDataInput, NeuralEngine};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Thread-safe cache of latest market prices
pub type PriceCache = Arc<RwLock<HashMap<String, HashMap<ExchangeId, MarketTick>>>>;

/// Opportunity detector using AI/ML for arbitrage detection
pub struct OpportunityDetector {
    config: ArbitrageConfig,
    price_cache: PriceCache,
    neural_engine: Option<Arc<NeuralEngine>>,
}

impl OpportunityDetector {
    /// Create a new opportunity detector with optional neural engine for ML-powered scoring
    pub fn new(config: ArbitrageConfig, neural_engine: Option<Arc<NeuralEngine>>) -> Self {
        if neural_engine.is_some() {
            info!("🧠 OpportunityDetector initialized with NeuralEngine for ML-powered confidence scoring");
        } else {
            debug!("OpportunityDetector initialized without NeuralEngine (using default scoring)");
        }

        Self {
            config,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            neural_engine,
        }
    }

    /// Update the price cache with a new tick
    pub async fn update_price(&self, tick: MarketTick, exchange: ExchangeId) {
        let mut cache = self.price_cache.write().await;
        cache
            .entry(tick.symbol.clone())
            .or_default()
            .insert(exchange, tick);
    }

    /// Detect arbitrage opportunities across exchanges
    pub async fn detect_opportunities(&self) -> ArbitrageResult<Vec<ArbitrageOpportunity>> {
        let cache = self.price_cache.read().await;
        let mut opportunities = Vec::new();

        for (symbol, exchange_prices) in cache.iter() {
            if exchange_prices.len() < 2 {
                continue; // Need at least 2 exchanges to arbitrage
            }

            // Find best bid (highest price to sell) and best ask (lowest price to buy)
            let mut best_bid: Option<(&ExchangeId, &MarketTick)> = None;
            let mut best_ask: Option<(&ExchangeId, &MarketTick)> = None;

            for (exchange, tick) in exchange_prices {
                // Update best bid (highest bid price)
                if let Some((_, current_best)) = best_bid {
                    if tick.bid > current_best.bid {
                        best_bid = Some((exchange, tick));
                    }
                } else {
                    best_bid = Some((exchange, tick));
                }

                // Update best ask (lowest ask price)
                if let Some((_, current_best)) = best_ask {
                    if tick.ask < current_best.ask {
                        best_ask = Some((exchange, tick));
                    }
                } else {
                    best_ask = Some((exchange, tick));
                }
            }

            if let (Some((sell_exchange, sell_tick)), Some((buy_exchange, buy_tick))) =
                (best_bid, best_ask)
            {
                // Ensure we are not trading on the same exchange
                if sell_exchange == buy_exchange {
                    continue;
                }

                // Check for profitability
                let spread = sell_tick.bid - buy_tick.ask;

                if spread > Decimal::ZERO {
                    let gross_profit_percentage = (spread / buy_tick.ask).to_f64().unwrap_or(0.0);

                    // Estimate fees (conservative 0.1% per side = 0.2% total)
                    let estimated_fee_percentage = 0.002;
                    let net_profit_percentage = gross_profit_percentage - estimated_fee_percentage;

                    // Check if it meets minimum profit threshold
                    if net_profit_percentage >= self.config.min_profit_percentage / 100.0 {
                        let quantity = self.config.max_position_size;

                        // Calculate confidence score using NeuralEngine if available
                        let confidence_score = self
                            .calculate_confidence_score(symbol, buy_tick, sell_tick)
                            .await;

                        let opportunity = ArbitrageOpportunity {
                            id: Uuid::new_v4(),
                            symbol: symbol.clone(),
                            buy_exchange: *buy_exchange,
                            sell_exchange: *sell_exchange,
                            buy_price: buy_tick.ask,
                            sell_price: sell_tick.bid,
                            price_difference: spread,
                            profit_percentage: net_profit_percentage * 100.0,
                            estimated_profit: (spread * quantity)
                                * Decimal::from_f64_retain(1.0 - estimated_fee_percentage)
                                    .unwrap_or(Decimal::ONE),
                            confidence_score,
                            max_quantity: quantity,
                            time_sensitivity: TimeSensitivity::High,
                            risk_score: 0.1,
                            execution_complexity: ExecutionComplexity::Simple,
                            detected_at: Utc::now(),
                            expires_at: Utc::now() + chrono::Duration::seconds(10),
                        };

                        opportunities.push(opportunity);
                    }
                }
            }
        }

        if !opportunities.is_empty() {
            info!(
                "🎯 Detected {} arbitrage opportunities",
                opportunities.len()
            );
        }

        Ok(opportunities)
    }

    /// Calculate confidence score using NeuralEngine if available, otherwise use default
    async fn calculate_confidence_score(
        &self,
        symbol: &str,
        buy_tick: &MarketTick,
        sell_tick: &MarketTick,
    ) -> f64 {
        match &self.neural_engine {
            Some(engine) => {
                // Convert MarketTick to MarketDataInput for neural engine
                let buy_data = Self::tick_to_market_data(buy_tick);
                let sell_data = Self::tick_to_market_data(sell_tick);

                // Try to get prediction from neural engine
                match engine
                    .predict_cross_exchange_arbitrage(
                        symbol,
                        "primary",
                        "secondary",
                        &buy_data,
                        &sell_data,
                    )
                    .await
                {
                    Ok(prediction) => {
                        debug!(
                            "🧠 NeuralEngine confidence for {}: {:.2}%",
                            symbol,
                            prediction.confidence_score * 100.0
                        );
                        prediction.confidence_score
                    }
                    Err(e) => {
                        warn!(
                            "NeuralEngine prediction failed for {}, using default: {}",
                            symbol, e
                        );
                        self.default_confidence_score()
                    }
                }
            }
            None => self.default_confidence_score(),
        }
    }

    /// Default confidence score when NeuralEngine is not available
    fn default_confidence_score(&self) -> f64 {
        0.9 // Conservative default for Gordon Gekko mode
    }

    /// Convert MarketTick to MarketDataInput for neural engine compatibility
    fn tick_to_market_data(tick: &MarketTick) -> MarketDataInput {
        let price = tick.last.to_f64().unwrap_or(0.0);
        let bid = tick.bid.to_f64().unwrap_or(0.0);
        let ask = tick.ask.to_f64().unwrap_or(0.0);
        let volume = tick.volume_24h.to_f64().unwrap_or(0.0);

        // Estimate high/low from bid/ask spread
        let high = ask * 1.005; // +0.5%
        let low = bid * 0.995; // -0.5%

        MarketDataInput {
            price,
            high,
            low,
            volume,
            avg_volume: volume, // Same as current (no historical data)
            bid,
            ask,
            timestamp: tick.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_arbitrage_detection_without_neural_engine() {
        let config = ArbitrageConfig::default();
        let detector = OpportunityDetector::new(config, None);

        // Simulate prices
        let tick_a = MarketTick {
            symbol: "BTC-USD".into(),
            bid: dec!(99),
            ask: dec!(100),
            last: dec!(99.5),
            volume_24h: dec!(1000),
            timestamp: Utc::now(),
        };

        let tick_b = MarketTick {
            symbol: "BTC-USD".into(),
            bid: dec!(102),
            ask: dec!(103),
            last: dec!(102.5),
            volume_24h: dec!(1000),
            timestamp: Utc::now(),
        };

        detector.update_price(tick_a, ExchangeId::Coinbase).await;
        detector.update_price(tick_b, ExchangeId::BinanceUs).await;

        let opps = detector.detect_opportunities().await.unwrap();
        assert_eq!(opps.len(), 1);
        assert_eq!(opps[0].buy_exchange, ExchangeId::Coinbase);
        assert_eq!(opps[0].sell_exchange, ExchangeId::BinanceUs);
        assert_eq!(opps[0].price_difference, dec!(2));
        // Without NeuralEngine, should use default 0.9
        assert!((opps[0].confidence_score - 0.9).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_tick_to_market_data_conversion() {
        let tick = MarketTick {
            symbol: "ETH-USD".into(),
            bid: dec!(2000),
            ask: dec!(2005),
            last: dec!(2002.5),
            volume_24h: dec!(50000),
            timestamp: Utc::now(),
        };

        let market_data = OpportunityDetector::tick_to_market_data(&tick);

        assert!((market_data.price - 2002.5).abs() < 0.1);
        assert!((market_data.bid - 2000.0).abs() < 0.1);
        assert!((market_data.ask - 2005.0).abs() < 0.1);
        assert!((market_data.volume - 50000.0).abs() < 0.1);
    }
}

// Import ArbitrageConfig from parent module
use crate::ArbitrageConfig;

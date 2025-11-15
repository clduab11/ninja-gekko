---
name: Arbitrage Engine - Core Business Logic
about: Implement opportunity detection, risk scoring, and execution coordination
title: "[MILESTONE 2] Implement Arbitrage Detection & Execution Logic"
labels: critical, arbitrage, trading-logic, ai-agent
assignees: ''
---

## 🤖 GitHub Copilot Context Prompt

**Copy this into your Copilot chat before starting implementation:**

```
You are implementing MILESTONE 2: Arbitrage Detection & Execution Logic for a production Rust trading system. Follow November 2025 standards: Rust 2021 edition, tokio 1.0+ async, thiserror for errors, tracing for logging, no unsafe code. Implement volatility scanning with multi-timeframe analysis (1m/5m/15m windows), cross-exchange opportunity detection with confidence scoring (0-1 scale, 85%+ threshold), Kelly Criterion position sizing with constraints, simultaneous order execution coordination. Use Decimal type for all financial calculations, never f64. Every function needs rustdoc with examples. Risk management: enforce stop-loss (2%), circuit breakers (5 consecutive losses), daily loss limits ($5000), position exposure limits (15% per symbol). Event-driven: emit to event bus for all opportunities/executions. Performance: <100ms volatility scan, <50ms opportunity detection, <100ms execution. Testing: >80% coverage, unit tests for each component, integration tests for full arbitrage cycle, property-based tests with proptest for Kelly Criterion. Reference existing types in crates/arbitrage-engine/src/lib.rs and config/arbitrage.toml for parameters. Code must be production-grade: zero clippy warnings, formatted with cargo fmt, comprehensive error handling, audit logging. This is the core revenue feature—implement with precision and correctness.
```

---


## Overview

**Milestone**: Arbitrage Engine Implementation
**Priority**: CRITICAL - Core revenue generation feature
**Implementation Scope**: Core revenue generation - autonomous arbitrage trading
**Dependencies**: Issue #1 (Exchange API Integration)

## Problem Statement

The arbitrage engine has comprehensive type definitions, configuration, and infrastructure, but the actual opportunity detection algorithms, risk scoring, capital allocation, and execution coordination are stubbed with `// TODO` comments.

**Current State**:
- ✅ Module structure defined in `crates/arbitrage-engine/src/`
- ✅ Configuration loaded from `config/arbitrage.toml`
- ✅ Types defined (OpportunityData, VolatilityScore, ExecutionResult)
- ✅ Error handling infrastructure (`ArbitrageError`)
- ❌ Volatility scanning algorithm not implemented
- ❌ Cross-exchange opportunity detection returns empty Vec
- ❌ Capital allocation logic stubbed
- ❌ Execution engine returns mock results

**Target State**: Fully functional arbitrage system that detects profitable opportunities across exchanges, allocates capital intelligently, executes trades, and manages risk.

---

## Implementation Checklist

### Phase 1: Volatility Scanner

**Files to Modify**: `crates/arbitrage-engine/src/volatility_scanner.rs`

#### 1.1 Price Change Analysis

- [ ] **Implement windowed price change calculation**
  ```rust
  use std::collections::VecDeque;

  pub struct PriceWindow {
      prices: VecDeque<(Instant, Decimal)>,
      window_duration: Duration,
  }

  impl PriceWindow {
      pub fn add_price(&mut self, price: Decimal) {
          let now = Instant::now();
          self.prices.push_back((now, price));

          // Remove old prices outside window
          while let Some(&(timestamp, _)) = self.prices.front() {
              if now.duration_since(timestamp) > self.window_duration {
                  self.prices.pop_front();
              } else {
                  break;
              }
          }
      }

      pub fn calculate_change_percent(&self) -> Option<Decimal> {
          if self.prices.len() < 2 {
              return None;
          }

          let oldest = self.prices.front()?.1;
          let newest = self.prices.back()?.1;

          Some(((newest - oldest) / oldest) * Decimal::from(100))
      }

      pub fn calculate_volatility(&self) -> Option<Decimal> {
          if self.prices.len() < 2 {
              return None;
          }

          // Calculate standard deviation of returns
          let returns: Vec<Decimal> = self.prices
              .iter()
              .zip(self.prices.iter().skip(1))
              .map(|((_, p1), (_, p2))| (p2 - p1) / p1)
              .collect();

          let mean = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());
          let variance = returns.iter()
              .map(|r| (r - mean).powi(2))
              .sum::<Decimal>() / Decimal::from(returns.len());

          Some(variance.sqrt())
      }
  }
  ```

- [ ] **Implement multi-timeframe volatility analysis**
  ```rust
  pub struct VolatilityAnalyzer {
      short_window: PriceWindow,   // 1 minute
      medium_window: PriceWindow,  // 5 minutes
      long_window: PriceWindow,    // 15 minutes
  }

  impl VolatilityAnalyzer {
      pub fn analyze(&self) -> VolatilityMetrics {
          VolatilityMetrics {
              short_term_volatility: self.short_window.calculate_volatility(),
              medium_term_volatility: self.medium_window.calculate_volatility(),
              long_term_volatility: self.long_window.calculate_volatility(),
              price_momentum: self.calculate_momentum(),
          }
      }

      fn calculate_momentum(&self) -> Decimal {
          let short_change = self.short_window.calculate_change_percent().unwrap_or(Decimal::ZERO);
          let long_change = self.long_window.calculate_change_percent().unwrap_or(Decimal::ZERO);

          // Momentum score: positive if accelerating upward
          short_change - long_change
      }
  }
  ```

#### 1.2 Volume Analysis

- [ ] **Implement volume surge detection**
  ```rust
  pub struct VolumeAnalyzer {
      volume_history: VecDeque<(Instant, Decimal)>,
      baseline_window: Duration,
  }

  impl VolumeAnalyzer {
      pub fn detect_surge(&self, current_volume: Decimal) -> VolumeSurgeResult {
          let baseline_avg = self.calculate_baseline_average();

          let surge_ratio = if baseline_avg > Decimal::ZERO {
              current_volume / baseline_avg
          } else {
              Decimal::ONE
          };

          VolumeSurgeResult {
              is_surge: surge_ratio > Decimal::from_str("2.0").unwrap(),
              surge_multiplier: surge_ratio,
              volume_score: self.normalize_volume_score(surge_ratio),
          }
      }

      fn calculate_baseline_average(&self) -> Decimal {
          if self.volume_history.is_empty() {
              return Decimal::ZERO;
          }

          let sum: Decimal = self.volume_history.iter().map(|(_, v)| v).sum();
          sum / Decimal::from(self.volume_history.len())
      }

      fn normalize_volume_score(&self, ratio: Decimal) -> u32 {
          // Normalize to 0-100 scale
          let score = (ratio * Decimal::from(20)).min(Decimal::from(100));
          score.to_u32().unwrap_or(0)
      }
  }
  ```

#### 1.3 Bid-Ask Spread Analysis

- [ ] **Implement spread calculation and scoring**
  ```rust
  pub fn calculate_spread_metrics(
      order_book: &OrderBook,
  ) -> SpreadMetrics {
      let best_bid = order_book.bids.first().map(|b| b.price);
      let best_ask = order_book.asks.first().map(|a| a.price);

      let (spread, spread_percent) = if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
          let spread = ask - bid;
          let mid_price = (bid + ask) / Decimal::from(2);
          let spread_percent = (spread / mid_price) * Decimal::from(100);
          (spread, spread_percent)
      } else {
          (Decimal::ZERO, Decimal::ZERO)
      };

      SpreadMetrics {
          spread,
          spread_percent,
          tightness_score: calculate_tightness_score(spread_percent),
          liquidity_score: calculate_liquidity_score(order_book),
      }
  }

  fn calculate_tightness_score(spread_percent: Decimal) -> u32 {
      // Lower spread = higher score
      // 0.01% spread = 100 points
      // 1% spread = 0 points
      let inverse_score = (Decimal::from(100) - (spread_percent * Decimal::from(100)))
          .max(Decimal::ZERO)
          .min(Decimal::from(100));

      inverse_score.to_u32().unwrap_or(0)
  }

  fn calculate_liquidity_score(order_book: &OrderBook) -> u32 {
      // Sum depth at top 5 levels on each side
      let bid_depth: Decimal = order_book.bids.iter()
          .take(5)
          .map(|level| level.quantity)
          .sum();

      let ask_depth: Decimal = order_book.asks.iter()
          .take(5)
          .map(|level| level.quantity)
          .sum();

      let total_depth = bid_depth + ask_depth;

      // Normalize to 0-100 (assuming >100 BTC is max liquidity)
      (total_depth / Decimal::from(100) * Decimal::from(100))
          .min(Decimal::from(100))
          .to_u32()
          .unwrap_or(0)
  }
  ```

#### 1.4 Aggregate Volatility Scoring

- [ ] **Implement VolatilityScanner.scan() method**
  ```rust
  impl VolatilityScanner {
      pub async fn scan(
          &mut self,
          market_data: &MarketData,
      ) -> ArbitrageResult<VolatilityScore> {
          // Update internal state
          self.price_analyzer.add_price(market_data.last_price);
          self.volume_analyzer.add_volume(market_data.volume_24h);

          // Calculate individual metrics
          let volatility_metrics = self.price_analyzer.analyze();
          let volume_surge = self.volume_analyzer.detect_surge(market_data.volume);
          let spread_metrics = calculate_spread_metrics(&market_data.order_book);

          // Aggregate into final score (0-100)
          let aggregate_score = self.calculate_aggregate_score(
              &volatility_metrics,
              &volume_surge,
              &spread_metrics,
          );

          Ok(VolatilityScore {
              symbol: market_data.symbol.clone(),
              score: aggregate_score,
              price_change_1m: volatility_metrics.short_term_change,
              volume_surge_ratio: volume_surge.surge_multiplier,
              spread_percent: spread_metrics.spread_percent,
              timestamp: Utc::now(),
          })
      }

      fn calculate_aggregate_score(
          &self,
          volatility: &VolatilityMetrics,
          volume: &VolumeSurgeResult,
          spread: &SpreadMetrics,
      ) -> u32 {
          // Weighted scoring: 40% volatility, 30% volume, 30% liquidity
          let vol_component = volatility.short_term_volatility
              .map(|v| (v * Decimal::from(1000)).min(Decimal::from(40)))
              .unwrap_or(Decimal::ZERO);

          let volume_component = Decimal::from(volume.volume_score) * Decimal::from_str("0.3").unwrap();
          let liquidity_component = Decimal::from(spread.liquidity_score) * Decimal::from_str("0.3").unwrap();

          (vol_component + volume_component + liquidity_component)
              .to_u32()
              .unwrap_or(0)
      }
  }
  ```

#### 1.5 Testing

- [ ] **Add unit tests for price window calculations**
- [ ] **Add unit tests for volume surge detection**
- [ ] **Add unit tests for spread metrics**
- [ ] **Add integration test with simulated market data**
- [ ] **Benchmark scan performance** (target: <100ms per symbol)

---

### Phase 2: Opportunity Detector

**Files to Modify**: `crates/arbitrage-engine/src/opportunity_detector.rs`

#### 2.1 Cross-Exchange Price Comparison

- [ ] **Implement price comparison algorithm**
  ```rust
  impl OpportunityDetector {
      pub async fn detect_opportunities(
          &self,
          markets: Vec<MarketTick>,
      ) -> ArbitrageResult<Vec<OpportunityData>> {
          let mut opportunities = Vec::new();

          // Group markets by symbol
          let markets_by_symbol = self.group_by_symbol(markets);

          for (symbol, ticks) in markets_by_symbol {
              if ticks.len() < 2 {
                  continue; // Need at least 2 exchanges to compare
              }

              // Find all profitable pairs
              for i in 0..ticks.len() {
                  for j in (i + 1)..ticks.len() {
                      if let Some(opp) = self.evaluate_pair(&ticks[i], &ticks[j]).await? {
                          opportunities.push(opp);
                      }
                  }
              }
          }

          // Sort by expected profit descending
          opportunities.sort_by(|a, b| {
              b.expected_profit.partial_cmp(&a.expected_profit).unwrap()
          });

          Ok(opportunities)
      }

      fn group_by_symbol(&self, markets: Vec<MarketTick>) -> HashMap<String, Vec<MarketTick>> {
          let mut grouped = HashMap::new();
          for tick in markets {
              grouped.entry(tick.symbol.clone())
                  .or_insert_with(Vec::new)
                  .push(tick);
          }
          grouped
      }
  }
  ```

- [ ] **Implement pair evaluation with fee calculation**
  ```rust
  async fn evaluate_pair(
      &self,
      tick_a: &MarketTick,
      tick_b: &MarketTick,
  ) -> ArbitrageResult<Option<OpportunityData>> {
      // Determine buy and sell sides
      let (buy_tick, sell_tick) = if tick_a.ask_price < tick_b.bid_price {
          (tick_a, tick_b)
      } else if tick_b.ask_price < tick_a.bid_price {
          (tick_b, tick_a)
      } else {
          return Ok(None); // No arbitrage opportunity
      };

      let buy_price = buy_tick.ask_price;
      let sell_price = sell_tick.bid_price;

      // Calculate fees
      let buy_fee_rate = self.config.exchanges
          .get(&buy_tick.exchange)
          .map(|e| e.trading_fee)
          .unwrap_or(Decimal::from_str("0.001").unwrap());

      let sell_fee_rate = self.config.exchanges
          .get(&sell_tick.exchange)
          .map(|e| e.trading_fee)
          .unwrap_or(Decimal::from_str("0.001").unwrap());

      // Calculate gross profit
      let gross_profit_percent = ((sell_price - buy_price) / buy_price) * Decimal::from(100);

      // Subtract fees
      let net_profit_percent = gross_profit_percent - (buy_fee_rate + sell_fee_rate) * Decimal::from(100);

      // Check minimum profit threshold
      let min_profit = self.config.min_profit_percentage;
      if net_profit_percent < min_profit {
          return Ok(None);
      }

      // Calculate confidence score
      let confidence = self.calculate_confidence(
          buy_tick,
          sell_tick,
          net_profit_percent,
      ).await?;

      // Check minimum confidence threshold
      if confidence < self.config.min_confidence_score {
          return Ok(None);
      }

      Ok(Some(OpportunityData {
          id: Uuid::new_v4().to_string(),
          symbol: buy_tick.symbol.clone(),
          buy_exchange: buy_tick.exchange.clone(),
          sell_exchange: sell_tick.exchange.clone(),
          buy_price,
          sell_price,
          expected_profit: net_profit_percent,
          confidence_score: confidence,
          max_quantity: self.calculate_max_quantity(buy_tick, sell_tick),
          detected_at: Utc::now(),
          status: OpportunityStatus::Detected,
      }))
  }
  ```

#### 2.2 Confidence Scoring

- [ ] **Implement multi-factor confidence calculation**
  ```rust
  async fn calculate_confidence(
      &self,
      buy_tick: &MarketTick,
      sell_tick: &MarketTick,
      profit_percent: Decimal,
  ) -> ArbitrageResult<Decimal> {
      let mut score = Decimal::ZERO;

      // Factor 1: Profit magnitude (0-30 points)
      // Higher profit = higher confidence
      let profit_score = (profit_percent * Decimal::from(10))
          .min(Decimal::from(30));
      score += profit_score;

      // Factor 2: Order book depth (0-25 points)
      let depth_score = self.calculate_depth_score(buy_tick, sell_tick);
      score += depth_score;

      // Factor 3: Historical fill rate (0-20 points)
      let fill_rate = self.get_historical_fill_rate(&buy_tick.exchange, &buy_tick.symbol).await?;
      score += fill_rate * Decimal::from(20);

      // Factor 4: Exchange API latency (0-15 points)
      let latency_score = self.calculate_latency_score(buy_tick, sell_tick).await?;
      score += latency_score;

      // Factor 5: Market volatility context (0-10 points)
      // Lower recent volatility = more stable = higher confidence
      let volatility_score = self.calculate_volatility_context(&buy_tick.symbol).await?;
      score += volatility_score;

      // Normalize to 0-1 range
      Ok(score / Decimal::from(100))
  }

  fn calculate_depth_score(&self, buy_tick: &MarketTick, sell_tick: &MarketTick) -> Decimal {
      // Check if there's enough liquidity at the target prices
      let buy_depth = buy_tick.ask_quantity;
      let sell_depth = sell_tick.bid_quantity;

      let min_depth = buy_depth.min(sell_depth);

      // Assume 10 BTC is "excellent" depth
      let normalized = (min_depth / Decimal::from(10))
          .min(Decimal::ONE);

      normalized * Decimal::from(25)
  }

  async fn calculate_latency_score(
      &self,
      buy_tick: &MarketTick,
      sell_tick: &MarketTick,
  ) -> ArbitrageResult<Decimal> {
      // Get recent API latency metrics for both exchanges
      let buy_latency = self.get_avg_latency(&buy_tick.exchange).await?;
      let sell_latency = self.get_avg_latency(&sell_tick.exchange).await?;

      let total_latency = buy_latency + sell_latency;

      // <100ms = 15 points, >500ms = 0 points
      let score = if total_latency < 100 {
          Decimal::from(15)
      } else if total_latency > 500 {
          Decimal::ZERO
      } else {
          Decimal::from(15) * (Decimal::from(500 - total_latency) / Decimal::from(400))
      };

      Ok(score)
  }
  ```

#### 2.3 Multi-Hop & Triangular Arbitrage

- [ ] **Implement triangular arbitrage detection**
  ```rust
  pub async fn detect_triangular_opportunities(
      &self,
      exchange: &str,
      markets: &[MarketTick],
  ) -> ArbitrageResult<Vec<TriangularOpportunity>> {
      let mut opportunities = Vec::new();

      // Common triangular paths: BTC/USD -> ETH/BTC -> ETH/USD
      let paths = vec![
          ("BTC/USD", "ETH/BTC", "ETH/USD"),
          ("BTC/USD", "LTC/BTC", "LTC/USD"),
          // Add more paths
      ];

      for (pair1, pair2, pair3) in paths {
          if let Some(opp) = self.evaluate_triangular_path(
              exchange,
              markets,
              pair1,
              pair2,
              pair3,
          ).await? {
              opportunities.push(opp);
          }
      }

      Ok(opportunities)
  }

  async fn evaluate_triangular_path(
      &self,
      exchange: &str,
      markets: &[MarketTick],
      pair1: &str,
      pair2: &str,
      pair3: &str,
  ) -> ArbitrageResult<Option<TriangularOpportunity>> {
      // Find market data for each pair
      let tick1 = markets.iter().find(|m| m.symbol == pair1 && m.exchange == exchange);
      let tick2 = markets.iter().find(|m| m.symbol == pair2 && m.exchange == exchange);
      let tick3 = markets.iter().find(|m| m.symbol == pair3 && m.exchange == exchange);

      let (t1, t2, t3) = match (tick1, tick2, tick3) {
          (Some(a), Some(b), Some(c)) => (a, b, c),
          _ => return Ok(None),
      };

      // Calculate if path is profitable
      // Start with $1000 USD
      let start_amount = Decimal::from(1000);

      // Step 1: USD -> BTC
      let btc_amount = start_amount / t1.ask_price;

      // Step 2: BTC -> ETH
      let eth_amount = btc_amount * t2.bid_price;

      // Step 3: ETH -> USD
      let final_usd = eth_amount * t3.bid_price;

      let profit_percent = ((final_usd - start_amount) / start_amount) * Decimal::from(100);

      // Factor in fees (3 trades)
      let fee_rate = Decimal::from_str("0.001").unwrap();
      let net_profit = profit_percent - (fee_rate * Decimal::from(3) * Decimal::from(100));

      if net_profit > self.config.min_profit_percentage {
          Ok(Some(TriangularOpportunity {
              exchange: exchange.to_string(),
              path: vec![pair1.to_string(), pair2.to_string(), pair3.to_string()],
              expected_profit: net_profit,
              start_currency: "USD".to_string(),
              start_amount,
          }))
      } else {
          Ok(None)
      }
  }
  ```

#### 2.4 Opportunity Ranking

- [ ] **Implement opportunity prioritization**
  ```rust
  pub fn rank_opportunities(
      &self,
      mut opportunities: Vec<OpportunityData>,
  ) -> Vec<OpportunityData> {
      opportunities.sort_by(|a, b| {
          // Multi-criteria ranking
          let score_a = self.calculate_opportunity_score(a);
          let score_b = self.calculate_opportunity_score(b);

          score_b.partial_cmp(&score_a).unwrap()
      });

      opportunities
  }

  fn calculate_opportunity_score(&self, opp: &OpportunityData) -> Decimal {
      // Weighted score combining profit and confidence
      let profit_weight = Decimal::from_str("0.6").unwrap();
      let confidence_weight = Decimal::from_str("0.4").unwrap();

      (opp.expected_profit * profit_weight) + (opp.confidence_score * Decimal::from(100) * confidence_weight)
  }
  ```

#### 2.5 Testing

- [ ] **Add unit tests for pair evaluation**
- [ ] **Add unit tests for confidence scoring**
- [ ] **Add unit tests for triangular arbitrage**
- [ ] **Add integration test with simulated market data**
- [ ] **Test opportunity ranking algorithm**

---

### Phase 3: Capital Allocator

**Files to Modify**: `crates/arbitrage-engine/src/capital_allocator.rs`

#### 3.1 Kelly Criterion Implementation

- [ ] **Implement Kelly Criterion position sizing**
  ```rust
  impl CapitalAllocator {
      pub async fn calculate_position_size(
          &self,
          opportunity: &OpportunityData,
          available_capital: Decimal,
      ) -> ArbitrageResult<Decimal> {
          // Kelly Criterion: f* = (bp - q) / b
          // where:
          // f* = fraction of capital to bet
          // b = odds received (profit ratio)
          // p = probability of winning (confidence)
          // q = probability of losing (1 - p)

          let win_prob = opportunity.confidence_score;
          let loss_prob = Decimal::ONE - win_prob;
          let profit_ratio = opportunity.expected_profit / Decimal::from(100);

          let kelly_fraction = if profit_ratio > Decimal::ZERO {
              ((profit_ratio * win_prob) - loss_prob) / profit_ratio
          } else {
              Decimal::ZERO
          };

          // Apply Kelly multiplier (typically 0.25 to 0.5 for safety)
          let fractional_kelly = kelly_fraction * self.config.kelly_multiplier;

          // Calculate raw position size
          let raw_size = available_capital * fractional_kelly;

          // Apply constraints
          self.apply_position_constraints(raw_size, opportunity).await
      }

      async fn apply_position_constraints(
          &self,
          raw_size: Decimal,
          opportunity: &OpportunityData,
      ) -> ArbitrageResult<Decimal> {
          let mut size = raw_size;

          // Constraint 1: Maximum position size
          size = size.min(self.config.max_position_size);

          // Constraint 2: Maximum quantity available in order books
          size = size.min(opportunity.max_quantity);

          // Constraint 3: Maximum portfolio exposure per symbol
          let current_exposure = self.get_current_exposure(&opportunity.symbol).await?;
          let max_total_exposure = self.config.max_position_exposure_percent * available_capital;
          size = size.min(max_total_exposure - current_exposure);

          // Constraint 4: Minimum viable trade size
          if size < self.config.min_trade_size {
              return Ok(Decimal::ZERO); // Don't trade if too small
          }

          Ok(size)
      }
  }
  ```

#### 3.2 Dynamic Allocation Based on Performance

- [ ] **Implement allocation adjustment based on recent P&L**
  ```rust
  pub async fn adjust_for_performance(
      &self,
      base_allocation: Decimal,
  ) -> ArbitrageResult<Decimal> {
      // Get recent trading performance
      let recent_pnl = self.get_recent_pnl(Duration::from_hours(24)).await?;
      let recent_win_rate = self.get_recent_win_rate(Duration::from_hours(24)).await?;

      // Reduce allocation if recent performance is poor
      let performance_multiplier = if recent_pnl < Decimal::ZERO {
          // Losing money - reduce allocation
          let loss_severity = (recent_pnl.abs() / self.config.max_daily_loss).min(Decimal::ONE);
          Decimal::ONE - (loss_severity * Decimal::from_str("0.5").unwrap())
      } else if recent_win_rate < Decimal::from_str("0.6").unwrap() {
          // Low win rate - be more conservative
          Decimal::from_str("0.7").unwrap()
      } else {
          // Performing well - maintain or slightly increase
          Decimal::ONE
      };

      Ok(base_allocation * performance_multiplier)
  }
  ```

#### 3.3 Gekko Mode Aggressive Allocation

- [ ] **Implement Gekko mode allocation strategy**
  ```rust
  pub async fn calculate_gekko_allocation(
      &self,
      opportunity: &OpportunityData,
      available_capital: Decimal,
  ) -> ArbitrageResult<Decimal> {
      if !self.config.gekko_mode {
          return self.calculate_position_size(opportunity, available_capital).await;
      }

      // Gekko mode: 90% capital allocation aggressiveness
      let base_allocation = available_capital * self.config.allocation_aggressiveness;

      // Only take high-confidence opportunities in Gekko mode
      if opportunity.confidence_score < Decimal::from_str("0.85").unwrap() {
          return Ok(Decimal::ZERO);
      }

      // Apply maximum position constraints
      let constrained = base_allocation
          .min(self.config.max_position_size)
          .min(opportunity.max_quantity);

      Ok(constrained)
  }
  ```

#### 3.4 Portfolio Exposure Tracking

- [ ] **Implement real-time exposure calculation**
  ```rust
  pub async fn get_current_exposure(&self, symbol: &str) -> ArbitrageResult<Decimal> {
      // Query database for open positions in this symbol
      let positions = self.db.get_open_positions(symbol).await?;

      let total_exposure: Decimal = positions.iter()
          .map(|p| p.quantity * p.entry_price)
          .sum();

      Ok(total_exposure)
  }

  pub async fn check_risk_limits(&self, new_trade: &ProposedTrade) -> ArbitrageResult<bool> {
      // Check 1: Daily loss limit
      let daily_pnl = self.get_daily_pnl().await?;
      if daily_pnl < -self.config.max_daily_loss {
          return Ok(false);
      }

      // Check 2: Maximum portfolio exposure
      let total_exposure = self.get_total_portfolio_exposure().await?;
      let available_capital = self.get_available_capital().await?;

      if total_exposure / available_capital > Decimal::from_str("0.95").unwrap() {
          return Ok(false);
      }

      // Check 3: Per-symbol exposure
      let symbol_exposure = self.get_current_exposure(&new_trade.symbol).await?;
      let max_symbol_exposure = available_capital * self.config.max_position_exposure_percent;

      if symbol_exposure + new_trade.quantity > max_symbol_exposure {
          return Ok(false);
      }

      Ok(true)
  }
  ```

#### 3.5 Testing

- [ ] **Add unit tests for Kelly Criterion calculation**
- [ ] **Add unit tests for constraint application**
- [ ] **Add unit tests for Gekko mode allocation**
- [ ] **Test exposure tracking logic**
- [ ] **Test risk limit checks**

---

### Phase 4: Execution Engine

**Files to Modify**: `crates/arbitrage-engine/src/execution_engine.rs`

#### 4.1 Order Routing & Coordination

- [ ] **Implement simultaneous order placement**
  ```rust
  impl ExecutionEngine {
      pub async fn execute_arbitrage(
          &mut self,
          opportunity: &OpportunityData,
          position_size: Decimal,
      ) -> ArbitrageResult<ExecutionResult> {
          // Validate opportunity is still valid
          if !self.validate_opportunity(opportunity).await? {
              return Err(ArbitrageError::StaleOpportunity);
          }

          // Check risk limits
          if !self.capital_allocator.check_risk_limits(&ProposedTrade {
              symbol: opportunity.symbol.clone(),
              quantity: position_size,
          }).await? {
              return Err(ArbitrageError::RiskLimitExceeded);
          }

          // Execute both legs simultaneously
          let (buy_result, sell_result) = tokio::try_join!(
              self.place_buy_order(opportunity, position_size),
              self.place_sell_order(opportunity, position_size)
          )?;

          // Handle partial fills
          let execution = self.reconcile_fills(buy_result, sell_result).await?;

          // Emit execution event
          self.event_bus.dispatch(Event::Execution(ExecutionEvent {
              opportunity_id: opportunity.id.clone(),
              buy_order_id: execution.buy_order_id.clone(),
              sell_order_id: execution.sell_order_id.clone(),
              realized_profit: execution.profit,
              timestamp: Utc::now(),
          })).await?;

          Ok(execution)
      }

      async fn place_buy_order(
          &self,
          opportunity: &OpportunityData,
          quantity: Decimal,
      ) -> ArbitrageResult<OrderResult> {
          let connector = self.exchange_manager
              .get_connector(&opportunity.buy_exchange)
              .await?;

          let order = connector.place_order(
              &opportunity.symbol,
              OrderSide::Buy,
              OrderType::Limit,
              quantity,
              Some(opportunity.buy_price),
          ).await.map_err(|e| ArbitrageError::OrderPlacementFailed(e.to_string()))?;

          Ok(OrderResult {
              order_id: order.id,
              status: order.status,
              filled_quantity: order.filled_quantity,
              average_price: order.price,
          })
      }

      async fn place_sell_order(
          &self,
          opportunity: &OpportunityData,
          quantity: Decimal,
      ) -> ArbitrageResult<OrderResult> {
          let connector = self.exchange_manager
              .get_connector(&opportunity.sell_exchange)
              .await?;

          let order = connector.place_order(
              &opportunity.symbol,
              OrderSide::Sell,
              OrderType::Limit,
              quantity,
              Some(opportunity.sell_price),
          ).await.map_err(|e| ArbitrageError::OrderPlacementFailed(e.to_string()))?;

          Ok(OrderResult {
              order_id: order.id,
              status: order.status,
              filled_quantity: order.filled_quantity,
              average_price: order.price,
          })
      }
  }
  ```

#### 4.2 Partial Fill Handling

- [ ] **Implement fill reconciliation**
  ```rust
  async fn reconcile_fills(
      &self,
      buy_result: OrderResult,
      sell_result: OrderResult,
  ) -> ArbitrageResult<ExecutionResult> {
      // Wait for fills with timeout
      let timeout = Duration::from_secs(30);
      let start = Instant::now();

      let mut buy_filled = buy_result.filled_quantity;
      let mut sell_filled = sell_result.filled_quantity;

      while start.elapsed() < timeout {
          if buy_filled > Decimal::ZERO && sell_filled > Decimal::ZERO {
              break;
          }

          tokio::time::sleep(Duration::from_millis(500)).await;

          // Poll order status
          // Update buy_filled and sell_filled
      }

      // Handle mismatch
      let executed_quantity = buy_filled.min(sell_filled);

      if executed_quantity == Decimal::ZERO {
          // Cancel both orders
          self.cancel_orders(&buy_result.order_id, &sell_result.order_id).await?;
          return Err(ArbitrageError::NoFills);
      }

      // If partial fills don't match, handle excess
      if buy_filled > sell_filled {
          // Excess bought - need to sell remainder
          self.handle_excess_position(
              &buy_result,
              buy_filled - sell_filled,
              OrderSide::Sell,
          ).await?;
      } else if sell_filled > buy_filled {
          // Excess sold (short) - need to buy back
          self.handle_excess_position(
              &sell_result,
              sell_filled - buy_filled,
              OrderSide::Buy,
          ).await?;
      }

      // Calculate realized profit
      let buy_cost = buy_result.average_price.unwrap() * executed_quantity;
      let sell_proceeds = sell_result.average_price.unwrap() * executed_quantity;
      let profit = sell_proceeds - buy_cost;

      Ok(ExecutionResult {
          buy_order_id: buy_result.order_id,
          sell_order_id: sell_result.order_id,
          executed_quantity,
          profit,
          timestamp: Utc::now(),
      })
  }
  ```

#### 4.3 Timeout & Cancellation Logic

- [ ] **Implement order timeout handling**
  ```rust
  async fn monitor_order_with_timeout(
      &self,
      order_id: &str,
      exchange: &str,
      timeout: Duration,
  ) -> ArbitrageResult<OrderStatus> {
      let start = Instant::now();

      loop {
          let connector = self.exchange_manager.get_connector(exchange).await?;
          let status = connector.get_order_status(order_id).await?;

          match status {
              OrderStatus::Filled => return Ok(status),
              OrderStatus::Cancelled | OrderStatus::Rejected => {
                  return Err(ArbitrageError::OrderFailed(order_id.to_string()));
              }
              _ => {
                  if start.elapsed() > timeout {
                      // Timeout - cancel order
                      connector.cancel_order(order_id).await?;
                      return Err(ArbitrageError::OrderTimeout);
                  }

                  tokio::time::sleep(Duration::from_millis(500)).await;
              }
          }
      }
  }
  ```

#### 4.4 Performance Tracking

- [ ] **Implement execution metrics collection**
  ```rust
  pub struct ExecutionMetrics {
      pub order_placement_latency: Duration,
      pub fill_time: Duration,
      pub slippage: Decimal,
      pub realized_profit: Decimal,
      pub expected_profit: Decimal,
  }

  impl ExecutionEngine {
      async fn record_execution_metrics(
          &self,
          opportunity: &OpportunityData,
          execution: &ExecutionResult,
          start_time: Instant,
      ) -> ArbitrageResult<()> {
          let metrics = ExecutionMetrics {
              order_placement_latency: start_time.elapsed(),
              fill_time: execution.timestamp.signed_duration_since(opportunity.detected_at)
                  .to_std()
                  .unwrap_or_default(),
              slippage: self.calculate_slippage(opportunity, execution),
              realized_profit: execution.profit,
              expected_profit: opportunity.expected_profit,
          };

          // Store metrics to database
          self.db.store_execution_metrics(&metrics).await?;

          // Emit metrics event
          self.event_bus.dispatch(Event::Metrics(MetricsEvent {
              metrics,
          })).await?;

          Ok(())
      }

      fn calculate_slippage(
          &self,
          opportunity: &OpportunityData,
          execution: &ExecutionResult,
      ) -> Decimal {
          let expected_buy_cost = opportunity.buy_price * execution.executed_quantity;
          let expected_sell_proceeds = opportunity.sell_price * execution.executed_quantity;
          let expected_profit = expected_sell_proceeds - expected_buy_cost;

          let actual_profit = execution.profit;

          ((expected_profit - actual_profit) / expected_profit) * Decimal::from(100)
      }
  }
  ```

#### 4.5 Testing

- [ ] **Add unit tests for order placement**
- [ ] **Add unit tests for fill reconciliation**
- [ ] **Add unit tests for timeout handling**
- [ ] **Add integration test with mock exchanges**
- [ ] **Test simultaneous order execution**
- [ ] **Benchmark execution latency** (target: <100ms total)

---

### Phase 5: Risk Management Integration

**Files to Modify**: `crates/arbitrage-engine/src/execution_engine.rs`, `crates/arbitrage-engine/src/risk_manager.rs` (create)

#### 5.1 Stop-Loss Implementation

- [ ] **Implement stop-loss activation**
  ```rust
  pub struct RiskManager {
      config: RiskConfig,
      active_positions: HashMap<String, Position>,
  }

  impl RiskManager {
      pub async fn check_stop_loss(&self, position: &Position) -> bool {
          let current_loss_percent = ((position.current_value - position.entry_value) / position.entry_value) * Decimal::from(100);

          current_loss_percent < -self.config.stop_loss_percent
      }

      pub async fn trigger_emergency_exit(&mut self) -> ArbitrageResult<()> {
          tracing::warn!("Emergency exit triggered - closing all positions");

          for (symbol, position) in &self.active_positions {
              self.close_position(symbol, position).await?;
          }

          self.active_positions.clear();

          Ok(())
      }
  }
  ```

#### 5.2 Circuit Breaker for Consecutive Losses

- [ ] **Implement consecutive loss tracking**
  ```rust
  pub struct LossTracker {
      consecutive_losses: u32,
      max_consecutive_losses: u32,
  }

  impl LossTracker {
      pub fn record_trade_result(&mut self, profit: Decimal) -> bool {
          if profit < Decimal::ZERO {
              self.consecutive_losses += 1;
              if self.consecutive_losses >= self.max_consecutive_losses {
                  return true; // Circuit breaker triggered
              }
          } else {
              self.consecutive_losses = 0; // Reset on win
          }

          false
      }
  }
  ```

#### 5.3 Daily Loss Limit

- [ ] **Implement daily P&L tracking and enforcement**
- [ ] **Add automatic trading pause on limit hit**

#### 5.4 Testing

- [ ] **Test stop-loss activation**
- [ ] **Test circuit breaker triggering**
- [ ] **Test daily loss limit enforcement**

---

## Acceptance Criteria

### Functional Requirements

- [ ] Volatility scanner produces scores for all monitored symbols
- [ ] Opportunity detector identifies profitable arbitrage opportunities
- [ ] Capital allocator calculates appropriate position sizes using Kelly Criterion
- [ ] Execution engine places simultaneous orders on both exchanges
- [ ] Partial fills are handled correctly
- [ ] Risk limits are enforced (stop-loss, daily loss, consecutive losses)
- [ ] All executions are logged to database with full audit trail

### Performance Requirements

- [ ] Volatility scan completes in <100ms per symbol
- [ ] Opportunity detection runs in <50ms for 10 symbols across 3 exchanges
- [ ] Order execution completes in <100ms from detection to placement
- [ ] System maintains >85% confidence score threshold for executed trades
- [ ] Actual profit matches expected profit within 10% (accounting for slippage)

### Code Quality

- [ ] All functions have rustdoc comments
- [ ] No `cargo clippy` warnings
- [ ] Code formatted with `cargo fmt`
- [ ] Unit test coverage >80%
- [ ] Integration tests cover full arbitrage cycle
- [ ] Benchmarks demonstrate performance targets are met

---

## Implementation Notes for AI Agents

### Order of Implementation

1. **Start with VolatilityScanner** - Foundational data analysis
2. **Then OpportunityDetector** - Core business logic
3. **Then CapitalAllocator** - Position sizing
4. **Finally ExecutionEngine** - Trading execution
5. **Add RiskManager** - Safety mechanisms

### Key Dependencies

- Requires Issue #1 (Exchange API Integration) to be completed first
- Uses `event-bus` crate for event emission
- Uses `database` crate for persistence
- Uses `config/arbitrage.toml` for configuration

### Testing Strategy

1. **Unit tests**: Test each component in isolation with mock data
2. **Integration tests**: Test full flow with mock exchanges
3. **Paper trading**: Test with real market data but simulated orders
4. **Live testing**: Small position sizes on real exchanges

### Database Schema

Ensure these tables exist (from `database/migrations/`):
- `arbitrage_opportunities` - Detected opportunities
- `arbitrage_executions` - Executed trades
- `arbitrage_performance` - P&L tracking

---

## Verification Commands

```bash
# Unit tests
cargo test -p arbitrage-engine

# Integration tests
cargo test -p arbitrage-engine --test integration_tests

# Benchmarks
cargo bench -p arbitrage-engine

# Run with paper trading mode
cargo run -- --paper-trading --config config/arbitrage.toml
```

---

## Related Issues

- Depends on: #1 (Exchange API Integration)
- Blocks: Production deployment

## References

- Internal: `crates/arbitrage-engine/src/lib.rs:1`
- Internal: `config/arbitrage.toml:1`
- Kelly Criterion: https://en.wikipedia.org/wiki/Kelly_criterion

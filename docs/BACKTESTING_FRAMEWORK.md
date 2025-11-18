# Ninja Gekko Backtesting Framework Design

## Overview

This document outlines the design and requirements for the Ninja Gekko backtesting framework. The framework enables strategy validation against historical data before live deployment, ensuring strategies meet performance and risk criteria.

**References**:
- `AGENTS.md` - Performance targets and testing requirements
- `crates/data-pipeline/src/pipeline.rs` - Data handling patterns
- `crates/strategy-engine/src/lib.rs` - Strategy interfaces

---

## Objectives

1. **Validate Strategies**: Test trading strategies against historical data
2. **Measure Performance**: Calculate realistic performance metrics
3. **Assess Risk**: Evaluate risk characteristics and drawdowns
4. **Optimize Parameters**: Enable strategy parameter tuning
5. **Ensure Reliability**: Deterministic, reproducible results

---

## Architecture

### Component Overview

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Data Loader   │────▶│  Event Replayer │────▶│ Strategy Engine │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Report Generator│◀────│  Metrics Calc   │◀────│ Order Simulator │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Core Components

#### 1. Data Loader

Responsible for loading and validating historical data.

```rust
pub struct DataLoader {
    data_source: DataSource,
    date_range: DateRange,
    symbols: Vec<String>,
    resolution: TimeResolution,
}

impl DataLoader {
    /// Load historical data for backtesting
    pub async fn load(&self) -> Result<HistoricalData, BacktestError> {
        // Load from database or files
        // Validate data integrity
        // Fill gaps if configured
        // Return normalized data
    }
}
```

**Data Sources**:
- PostgreSQL time-series tables
- CSV/Parquet files
- External data providers

**Validation**:
- Check for gaps in data
- Validate price ranges
- Ensure chronological ordering
- Detect duplicate timestamps

#### 2. Event Replayer

Replays historical events in chronological order.

```rust
pub struct EventReplayer {
    events: Vec<HistoricalEvent>,
    current_index: usize,
    speed_multiplier: f64,
}

impl EventReplayer {
    /// Get next event in sequence
    pub fn next_event(&mut self) -> Option<&HistoricalEvent> {
        // Return events in timestamp order
        // Support multiple event types
        // Handle gaps appropriately
    }

    /// Seek to specific timestamp
    pub fn seek(&mut self, timestamp: DateTime<Utc>) -> Result<(), BacktestError> {
        // Binary search for efficiency
    }
}
```

**Event Types**:
- Market data ticks
- OHLCV candles
- Order book updates
- Trade executions

#### 3. Order Simulator

Simulates order execution with realistic conditions.

```rust
pub struct OrderSimulator {
    slippage_model: SlippageModel,
    commission_model: CommissionModel,
    fill_model: FillModel,
    latency_model: LatencyModel,
}

impl OrderSimulator {
    /// Simulate order execution
    pub fn execute_order(
        &self,
        order: &Order,
        market_state: &MarketState,
    ) -> ExecutionResult {
        // Apply latency
        // Calculate slippage
        // Determine fill price and quantity
        // Calculate commission
        // Return execution result
    }
}
```

**Models**:

##### Slippage Model
```rust
pub enum SlippageModel {
    /// No slippage (unrealistic)
    None,
    /// Fixed percentage
    Fixed { percentage: f64 },
    /// Volume-based impact
    VolumeImpact { factor: f64 },
    /// Order book simulation
    OrderBook { depth: usize },
}
```

##### Commission Model
```rust
pub enum CommissionModel {
    /// Fixed fee per trade
    Fixed { amount: Decimal },
    /// Percentage of trade value
    Percentage { rate: Decimal },
    /// Tiered based on volume
    Tiered { tiers: Vec<CommissionTier> },
    /// Exchange-specific
    Exchange { exchange_id: ExchangeId },
}
```

##### Fill Model
```rust
pub enum FillModel {
    /// Instant complete fill
    Immediate,
    /// Partial fills based on liquidity
    Liquidity { available_liquidity: f64 },
    /// Realistic partial fills
    Realistic { fill_probability: f64 },
}
```

#### 4. Strategy Engine Integration

Integrates with existing strategy engine for consistent behavior.

```rust
pub trait BacktestableStrategy: Strategy {
    /// Called for each historical event
    fn on_historical_event(&mut self, event: &HistoricalEvent) -> Vec<Signal>;

    /// Get strategy state for snapshot
    fn get_state(&self) -> StrategyState;

    /// Restore strategy from snapshot
    fn restore_state(&mut self, state: StrategyState);
}
```

#### 5. Metrics Calculator

Calculates comprehensive performance metrics.

```rust
pub struct MetricsCalculator {
    equity_curve: Vec<EquityPoint>,
    trades: Vec<CompletedTrade>,
    positions: Vec<PositionSnapshot>,
}

impl MetricsCalculator {
    /// Calculate all performance metrics
    pub fn calculate_metrics(&self) -> BacktestMetrics {
        BacktestMetrics {
            // Return metrics
            total_return: self.calculate_total_return(),
            sharpe_ratio: self.calculate_sharpe_ratio(),
            max_drawdown: self.calculate_max_drawdown(),
            win_rate: self.calculate_win_rate(),
            profit_factor: self.calculate_profit_factor(),
            // ... more metrics
        }
    }
}
```

**Metrics**:

| Metric | Description | Formula |
|--------|-------------|---------|
| Total Return | Overall profit/loss | `(final - initial) / initial` |
| CAGR | Compound annual growth | `(final/initial)^(1/years) - 1` |
| Sharpe Ratio | Risk-adjusted return | `(return - risk_free) / std_dev` |
| Sortino Ratio | Downside risk-adjusted | `(return - risk_free) / downside_dev` |
| Max Drawdown | Largest peak-to-trough | `max((peak - trough) / peak)` |
| Win Rate | Percentage winning trades | `wins / total_trades` |
| Profit Factor | Gross profit / loss | `gross_profit / gross_loss` |
| Expectancy | Expected profit per trade | `win_rate * avg_win - loss_rate * avg_loss` |
| Calmar Ratio | CAGR / Max Drawdown | `cagr / max_drawdown` |

#### 6. Report Generator

Generates comprehensive backtest reports.

```rust
pub struct ReportGenerator {
    metrics: BacktestMetrics,
    trades: Vec<CompletedTrade>,
    equity_curve: Vec<EquityPoint>,
    config: BacktestConfig,
}

impl ReportGenerator {
    /// Generate HTML report
    pub fn generate_html(&self) -> String {
        // Performance summary
        // Equity curve chart
        // Drawdown chart
        // Trade list
        // Monthly returns
    }

    /// Generate JSON report
    pub fn generate_json(&self) -> serde_json::Value {
        // Structured data for programmatic access
    }
}
```

---

## Configuration

### Backtest Configuration

```rust
pub struct BacktestConfig {
    /// Date range for backtest
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,

    /// Initial capital
    pub initial_capital: Decimal,

    /// Symbols to trade
    pub symbols: Vec<String>,

    /// Data resolution
    pub resolution: TimeResolution,

    /// Execution models
    pub slippage_model: SlippageModel,
    pub commission_model: CommissionModel,
    pub fill_model: FillModel,

    /// Risk parameters
    pub max_position_size: Decimal,
    pub max_drawdown_limit: Decimal,

    /// Output options
    pub generate_report: bool,
    pub save_trades: bool,
}
```

### Example Configuration

```toml
[backtest]
start_date = "2024-01-01T00:00:00Z"
end_date = "2024-12-31T23:59:59Z"
initial_capital = 100000.0
symbols = ["BTC-USD", "ETH-USD"]
resolution = "1m"

[backtest.slippage]
model = "VolumeImpact"
factor = 0.001

[backtest.commission]
model = "Percentage"
rate = 0.001

[backtest.risk]
max_position_size = 0.1
max_drawdown_limit = 0.2
```

---

## Validation Requirements

### Data Validation

1. **Completeness**: No more than 1% missing data points
2. **Accuracy**: Prices within expected ranges
3. **Ordering**: Strictly chronological
4. **Consistency**: OHLC relationships valid (high >= low, etc.)

### Results Validation

1. **Determinism**: Same inputs produce same outputs
2. **Reproducibility**: Results can be recreated
3. **Accuracy**: Calculations match reference implementations

---

## Performance Requirements

Based on `AGENTS.md` targets:

| Operation | Target | Description |
|-----------|--------|-------------|
| Data Loading | < 10s | Load 1 year of minute data |
| Event Processing | 100k events/sec | Process historical events |
| Report Generation | < 5s | Generate full HTML report |
| Memory Usage | < 2GB | For standard backtests |

---

## Integration Points

### Data Pipeline

```rust
// Use existing data pipeline for normalization
use data_pipeline::normalize::normalize_market_data;

let raw_data = load_historical_data()?;
let normalized = normalize_market_data(raw_data)?;
```

### Strategy Engine

```rust
// Use existing strategy interface
use strategy_engine::Strategy;

let strategy = MyStrategy::new(params);
let backtest = Backtest::new(config);
let results = backtest.run(&strategy)?;
```

### Event Bus

```rust
// Compatible with event system
use event_bus::Event;

let events: Vec<MarketEvent> = replay_historical(data);
for event in events {
    strategy.on_event(&event);
}
```

---

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)

- [ ] Data loader with validation
- [ ] Event replayer
- [ ] Basic order simulator (immediate fill)

### Phase 2: Execution Models (Week 3)

- [ ] Slippage models
- [ ] Commission models
- [ ] Fill models
- [ ] Latency simulation

### Phase 3: Metrics & Reporting (Week 4)

- [ ] Metrics calculator
- [ ] Equity curve tracking
- [ ] HTML report generator
- [ ] JSON export

### Phase 4: Optimization & Testing (Week 5)

- [ ] Parameter optimization support
- [ ] Walk-forward analysis
- [ ] Monte Carlo simulation
- [ ] Comprehensive tests

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_sharpe_ratio_calculation() {
    let returns = vec![0.01, -0.005, 0.02, 0.01, -0.01];
    let sharpe = calculate_sharpe_ratio(&returns, 0.0);
    assert!((sharpe - expected_sharpe).abs() < 0.001);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_backtest() {
    let config = BacktestConfig::default();
    let strategy = SimpleMovingAverageCrossover::new(10, 20);
    let backtest = Backtest::new(config);

    let results = backtest.run(&strategy).await.unwrap();

    assert!(results.metrics.total_return.is_finite());
    assert!(results.trades.len() > 0);
}
```

### Validation Tests

```rust
#[test]
fn test_backtest_determinism() {
    let config = BacktestConfig::default();
    let strategy = TestStrategy::new();

    let results1 = Backtest::new(config.clone()).run(&strategy);
    let results2 = Backtest::new(config).run(&strategy);

    assert_eq!(results1.metrics, results2.metrics);
}
```

---

## Future Enhancements

### Multi-Asset Support

- Portfolio backtesting
- Correlation analysis
- Asset allocation optimization

### Advanced Features

- Machine learning integration
- Sentiment data inclusion
- Alternative data sources
- Real-time comparison

### Performance Optimization

- Parallel event processing
- GPU acceleration for calculations
- Incremental computation

---

## References

- `AGENTS.md` - Core principles and targets
- `crates/data-pipeline/` - Data handling patterns
- `crates/strategy-engine/` - Strategy interfaces
- `docs/arbitrage_architecture.md` - System architecture

---

*This framework design ensures strategies are thoroughly validated before live deployment, reducing risk and improving confidence in trading performance.*

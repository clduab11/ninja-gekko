# Neural Network Models

This directory contains trained neural network models for the Ninja Gekko trading system.

## Directory Structure

```
models/
├── volatility/          # Volatility prediction models
│   └── volatility_predictor.net   # FANN volatility model
├── arbitrage/           # Arbitrage detection models
│   └── arbitrage_detector.net     # FANN arbitrage model
├── risk/                # Risk assessment models
│   └── risk_assessor.net          # FANN risk model
└── README.md
```

## Model Formats

### FANN (.net) Files
- **Format**: ruv-FANN compatible network files
- **Usage**: Load with `FannModel::load_from_file()` when implemented
- **Current Status**: **Untrained networks** - Models are initialized with random weights and must be trained on historical data before production use

### ⚠️ Important: Training Required

The neural models created by `FannModel::create_*_model()` are **untrained**. Before using in production:

1. Collect historical training data (market data, spreads, outcomes)
2. Train models using ruv-FANN training APIs
3. Save trained weights to `.net` files
4. Validate model accuracy meets threshold (≥85%)

### Model Specifications

#### Volatility Predictor
- **Inputs (7)**: price, high, low, volume, avg_volume, bid, ask
- **Outputs (4)**: volatility_1m, volatility_5m, volatility_15m, confidence
- **Architecture**: 7 → 32 → 16 → 4

#### Arbitrage Detector
- **Inputs (9)**: primary_price, secondary_price, spread, primary_volume, secondary_volume, primary_bid, primary_ask, secondary_bid, secondary_ask
- **Outputs (5)**: spread_1m, spread_5m, arb_probability, expected_profit, confidence
- **Architecture**: 9 → 48 → 24 → 12 → 5

#### Risk Assessor
- **Inputs (5)**: position_size, volatility, spread, volume_ratio, time_factor
- **Outputs (5)**: overall_risk, liquidity_risk, execution_risk, max_position, confidence
- **Architecture**: 5 → 24 → 12 → 5

## Training

Models are currently initialized with random weights and can be trained using:
- Historical market data for volatility prediction
- Cross-exchange spread data for arbitrage detection
- Position outcome data for risk assessment

## Configuration

Model paths can be configured in `config/arbitrage.toml`:

```toml
[neural_engine]
volatility_model_path = "models/volatility/volatility_predictor.net"
arbitrage_model_path = "models/arbitrage/arbitrage_detector.net"
risk_model_path = "models/risk/risk_assessor.net"
```

## Performance Targets

- **Inference time**: < 50ms per prediction
- **Model accuracy**: > 85% validation accuracy
- **Memory footprint**: < 100MB per model

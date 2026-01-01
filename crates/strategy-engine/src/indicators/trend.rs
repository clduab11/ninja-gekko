use crate::indicators::{buffer, dec_to_f64, f64_to_dec, Indicator, IndicatorValue};
use rust_decimal::Decimal;
use yata::core::{Method, PeriodType, ValueType};
use yata::methods::{ADI, EMA, SMA};

// ============================================================================
// SMA
// ============================================================================
/// Simple Moving Average (SMA).
///
/// An unweighted moving average.
pub struct Sma {
    inner: SMA,
    current: Option<ValueType>,
    samples: usize,
    period: usize,
}

impl Sma {
    /// Create a new SMA.
    pub fn new(period: usize) -> Self {
        Self {
            inner: SMA::new(period as PeriodType, &0.0).unwrap(),
            current: None,
            samples: 0,
            period,
        }
    }
}

impl Indicator for Sma {
    fn name(&self) -> &'static str {
        "SMA"
    }

    fn update(&mut self, price: Decimal) -> IndicatorValue {
        let val = dec_to_f64(price);
        let result = self.inner.next(&val);
        self.current = Some(result);
        self.samples += 1;
        IndicatorValue {
            value: f64_to_dec(result),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        self.current.map(|v| IndicatorValue {
            value: f64_to_dec(v),
            signal: None,
        })
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

// ============================================================================
// EMA
// ============================================================================
/// Exponential Moving Average (EMA).
///
/// A weighted moving average that gives more weighting or importance to recent price data.
pub struct Ema {
    inner: EMA,
    current: Option<ValueType>,
    samples: usize,
    period: usize,
}

impl Ema {
    /// Create a new EMA.
    pub fn new(period: usize) -> Self {
        Self {
            inner: EMA::new(period as PeriodType, &0.0).unwrap(),
            current: None,
            samples: 0,
            period,
        }
    }
}

impl Indicator for Ema {
    fn name(&self) -> &'static str {
        "EMA"
    }

    fn update(&mut self, price: Decimal) -> IndicatorValue {
        let val = dec_to_f64(price);
        let result = self.inner.next(&val);
        self.current = Some(result);
        self.samples += 1;
        IndicatorValue {
            value: f64_to_dec(result),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        self.current.map(|v| IndicatorValue {
            value: f64_to_dec(v),
            signal: None,
        })
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

// ============================================================================
// MACD
// ============================================================================
/// Moving Average Convergence/Divergence (MACD).
///
/// A trend-following momentum indicator that shows the relationship between two moving averages of prices.
pub struct Macd {
    fast_ema: EMA,
    slow_ema: EMA,
    signal_ema: EMA,
    current_macd: Option<ValueType>,
    current_signal: Option<ValueType>,
    samples: usize,
    warmup: usize,
}

impl Macd {
    /// Create a new MACD.
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_ema: EMA::new(fast_period as PeriodType, &0.0).unwrap(),
            slow_ema: EMA::new(slow_period as PeriodType, &0.0).unwrap(),
            signal_ema: EMA::new(signal_period as PeriodType, &0.0).unwrap(),
            current_macd: None,
            current_signal: None,
            samples: 0,
            warmup: slow_period + signal_period,
        }
    }
}

impl Indicator for Macd {
    fn name(&self) -> &'static str {
        "MACD"
    }

    fn update(&mut self, price: Decimal) -> IndicatorValue {
        let val = dec_to_f64(price);
        let fast = self.fast_ema.next(&val);
        let slow = self.slow_ema.next(&val);
        let macd_line = fast - slow;

        // Signal line is EMA of MACD line
        let signal_line = self.signal_ema.next(&macd_line);

        self.current_macd = Some(macd_line);
        self.current_signal = Some(signal_line);
        self.samples += 1;

        IndicatorValue {
            value: f64_to_dec(macd_line),
            signal: Some(f64_to_dec(signal_line)),
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        match (self.current_macd, self.current_signal) {
            (Some(m), Some(s)) => Some(IndicatorValue {
                value: f64_to_dec(m),
                signal: Some(f64_to_dec(s)),
            }),
            _ => None,
        }
    }

    fn warmup_period(&self) -> usize {
        self.warmup
    }
    fn is_ready(&self) -> bool {
        self.samples >= self.warmup
    }
}

// ============================================================================
// ADX
// ============================================================================
/// Average Directional Index (ADX).
///
/// Used to determine the strength of a trend.
pub struct Adx {
    inner: ADI,
    current: Option<ValueType>,
    samples: usize,
    period: usize,
}

impl Adx {
    /// Create a new ADX.
    pub fn new(period: usize) -> Self {
        Self {
            inner: ADI::new(period as PeriodType, &yata::core::Candle::default()).unwrap(),
            current: None,
            samples: 0,
            period,
        }
    }
}

impl Indicator for Adx {
    fn name(&self) -> &'static str {
        "ADX"
    }

    // ADX requires TR, +DM, -DM which needs High/Low.
    // `update(price)` only gives one value. `update_ohlcv` gives candle.
    fn update(&mut self, _price: Decimal) -> IndicatorValue {
        // Not supported for ADX properly
        IndicatorValue {
            value: Decimal::ZERO,
            signal: None,
        }
    }

    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        let high = dec_to_f64(candle.high);
        let low = dec_to_f64(candle.low);
        let close = dec_to_f64(candle.close);

        let yata_candle = yata::core::Candle {
            open: dec_to_f64(candle.open),
            high,
            low,
            close,
            volume: dec_to_f64(candle.volume),
        };

        // This assumes `AverageDirectionalIndex` implements `Next<&Candle>`.
        let result = self.inner.next(&yata_candle);
        self.current = Some(result);
        self.samples += 1;
        IndicatorValue {
            value: f64_to_dec(result),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        self.current.map(|v| IndicatorValue {
            value: f64_to_dec(v),
            signal: None,
        })
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

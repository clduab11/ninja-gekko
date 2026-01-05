use crate::indicators::{buffer, dec_to_f64, f64_to_dec, Indicator, IndicatorValue};
use rust_decimal::Decimal;
use std::collections::VecDeque;
use yata::core::{Method, PeriodType, ValueType};
use yata::methods::{EMA, SMA};

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
/// Manual implementation that computes +DI, -DI and ADX from TR, +DM, -DM.
pub struct Adx {
    period: usize,
    samples: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    tr_history: VecDeque<f64>,
    plus_dm_history: VecDeque<f64>,
    minus_dm_history: VecDeque<f64>,
    dx_history: VecDeque<f64>,
    current_adx: Option<f64>,
}

impl Adx {
    /// Create a new ADX.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            samples: 0,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            tr_history: VecDeque::with_capacity(period),
            plus_dm_history: VecDeque::with_capacity(period),
            minus_dm_history: VecDeque::with_capacity(period),
            dx_history: VecDeque::with_capacity(period),
            current_adx: None,
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

        // Calculate True Range (TR)
        let tr = if let Some(prev_close) = self.prev_close {
            let hl = high - low;
            let hpc = (high - prev_close).abs();
            let lpc = (low - prev_close).abs();
            hl.max(hpc).max(lpc)
        } else {
            high - low
        };

        // Calculate +DM and -DM
        let (plus_dm, minus_dm) = if let (Some(prev_h), Some(prev_l)) =
            (self.prev_high, self.prev_low)
        {
            let up_move = high - prev_h;
            let down_move = prev_l - low;

            let plus = if up_move > down_move && up_move > 0.0 {
                up_move
            } else {
                0.0
            };
            let minus = if down_move > up_move && down_move > 0.0 {
                down_move
            } else {
                0.0
            };
            (plus, minus)
        } else {
            (0.0, 0.0)
        };

        self.prev_high = Some(high);
        self.prev_low = Some(low);
        self.prev_close = Some(close);

        // Add to history
        self.tr_history.push_back(tr);
        self.plus_dm_history.push_back(plus_dm);
        self.minus_dm_history.push_back(minus_dm);

        if self.tr_history.len() > self.period {
            self.tr_history.pop_front();
            self.plus_dm_history.pop_front();
            self.minus_dm_history.pop_front();
        }

        self.samples += 1;

        if self.samples >= self.period {
            // Calculate smoothed values (Wilder's smoothing)
            let atr: f64 = self.tr_history.iter().sum::<f64>() / self.period as f64;
            let smooth_plus_dm: f64 =
                self.plus_dm_history.iter().sum::<f64>() / self.period as f64;
            let smooth_minus_dm: f64 =
                self.minus_dm_history.iter().sum::<f64>() / self.period as f64;

            // Calculate +DI and -DI
            let plus_di = if atr != 0.0 {
                (smooth_plus_dm / atr) * 100.0
            } else {
                0.0
            };
            let minus_di = if atr != 0.0 {
                (smooth_minus_dm / atr) * 100.0
            } else {
                0.0
            };

            // Calculate DX
            let di_sum = plus_di + minus_di;
            let dx = if di_sum != 0.0 {
                ((plus_di - minus_di).abs() / di_sum) * 100.0
            } else {
                0.0
            };

            self.dx_history.push_back(dx);
            if self.dx_history.len() > self.period {
                self.dx_history.pop_front();
            }

            // ADX is the smoothed average of DX
            if self.dx_history.len() >= self.period {
                let adx = self.dx_history.iter().sum::<f64>() / self.dx_history.len() as f64;
                self.current_adx = Some(adx);
            }
        }

        IndicatorValue {
            value: f64_to_dec(self.current_adx.unwrap_or(0.0)),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        self.current_adx.map(|v| IndicatorValue {
            value: f64_to_dec(v),
            signal: None,
        })
    }

    fn warmup_period(&self) -> usize {
        self.period * 2 // ADX needs 2x period for proper warmup
    }

    fn is_ready(&self) -> bool {
        self.samples >= self.period * 2
    }
}

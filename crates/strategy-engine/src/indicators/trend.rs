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
/// Implements Wilder's Smoothing Method as described in "New Concepts in
/// Technical Trading Systems" (J. Welles Wilder Jr., 1978).
///
/// Uses Wilder's exponential smoothing formula:
/// `Smoothed = [(Previous × (Period - 1)) + Current] / Period`
///
/// This produces values compatible with standard trading platforms (TradingView,
/// MetaTrader, TA-Lib).
pub struct Adx {
    period: usize,
    samples: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    // Raw values for initial accumulation
    tr_accumulator: f64,
    plus_dm_accumulator: f64,
    minus_dm_accumulator: f64,
    dx_accumulator: f64,
    // Wilder-smoothed values
    smoothed_tr: Option<f64>,
    smoothed_plus_dm: Option<f64>,
    smoothed_minus_dm: Option<f64>,
    smoothed_adx: Option<f64>,
    current_adx: Option<f64>,
}

impl Adx {
    /// Create a new ADX indicator with Wilder's Smoothing.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            samples: 0,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            tr_accumulator: 0.0,
            plus_dm_accumulator: 0.0,
            minus_dm_accumulator: 0.0,
            dx_accumulator: 0.0,
            smoothed_tr: None,
            smoothed_plus_dm: None,
            smoothed_minus_dm: None,
            smoothed_adx: None,
            current_adx: None,
        }
    }

    /// Apply Wilder's smoothing formula: (prev * (period - 1) + current) / period
    #[inline]
    fn wilder_smooth(&self, prev: f64, current: f64) -> f64 {
        (prev * (self.period as f64 - 1.0) + current) / self.period as f64
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
        self.samples += 1;

        // Phase 1: Accumulate raw values for initial period
        if self.samples <= self.period {
            self.tr_accumulator += tr;
            self.plus_dm_accumulator += plus_dm;
            self.minus_dm_accumulator += minus_dm;

            // At end of first period, initialize smoothed values with simple average
            if self.samples == self.period {
                self.smoothed_tr = Some(self.tr_accumulator / self.period as f64);
                self.smoothed_plus_dm = Some(self.plus_dm_accumulator / self.period as f64);
                self.smoothed_minus_dm = Some(self.minus_dm_accumulator / self.period as f64);
            }
        }
        // Phase 2: Apply Wilder's smoothing after initial period
        else if let (Some(prev_tr), Some(prev_plus_dm), Some(prev_minus_dm)) = (
            self.smoothed_tr,
            self.smoothed_plus_dm,
            self.smoothed_minus_dm,
        ) {
            // Wilder's smoothing: (prev * (period - 1) + current) / period
            self.smoothed_tr = Some(self.wilder_smooth(prev_tr, tr));
            self.smoothed_plus_dm = Some(self.wilder_smooth(prev_plus_dm, plus_dm));
            self.smoothed_minus_dm = Some(self.wilder_smooth(prev_minus_dm, minus_dm));
        }

        // Calculate DI values and DX once we have smoothed values
        if let (Some(atr), Some(smooth_plus_dm), Some(smooth_minus_dm)) = (
            self.smoothed_tr,
            self.smoothed_plus_dm,
            self.smoothed_minus_dm,
        ) {
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

            // Phase 3: Accumulate DX for second period (ADX calculation)
            let dx_samples = self.samples - self.period;
            if dx_samples <= self.period {
                self.dx_accumulator += dx;

                // At end of second period, initialize ADX with simple average of DX
                if dx_samples == self.period {
                    self.smoothed_adx = Some(self.dx_accumulator / self.period as f64);
                    self.current_adx = self.smoothed_adx;
                }
            }
            // Phase 4: Apply Wilder's smoothing to ADX after 2*period
            else if let Some(prev_adx) = self.smoothed_adx {
                let new_adx = self.wilder_smooth(prev_adx, dx);
                self.smoothed_adx = Some(new_adx);
                self.current_adx = Some(new_adx);
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

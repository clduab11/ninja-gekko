use crate::indicators::{buffer, dec_to_f64, f64_to_dec, Indicator, IndicatorValue};
use rust_decimal::Decimal;
use std::collections::VecDeque;

// ============================================================================
// OBV
// ============================================================================
/// On-Balance Volume (OBV).
///
/// A technical momentum indicator that uses volume flow to predict changes in stock price.
pub struct Obv {
    prev_close: Option<f64>,
    cumulative: f64,
    samples: usize,
}

impl Obv {
    /// Create a new On-Balance Volume indicator.
    pub fn new() -> Self {
        Self {
            prev_close: None,
            cumulative: 0.0,
            samples: 0,
        }
    }
}

impl Indicator for Obv {
    fn name(&self) -> &'static str {
        "OBV"
    }

    fn update(&mut self, _price: Decimal) -> IndicatorValue {
        IndicatorValue {
            value: Decimal::ZERO,
            signal: None,
        }
    }

    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        let c = dec_to_f64(candle.close);
        let v = dec_to_f64(candle.volume);

        if let Some(prev) = self.prev_close {
            if c > prev {
                self.cumulative += v;
            } else if c < prev {
                self.cumulative -= v;
            }
        }

        self.prev_close = Some(c);
        self.samples += 1;

        IndicatorValue {
            value: f64_to_dec(self.cumulative),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        Some(IndicatorValue {
            value: f64_to_dec(self.cumulative),
            signal: None,
        })
    }

    fn warmup_period(&self) -> usize {
        1
    }
    fn is_ready(&self) -> bool {
        self.samples >= 1
    }
}

// ============================================================================
// VWAP
// ============================================================================
/// Volume Weighted Average Price (VWAP).
///
/// A trading benchmark used by traders that gives the average price a security has traded at throughout the day, based on both volume and price.
pub struct Vwap {
    cum_pv: f64,
    cum_vol: f64,
    samples: usize,
    // Note: Production VWAP resets daily. This is cumulative (lifetime).
}

impl Vwap {
    /// Create a new VWAP indicator.
    pub fn new() -> Self {
        Self {
            cum_pv: 0.0,
            cum_vol: 0.0,
            samples: 0,
        }
    }
}

impl Indicator for Vwap {
    fn name(&self) -> &'static str {
        "VWAP"
    }

    fn update(&mut self, _price: Decimal) -> IndicatorValue {
        IndicatorValue {
            value: Decimal::ZERO,
            signal: None,
        }
    }

    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        let h = dec_to_f64(candle.high);
        let l = dec_to_f64(candle.low);
        let c = dec_to_f64(candle.close);
        let v = dec_to_f64(candle.volume);

        let tp = (h + l + c) / 3.0;

        self.cum_pv += tp * v;
        self.cum_vol += v;
        self.samples += 1;

        let vwap = if self.cum_vol != 0.0 {
            self.cum_pv / self.cum_vol
        } else {
            0.0
        };

        IndicatorValue {
            value: f64_to_dec(vwap),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        if self.cum_vol == 0.0 {
            return None;
        }
        Some(IndicatorValue {
            value: f64_to_dec(self.cum_pv / self.cum_vol),
            signal: None,
        })
    }

    fn warmup_period(&self) -> usize {
        1
    }
    fn is_ready(&self) -> bool {
        self.samples >= 1
    }
}

// ============================================================================
// MFI (Stubbed/Simple)
// ============================================================================
/// Money Flow Index (MFI).
///
/// An oscillator that uses both price and volume to measure buying and selling pressure.
pub struct Mfi {
    period: usize,
    samples: usize,
    history: VecDeque<(f64, f64)>, // (TP, Vol)
}

impl Mfi {
    /// Create a new MFI indicator (Stubbed).
    pub fn new(period: usize) -> Self {
        Self {
            period,
            samples: 0,
            history: VecDeque::new(),
        }
    }
}

impl Indicator for Mfi {
    fn name(&self) -> &'static str {
        "MFI"
    }

    fn update(&mut self, _price: Decimal) -> IndicatorValue {
        IndicatorValue {
            value: Decimal::ZERO,
            signal: None,
        }
    }

    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        let h = dec_to_f64(candle.high);
        let l = dec_to_f64(candle.low);
        let c = dec_to_f64(candle.close);
        let v = dec_to_f64(candle.volume);
        let tp = (h + l + c) / 3.0;

        self.history.push_back((tp, v));
        if self.history.len() > self.period + 1 {
            self.history.pop_front();
        }
        self.samples += 1;

        // Simplify: return 50.0 until full impl logic
        unimplemented!("MFI logic pending");
    }

    fn current(&self) -> Option<IndicatorValue> {
        Some(IndicatorValue {
            value: Decimal::from(50),
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

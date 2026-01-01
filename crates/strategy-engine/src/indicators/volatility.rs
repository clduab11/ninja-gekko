use crate::indicators::{buffer, dec_to_f64, f64_to_dec, Indicator, IndicatorValue};
use rust_decimal::Decimal;
use std::collections::VecDeque;
use yata::core::{Method, PeriodType};
use yata::methods::{EMA, SMA};

// ============================================================================
// ATR
// ============================================================================
/// Average True Range (ATR).
///
/// A market volatility indicator derived from the moving average of the true range.
pub struct Atr {
    ema: EMA, // ATR is smoothed TR
    prev_close: Option<f64>,
    samples: usize,
    period: usize,
}

impl Atr {
    /// Create a new ATR indicator.
    pub fn new(period: usize) -> Self {
        Self {
            ema: EMA::new(period as PeriodType, &0.0).unwrap(),
            prev_close: None,
            samples: 0,
            period,
        }
    }
}

impl Indicator for Atr {
    fn name(&self) -> &'static str {
        "ATR"
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

        let tr = if let Some(prev_c) = self.prev_close {
            let hl = h - l;
            let h_pc = (h - prev_c).abs();
            let l_pc = (l - prev_c).abs();
            hl.max(h_pc).max(l_pc)
        } else {
            h - l
        };

        self.prev_close = Some(c);
        let atr = self.ema.next(&tr); // Smoothing TR
        self.samples += 1;

        IndicatorValue {
            value: f64_to_dec(atr),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        None
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

// ============================================================================
// Bollinger Bands
// ============================================================================
/// Bollinger Bands.
///
/// Defined by a set of trendlines plotted two standard deviations (positively and negatively)
/// away from a simple moving average (SMA) of a security's price.
pub struct BollingerBands {
    sma: SMA,
    sigma: f64,
    period: usize,
    samples: usize,
    history: VecDeque<f64>,
    current_upper: Option<f64>,
    current_lower: Option<f64>,
    current_mid: Option<f64>,
}

impl BollingerBands {
    /// Create new Bollinger Bands with specified period and standard deviation multiplier (sigma).
    pub fn new(period: usize, sigma: f64) -> Self {
        Self {
            sma: SMA::new(period as PeriodType, &0.0).unwrap(),
            sigma,
            period,
            samples: 0,
            history: VecDeque::new(),
            current_upper: None,
            current_lower: None,
            current_mid: None,
        }
    }
}

impl Indicator for BollingerBands {
    fn name(&self) -> &'static str {
        "Bollinger Bands"
    }

    fn update(&mut self, price: Decimal) -> IndicatorValue {
        let val = dec_to_f64(price);
        self.history.push_back(val);
        if self.history.len() > self.period {
            self.history.pop_front();
        }

        let mid = self.sma.next(&val);

        let mut dev = 0.0;
        if self.history.len() >= self.period {
            let variance: f64 =
                self.history.iter().map(|&x| (x - mid).powi(2)).sum::<f64>() / self.period as f64;
            dev = variance.sqrt();
        }

        let upper = mid + self.sigma * dev;
        let lower = mid - self.sigma * dev;

        self.current_upper = Some(upper);
        self.current_lower = Some(lower);
        self.current_mid = Some(mid);
        self.samples += 1;

        // Return Middle Band as value, Upper/Lower?
        // IndicatorValue limitation: 1 value + 1 signal.
        // Let's return %B? Or just Middle.
        // Or return Upper/Lower as specific usage via casting?
        // Standard: Value = Mid. Check `MomentumStrategy` usage? It doesn't use BB yet.
        IndicatorValue {
            value: f64_to_dec(upper), // Returning Upper for Breakout check usually?
            signal: Some(f64_to_dec(lower)),
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        match (self.current_upper, self.current_lower) {
            (Some(u), Some(l)) => Some(IndicatorValue {
                value: f64_to_dec(u),
                signal: Some(f64_to_dec(l)),
            }),
            _ => None,
        }
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

// ============================================================================
// Keltner Channels
// ============================================================================
/// Keltner Channels.
///
/// Volatility-based bands that are placed above and below an exponential moving average.
pub struct KeltnerChannels {
    ema: EMA,
    atr: Atr, // Use our Atr struct
    multiplier: f64,
    period: usize,
    samples: usize,
    current_upper: Option<f64>,
    current_lower: Option<f64>,
}

impl KeltnerChannels {
    /// Create new Keltner Channels with specified period and ATR multiplier.
    pub fn new(period: usize, multiplier: f64) -> Self {
        Self {
            ema: EMA::new(period as PeriodType, &0.0).unwrap(),
            atr: Atr::new(period),
            multiplier,
            period,
            samples: 0,
            current_upper: None,
            current_lower: None,
        }
    }
}

impl Indicator for KeltnerChannels {
    fn name(&self) -> &'static str {
        "Keltner Channels"
    }

    fn update(&mut self, _price: Decimal) -> IndicatorValue {
        IndicatorValue {
            value: Decimal::ZERO,
            signal: None,
        }
    }

    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        let val = dec_to_f64(candle.close); // EMA on Close typically
        let mid = self.ema.next(&val);

        let atr_val = self.atr.update_ohlcv(candle).value; // Update ATR
        let atr_f64 = dec_to_f64(atr_val);

        let upper = mid + self.multiplier * atr_f64;
        let lower = mid - self.multiplier * atr_f64;

        self.current_upper = Some(upper);
        self.current_lower = Some(lower);
        self.samples += 1;

        IndicatorValue {
            value: f64_to_dec(upper),
            signal: Some(f64_to_dec(lower)),
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        match (self.current_upper, self.current_lower) {
            (Some(u), Some(l)) => Some(IndicatorValue {
                value: f64_to_dec(u),
                signal: Some(f64_to_dec(l)),
            }),
            _ => None,
        }
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

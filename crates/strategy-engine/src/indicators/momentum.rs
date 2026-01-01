use crate::indicators::{buffer, dec_to_f64, f64_to_dec, Indicator, IndicatorValue};
use rust_decimal::Decimal;
use std::collections::VecDeque;

// ============================================================================
// RSI
// ============================================================================
/// Relative Strength Index (RSI).
///
/// Measures the speed and change of price movements.
/// RSI oscillates between zero and 100.
/// Traditionally, and according to Wilder, RSI is considered overbought when above 70 and oversold when below 30.
pub struct Rsi {
    period: usize,
    samples: usize,
    prev_val: Option<f64>,
    avg_gain: f64,
    avg_loss: f64,
}

impl Rsi {
    /// Create a new RSI indicator with the specified period.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            samples: 0,
            prev_val: None,
            avg_gain: 0.0,
            avg_loss: 0.0,
        }
    }
}

impl Indicator for Rsi {
    fn name(&self) -> &'static str {
        "RSI"
    }

    fn update(&mut self, price: Decimal) -> IndicatorValue {
        let val = dec_to_f64(price);

        if let Some(prev) = self.prev_val {
            let change = val - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            if self.samples < self.period {
                // Initial SMA phase for Wilder's (accumulation)
                self.avg_gain += gain;
                self.avg_loss += loss;
            } else if self.samples == self.period {
                // First average: Include the current (Nth) gain/loss, then divide
                self.avg_gain += gain;
                self.avg_loss += loss;
                self.avg_gain /= self.period as f64;
                self.avg_loss /= self.period as f64;
            } else {
                // Smoothing thereafter: (PREV * (N-1) + CURR) / N
                let n = self.period as f64;
                self.avg_gain = (self.avg_gain * (n - 1.0) + gain) / n;
                self.avg_loss = (self.avg_loss * (n - 1.0) + loss) / n;
            }
        }

        self.prev_val = Some(val);
        self.samples += 1;

        let rs = if self.avg_loss == 0.0 {
            100.0 // Max RSI if no loss
        } else {
            self.avg_gain / self.avg_loss
        };

        let rsi = if self.avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + rs))
        };

        IndicatorValue {
            value: f64_to_dec(rsi),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        // Not perfectly stateless return, uses last calc state
        if self.samples < self.period {
            return None;
        }
        // Re-calcing RSI from state
        let rs = if self.avg_loss == 0.0 {
            100.0
        } else {
            self.avg_gain / self.avg_loss
        };
        let rsi = 100.0 - (100.0 / (1.0 + rs));
        Some(IndicatorValue {
            value: f64_to_dec(rsi),
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
// Stochastic
// ============================================================================
/// Stochastic Oscillator.
///
/// A momentum indicator comparing a particular closing price of a security to a range of its prices over a certain period of time.
/// The sensitivity of the oscillator to market movements is reducible by adjusting that time period or by taking a moving average of the result.
pub struct Stochastic {
    period: usize,
    samples: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    // Simple SMA for %K smoothing (using slow stochastic usually) or %D
    k_val: f64,
    d_val: f64,
}

impl Stochastic {
    /// Create a new Stochastic Oscillator.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            samples: 0,
            highs: VecDeque::new(),
            lows: VecDeque::new(),
            k_val: 0.0,
            d_val: 0.0,
        }
    }
}

impl Indicator for Stochastic {
    fn name(&self) -> &'static str {
        "Stochastic"
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

        self.highs.push_back(h);
        self.lows.push_back(l);
        if self.highs.len() > self.period {
            self.highs.pop_front();
            self.lows.pop_front();
        }

        self.samples += 1;

        if self.samples >= self.period {
            // Find highest high and lowest low
            let highest_high = self.highs.iter().fold(f64::MIN, |a, &b| a.max(b));
            let lowest_low = self.lows.iter().fold(f64::MAX, |a, &b| a.min(b));

            let numerator = c - lowest_low;
            let denominator = highest_high - lowest_low;

            if denominator == 0.0 {
                self.k_val = 50.0;
            } else {
                self.k_val = (numerator / denominator) * 100.0;
            }
            // Usually we smooth K to get D (e.g. SMA 3 of K)
            // For simplicity in manual: D = K here.
            self.d_val = self.k_val;
        }

        IndicatorValue {
            value: f64_to_dec(self.k_val),
            signal: Some(f64_to_dec(self.d_val)),
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        Some(IndicatorValue {
            value: f64_to_dec(self.k_val),
            signal: Some(f64_to_dec(self.d_val)),
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
// CCI
// ============================================================================
/// Commodity Channel Index (CCI).
///
/// oscillate between +100 and -100.
pub struct Cci {
    period: usize,
    samples: usize,
    typical_prices: VecDeque<f64>,
}

impl Cci {
    /// Create a new CCI indicator.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            samples: 0,
            typical_prices: VecDeque::new(),
        }
    }
}

impl Indicator for Cci {
    fn name(&self) -> &'static str {
        "CCI"
    }

    fn update(&mut self, _price: Decimal) -> IndicatorValue {
        IndicatorValue {
            value: Decimal::ZERO,
            signal: None,
        }
    }

    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        let tp =
            (dec_to_f64(candle.high) + dec_to_f64(candle.low) + dec_to_f64(candle.close)) / 3.0;
        self.typical_prices.push_back(tp);
        if self.typical_prices.len() > self.period {
            self.typical_prices.pop_front();
        }
        self.samples += 1;

        let mut cci = 0.0;
        if self.samples >= self.period {
            let sum: f64 = self.typical_prices.iter().sum();
            let sma = sum / self.period as f64;
            let mean_deviation: f64 = self
                .typical_prices
                .iter()
                .map(|&p| (p - sma).abs())
                .sum::<f64>()
                / self.period as f64;

            if mean_deviation != 0.0 {
                cci = (tp - sma) / (0.015 * mean_deviation);
            }
        }

        IndicatorValue {
            value: f64_to_dec(cci),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        None // Stateful, hard to return pure current without recalculating
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

// ============================================================================
// Williams %R (Stubbed/Manual)
// ============================================================================
/// Williams %R.
///
/// A momentum indicator that measures overbought and oversold levels.
pub struct WilliamsR {
    samples: usize,
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
}

impl WilliamsR {
    /// Create a new Williams %R indicator.
    pub fn new(period: usize) -> Self {
        Self {
            samples: 0,
            period,
            highs: VecDeque::new(),
            lows: VecDeque::new(),
        }
    }
}

impl Indicator for WilliamsR {
    fn name(&self) -> &'static str {
        "Williams %R"
    }

    fn update(&mut self, _price: Decimal) -> IndicatorValue {
        IndicatorValue {
            value: Decimal::ZERO,
            signal: None,
        }
    }

    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        // %R = (Highest High - Close) / (Highest High - Lowest Low) * -100
        let h = dec_to_f64(candle.high);
        let l = dec_to_f64(candle.low);
        let c = dec_to_f64(candle.close);

        self.highs.push_back(h);
        self.lows.push_back(l);
        if self.highs.len() > self.period {
            self.highs.pop_front();
            self.lows.pop_front();
        }
        self.samples += 1;

        let mut wpr = 0.0;
        if self.samples >= self.period {
            let highest_high = self.highs.iter().fold(f64::MIN, |a, &b| a.max(b));
            let lowest_low = self.lows.iter().fold(f64::MAX, |a, &b| a.min(b));

            let range = highest_high - lowest_low;
            if range != 0.0 {
                wpr = ((highest_high - c) / range) * -100.0;
            }
        }

        IndicatorValue {
            value: f64_to_dec(wpr),
            signal: None,
        }
    }

    fn current(&self) -> Option<IndicatorValue> {
        None // Stateful
    }

    fn warmup_period(&self) -> usize {
        self.period
    }
    fn is_ready(&self) -> bool {
        self.samples >= self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn rsi_warmup_period() {
        let rsi = Rsi::new(14);
        assert_eq!(rsi.warmup_period(), 14);
        assert!(!rsi.is_ready());
    }

    #[test]
    fn rsi_bounds() {
        let mut rsi = Rsi::new(14);

        // Feed ascending prices (should trend toward 100)
        for i in 1..=20 {
            let result = rsi.update(Decimal::from(i * 10));
            if rsi.is_ready() {
                assert!(result.value >= dec!(0));
                assert!(result.value <= dec!(100));
            }
        }
    }

    #[test]
    fn rsi_overbought_condition() {
        let mut rsi = Rsi::new(14);

        // Feed flat then strong uptrend
        for _ in 0..14 {
            rsi.update(dec!(100));
        }

        // Strong uptrend
        for i in 1..=10 {
            rsi.update(Decimal::from(100 + i * 5));
        }

        let current = rsi.current().unwrap();
        // With sustained uptrend, RSI should be high.
        assert!(
            current.value > dec!(50),
            "RSI should be elevated in uptrend"
        );
    }
}

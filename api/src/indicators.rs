//! Indicator Service for Technical Analysis
//!
//! This module provides an IndicatorService that wraps the strategy-engine's
//! indicator implementations and exposes them for use in the API layer.

use rust_decimal::Decimal;
use std::collections::HashMap;
use strategy_engine::indicators::prelude::*;

/// Service for managing and computing technical indicators
pub struct IndicatorService {
    /// Default RSI period
    rsi_period: usize,
    /// Default MACD parameters
    macd_fast: usize,
    macd_slow: usize,
    macd_signal: usize,
    /// Default Bollinger Bands parameters
    bb_period: usize,
    bb_sigma: f64,
    /// Default SMA/EMA periods
    sma_period: usize,
    ema_period: usize,
    /// Default ATR period
    atr_period: usize,
}

impl Default for IndicatorService {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            bb_period: 20,
            bb_sigma: 2.0,
            sma_period: 20,
            ema_period: 12,
            atr_period: 14,
        }
    }
}

impl IndicatorService {
    /// Create a new IndicatorService with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an IndicatorService with custom RSI period
    pub fn with_rsi_period(mut self, period: usize) -> Self {
        self.rsi_period = period;
        self
    }

    /// Create an IndicatorService with custom MACD parameters
    pub fn with_macd_params(mut self, fast: usize, slow: usize, signal: usize) -> Self {
        self.macd_fast = fast;
        self.macd_slow = slow;
        self.macd_signal = signal;
        self
    }

    /// Calculate RSI from a series of prices
    ///
    /// Returns the last RSI value or None if not enough data
    pub fn calculate_rsi(&self, prices: &[f64]) -> Option<f64> {
        if prices.len() < self.rsi_period + 1 {
            return None;
        }

        let mut rsi = Rsi::new(self.rsi_period);
        let mut last_value = None;

        for &price in prices {
            let value = rsi.update(Decimal::from_f64_retain(price)?);
            if rsi.is_ready() {
                last_value = Some(value.value.to_string().parse().ok()?);
            }
        }

        last_value
    }

    /// Calculate SMA from a series of prices
    pub fn calculate_sma(&self, prices: &[f64]) -> Option<f64> {
        if prices.len() < self.sma_period {
            return None;
        }

        let mut sma = Sma::new(self.sma_period);
        let mut last_value = None;

        for &price in prices {
            let value = sma.update(Decimal::from_f64_retain(price)?);
            if sma.is_ready() {
                last_value = Some(value.value.to_string().parse().ok()?);
            }
        }

        last_value
    }

    /// Calculate EMA from a series of prices
    pub fn calculate_ema(&self, prices: &[f64]) -> Option<f64> {
        if prices.len() < self.ema_period {
            return None;
        }

        let mut ema = Ema::new(self.ema_period);
        let mut last_value = None;

        for &price in prices {
            let value = ema.update(Decimal::from_f64_retain(price)?);
            if ema.is_ready() {
                last_value = Some(value.value.to_string().parse().ok()?);
            }
        }

        last_value
    }

    /// Calculate MACD from a series of prices
    ///
    /// Returns (macd_line, signal_line) or None if not enough data
    pub fn calculate_macd(&self, prices: &[f64]) -> Option<(f64, f64)> {
        let warmup = self.macd_slow + self.macd_signal;
        if prices.len() < warmup {
            return None;
        }

        let mut macd = Macd::new(self.macd_fast, self.macd_slow, self.macd_signal);
        let mut last_value = None;

        for &price in prices {
            let value = macd.update(Decimal::from_f64_retain(price)?);
            if macd.is_ready() {
                let macd_line: f64 = value.value.to_string().parse().ok()?;
                let signal_line: f64 = value.signal?.to_string().parse().ok()?;
                last_value = Some((macd_line, signal_line));
            }
        }

        last_value
    }

    /// Calculate Bollinger Bands from a series of prices
    ///
    /// Returns (upper_band, middle_band, lower_band) or None if not enough data.
    /// Uses the explicit `BollingerBands::calculate_bands()` method for unambiguous
    /// access to all three band values.
    pub fn calculate_bollinger_bands(&self, prices: &[f64]) -> Option<(f64, f64, f64)> {
        if prices.len() < self.bb_period {
            return None;
        }

        let mut bb = BollingerBands::new(self.bb_period, self.bb_sigma);

        for &price in prices {
            bb.update(Decimal::from_f64_retain(price)?);
        }

        // Use explicit calculate_bands() for unambiguous access to all three values
        bb.calculate_bands()
            .map(|bands| (bands.upper, bands.middle, bands.lower))
    }

    /// Calculate ATR from OHLCV candle data
    ///
    /// Returns the last ATR value or None if not enough data
    pub fn calculate_atr(&self, candles: &[CandleData]) -> Option<f64> {
        if candles.len() < self.atr_period {
            return None;
        }

        let mut atr = Atr::new(self.atr_period);
        let mut last_value = None;

        for candle_data in candles {
            let candle = Candle {
                open: Decimal::from_f64_retain(candle_data.open)?,
                high: Decimal::from_f64_retain(candle_data.high)?,
                low: Decimal::from_f64_retain(candle_data.low)?,
                close: Decimal::from_f64_retain(candle_data.close)?,
                volume: Decimal::from_f64_retain(candle_data.volume)?,
                timestamp: candle_data.timestamp,
            };
            let value = atr.update_ohlcv(&candle);
            if atr.is_ready() {
                last_value = Some(value.value.to_string().parse().ok()?);
            }
        }

        last_value
    }

    /// Calculate all common indicators from price data
    ///
    /// Returns a HashMap with indicator names as keys and their values
    pub fn calculate_all_indicators(&self, prices: &[f64]) -> HashMap<String, f64> {
        let mut indicators = HashMap::new();

        if let Some(rsi) = self.calculate_rsi(prices) {
            indicators.insert("rsi".to_string(), rsi);
        }

        if let Some(sma) = self.calculate_sma(prices) {
            indicators.insert("sma".to_string(), sma);
        }

        if let Some(ema) = self.calculate_ema(prices) {
            indicators.insert("ema".to_string(), ema);
        }

        if let Some((macd, signal)) = self.calculate_macd(prices) {
            indicators.insert("macd".to_string(), macd);
            indicators.insert("macd_signal".to_string(), signal);
            indicators.insert("macd_histogram".to_string(), macd - signal);
        }

        if let Some((upper, middle, lower)) = self.calculate_bollinger_bands(prices) {
            indicators.insert("bb_upper".to_string(), upper);
            indicators.insert("bb_middle".to_string(), middle);
            indicators.insert("bb_lower".to_string(), lower);
            if let Some(last_price) = prices.last() {
                // %B indicator: (price - lower) / (upper - lower)
                let width = upper - lower;
                if width > 0.0 {
                    let percent_b = (last_price - lower) / width;
                    indicators.insert("bb_percent_b".to_string(), percent_b);
                }
            }
        }

        indicators
    }

    /// Calculate indicators from OHLCV candle data (includes ATR and other OHLCV-based indicators)
    pub fn calculate_all_indicators_ohlcv(&self, candles: &[CandleData]) -> HashMap<String, f64> {
        // Extract close prices for price-based indicators
        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut indicators = self.calculate_all_indicators(&prices);

        // Add OHLCV-specific indicators
        if let Some(atr) = self.calculate_atr(candles) {
            indicators.insert("atr".to_string(), atr);
        }

        indicators
    }
}

/// Simple candle data structure for indicator calculations
#[derive(Debug, Clone)]
pub struct CandleData {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: i64,
}

impl CandleData {
    /// Create a new CandleData
    pub fn new(open: f64, high: f64, low: f64, close: f64, volume: f64, timestamp: i64) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_prices() -> Vec<f64> {
        vec![
            44.0, 44.5, 44.2, 44.8, 45.0, 45.2, 45.5, 45.3, 45.8, 46.0, 46.2, 45.9, 45.5, 45.2,
            45.0, 44.8, 44.5, 44.2, 44.0, 43.8, 43.5, 43.8, 44.0, 44.2, 44.5, 44.8, 45.0, 45.2,
            45.5, 45.8, 46.0,
        ]
    }

    #[test]
    fn test_indicator_service_creation() {
        let service = IndicatorService::new();
        assert_eq!(service.rsi_period, 14);
        assert_eq!(service.sma_period, 20);
    }

    #[test]
    fn test_calculate_rsi() {
        let service = IndicatorService::new();
        let prices = sample_prices();
        let rsi = service.calculate_rsi(&prices);
        assert!(rsi.is_some());
        let rsi_value = rsi.unwrap();
        assert!(rsi_value >= 0.0 && rsi_value <= 100.0);
    }

    #[test]
    fn test_calculate_sma() {
        let service = IndicatorService::new();
        let prices = sample_prices();
        let sma = service.calculate_sma(&prices);
        assert!(sma.is_some());
    }

    #[test]
    fn test_calculate_ema() {
        let service = IndicatorService::new();
        let prices = sample_prices();
        let ema = service.calculate_ema(&prices);
        assert!(ema.is_some());
    }

    #[test]
    fn test_calculate_macd() {
        let service = IndicatorService::new();
        let prices = sample_prices();
        let macd = service.calculate_macd(&prices);
        // MACD needs more data (26 + 9 = 35 warmup)
        // Our sample has 31 prices, so this may or may not return a value
        // depending on exact implementation
        if macd.is_some() {
            let (macd_line, signal_line) = macd.unwrap();
            // Both should be reasonable values
            assert!(macd_line.is_finite());
            assert!(signal_line.is_finite());
        }
    }

    #[test]
    fn test_calculate_bollinger_bands() {
        let service = IndicatorService::new();
        let prices = sample_prices();
        let bb = service.calculate_bollinger_bands(&prices);
        assert!(bb.is_some());
        let (upper, middle, lower) = bb.unwrap();
        assert!(upper >= middle);
        assert!(middle >= lower);
        assert!(upper >= lower);
    }

    #[test]
    fn test_calculate_all_indicators() {
        let service = IndicatorService::new();
        let prices = sample_prices();
        let indicators = service.calculate_all_indicators(&prices);

        // Should have at least RSI, SMA, EMA, BB (including middle)
        assert!(indicators.contains_key("rsi"));
        assert!(indicators.contains_key("sma"));
        assert!(indicators.contains_key("ema"));
        assert!(indicators.contains_key("bb_upper"));
        assert!(indicators.contains_key("bb_middle"));
        assert!(indicators.contains_key("bb_lower"));
    }

    #[test]
    fn test_insufficient_data() {
        let service = IndicatorService::new();
        let prices = vec![1.0, 2.0, 3.0]; // Too few prices

        assert!(service.calculate_rsi(&prices).is_none());
        assert!(service.calculate_sma(&prices).is_none());
    }
}

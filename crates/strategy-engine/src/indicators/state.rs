use crate::indicators::prelude::*;

/// Strategy-owned indicator state
pub struct IndicatorState {
    pub buffer: CandleBuffer,
    pub indicators: Vec<Box<dyn Indicator>>,
}

impl IndicatorState {
    pub fn new(buffer_depth: usize) -> Self {
        Self {
            buffer: CandleBuffer::new(buffer_depth),
            indicators: Vec::new(),
        }
    }

    pub fn add<I: Indicator + 'static>(&mut self, indicator: I) -> &mut Self {
        self.indicators.push(Box::new(indicator));
        self
    }

    /// Update all indicators with new candle
    pub fn update(&mut self, candle: Candle) -> Vec<IndicatorValue> {
        self.buffer.push(candle.clone());
        self.indicators
            .iter_mut()
            .map(|ind| ind.update_ohlcv(&candle))
            .collect()
    }
}

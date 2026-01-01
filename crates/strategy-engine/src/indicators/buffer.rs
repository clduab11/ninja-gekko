use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// OHLCV candle for indicator calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub timestamp: i64,
}

/// Ring buffer with configurable depth for indicator lookback
#[derive(Debug, Clone)]
pub struct CandleBuffer {
    inner: VecDeque<Candle>,
    capacity: usize,
}

impl CandleBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push candle, evicting oldest if at capacity. Returns evicted candle if any.
    pub fn push(&mut self, candle: Candle) -> Option<Candle> {
        let evicted = if self.inner.len() >= self.capacity {
            self.inner.pop_front()
        } else {
            None
        };
        self.inner.push_back(candle);
        evicted
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.inner.len() >= self.capacity
    }

    /// Get last N candles (most recent last)
    pub fn last_n(&self, n: usize) -> impl Iterator<Item = &Candle> {
        self.inner.iter().rev().take(n).rev()
    }

    /// Latest candle
    pub fn latest(&self) -> Option<&Candle> {
        self.inner.back()
    }
}

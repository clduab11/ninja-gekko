use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

pub mod buffer;
pub mod momentum;
pub mod state;
pub mod trend;
pub mod volatility;
pub mod volume;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndicatorValue {
    pub value: Decimal,
    pub signal: Option<Decimal>,
}

pub trait Indicator: Send {
    fn name(&self) -> &'static str;
    fn update(&mut self, price: Decimal) -> IndicatorValue;
    fn update_ohlcv(&mut self, candle: &buffer::Candle) -> IndicatorValue {
        self.update(candle.close)
    }
    fn current(&self) -> Option<IndicatorValue>;
    fn warmup_period(&self) -> usize;
    fn is_ready(&self) -> bool;
}

pub fn dec_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

pub fn f64_to_dec(f: f64) -> Decimal {
    Decimal::from_f64(f).unwrap_or(Decimal::ZERO)
}

pub mod prelude {
    pub use super::buffer::{Candle, CandleBuffer};
    pub use super::Indicator;
    pub use super::IndicatorValue;

    // Re-export common indicators
    pub use super::momentum::{Cci, Rsi, Stochastic, WilliamsR};
    pub use super::trend::{Adx, Ema, Macd, Sma};
    pub use super::volatility::{Atr, BollingerBands, BollingerBandsOutput, KeltnerChannels};
    pub use super::volume::{Mfi, Obv, Vwap};
}

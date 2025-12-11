use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use ahash::AHashMap;
use event_bus::{EventMetadata, EventSource, MarketEvent, MarketPayload, Priority};
use exchange_connectors::{ExchangeId, ExchangeOrder, StreamMessage, TradingPair};
use tracing::debug;

use crate::ingestion::RawMarketMessage;
use crate::order_book::{LevelTwoBook, OrderBookUpdate};

/// Sequence generator shared by all normalizers.
static GLOBAL_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Normalized event emitted by the normalization stage.
pub struct NormalizedEvent {
    pub exchange: ExchangeId,
    pub event: MarketEvent,
}

/// Normalizer that transforms raw WebSocket payloads into market events.
pub struct MarketNormalizer {
    books: AHashMap<ExchangeId, LevelTwoBook>,
}

impl MarketNormalizer {
    pub fn new() -> Self {
        Self {
            books: AHashMap::new(),
        }
    }

    /// Normalizes a raw message into an optional `MarketEvent` envelope.
    pub fn normalize(&mut self, message: RawMarketMessage) -> Option<NormalizedEvent> {
        let (exchange, payload) = message;
        match payload {
            StreamMessage::Tick(tick) => {
                let metadata = self.metadata(exchange, Priority::High);
                let pair = self
                    .books
                    .entry(exchange)
                    .or_default()
                    .instrument()
                    .or_else(|| Self::parse_symbol(&tick.symbol))
                    .unwrap_or_else(|| Self::default_pair(&tick.symbol));
                let event = MarketEvent::new(metadata, MarketPayload::Tick { tick, pair });
                Some(NormalizedEvent { exchange, event })
            }
            StreamMessage::OrderUpdate(order_update) => {
                if let Some(update) = Self::convert_order(exchange, order_update) {
                    let book = self.books.entry(exchange).or_default();
                    let payload = book.apply(update);
                    let metadata = self.metadata(exchange, Priority::High);
                    let event = MarketEvent::new(metadata, payload);
                    Some(NormalizedEvent { exchange, event })
                } else {
                    None
                }
            }
            StreamMessage::Ping => None,
            StreamMessage::Pong => None,
            StreamMessage::Error(err) => {
                debug!(%err, "stream error from {:?}", exchange);
                None
            }
        }
    }

    fn metadata(&self, exchange: ExchangeId, priority: Priority) -> EventMetadata {
        let source = EventSource::new(format!("normalizer.{:?}", exchange).to_lowercase());
        let mut metadata = EventMetadata::new(source, priority);
        metadata.sequence = GLOBAL_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        metadata
    }

    fn convert_order(exchange: ExchangeId, order: ExchangeOrder) -> Option<OrderBookUpdate> {
        let price = order
            .price
            .or_else(|| order.fills.first().map(|fill| fill.price))?;
        let quantity = order
            .fills
            .first()
            .map(|fill| fill.quantity)
            .unwrap_or(order.quantity);
        let pair = match Self::parse_symbol(&order.symbol) {
            Some(pair) => pair,
            None => {
                debug!(
                    exchange = ?exchange,
                    symbol = %order.symbol,
                    "unable to parse trading pair from order update"
                );
                return None;
            }
        };
        let sequence = GLOBAL_SEQUENCE.fetch_add(1, Ordering::Relaxed);

        Some(OrderBookUpdate::new(
            pair, order.side, price, quantity, sequence,
        ))
    }

    fn parse_symbol(symbol: &str) -> Option<TradingPair> {
        let mut parts: VecDeque<&str> = symbol.split(['-', '_']).collect();
        if parts.len() >= 2 {
            let base = parts.pop_front()?.to_string();
            let quote = parts.pop_front()?.to_string();
            Some(TradingPair {
                base,
                quote,
                symbol: symbol.to_string(),
            })
        } else {
            None
        }
    }

    fn default_pair(symbol: &str) -> TradingPair {
        TradingPair {
            base: symbol.to_string(),
            quote: String::new(),
            symbol: symbol.to_string(),
        }
    }
}

impl Default for MarketNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exchange_connectors::MarketTick;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tick_normalization() {
        let tick = MarketTick {
            symbol: "BTC-USD".into(),
            bid: dec!(30000.0),
            ask: dec!(30010.0),
            last: dec!(30005.0),
            volume_24h: dec!(1234),
            timestamp: chrono::Utc::now(),
        };
        let raw = (ExchangeId::Kraken, StreamMessage::Tick(tick));

        let mut normalizer = MarketNormalizer::new();
        let normalized = normalizer.normalize(raw).expect("normalized");
        match normalized.event.payload() {
            MarketPayload::Tick { tick, .. } => {
                assert_eq!(tick.symbol, "BTC-USD");
            }
            _ => panic!("unexpected payload"),
        }
    }
}

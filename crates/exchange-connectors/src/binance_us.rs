//! Binance.us API Connector (exchange spot markets)
//!
//! The implementation focuses on low-latency public market data streaming so it
//! can feed the Ninja Gekko data pipeline without trading credentials. REST
//! endpoints remain stubbed until order routing is required.

use crate::{
    Balance, ExchangeConnector, ExchangeError, ExchangeId, ExchangeOrder, ExchangeResult, Fill,
    MarketTick, OrderSide, OrderStatus, OrderType, StreamMessage, TransferRequest, TransferStatus,
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use url::Url;

const BINANCE_US_WS_URL: &str = "wss://stream.binance.us:9443/ws";
const BINANCE_US_REST_URL: &str = "https://api.binance.us";

/// Lightweight connector plumbing focused on WebSocket ingestion.
pub struct BinanceUsConnector {
    inner: Arc<BinanceInner>,
}

struct BinanceInner {
    connected: AtomicBool,
    ws_url: Url,
    #[allow(dead_code)]
    rest_url: Url,
}

impl BinanceUsConnector {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(BinanceInner {
                connected: AtomicBool::new(false),
                ws_url: Url::parse(BINANCE_US_WS_URL).expect("valid Binance.us ws url"),
                rest_url: Url::parse(BINANCE_US_REST_URL).expect("valid Binance.us rest url"),
            }),
        }
    }
}

#[async_trait]
impl ExchangeConnector for BinanceUsConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::BinanceUs
    }

    async fn connect(&mut self) -> ExchangeResult<()> {
        info!("connecting to Binance.us");
        self.inner.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&mut self) -> ExchangeResult<()> {
        self.inner.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.inner.connected.load(Ordering::SeqCst)
    }

    async fn get_trading_pairs(&self) -> ExchangeResult<Vec<crate::TradingPair>> {
        // REST plumbing is out of scope for this phase.
        Ok(vec![])
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        Ok(vec![])
    }

    async fn place_order(
        &self,
        _symbol: &str,
        _side: OrderSide,
        _order_type: OrderType,
        _quantity: Decimal,
        _price: Option<Decimal>,
    ) -> ExchangeResult<ExchangeOrder> {
        Err(ExchangeError::InvalidRequest(
            "Trading not implemented for Binance.us connector".to_string(),
        ))
    }

    async fn cancel_order(&self, _order_id: &str) -> ExchangeResult<ExchangeOrder> {
        Err(ExchangeError::InvalidRequest(
            "Trading not implemented for Binance.us connector".to_string(),
        ))
    }

    async fn get_order(&self, _order_id: &str) -> ExchangeResult<ExchangeOrder> {
        Err(ExchangeError::InvalidRequest(
            "Trading not implemented for Binance.us connector".to_string(),
        ))
    }

    async fn get_market_data(&self, _symbol: &str) -> ExchangeResult<MarketTick> {
        Err(ExchangeError::InvalidRequest(
            "Use WebSocket streaming for real-time Binance.us data".to_string(),
        ))
    }

    async fn start_market_stream(
        &self,
        symbols: Vec<String>,
    ) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        if symbols.is_empty() {
            return Err(ExchangeError::InvalidRequest(
                "at least one symbol must be provided for Binance.us streaming".into(),
            ));
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let ws_url = self.inner.ws_url.clone();
        let mapping = Arc::new(build_symbol_mapping(&symbols));
        let subscriptions = Arc::new(build_subscription_params(&symbols));

        tokio::spawn(async move {
            if let Err(err) =
                run_binance_market_stream(ws_url, mapping, subscriptions, tx.clone()).await
            {
                error!(%err, "binance.us stream terminated with error");
            }
        });

        Ok(rx)
    }

    async fn start_order_stream(&self) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        let (_tx, rx) = mpsc::unbounded_channel();
        warn!("Binance.us private order stream requires authenticated keys; not yet implemented");
        Ok(rx)
    }

    async fn transfer_funds(&self, _request: TransferRequest) -> ExchangeResult<String> {
        Err(ExchangeError::InvalidRequest(
            "Fund transfers not implemented for Binance.us connector".to_string(),
        ))
    }

    async fn get_transfer_status(&self, _transfer_id: &str) -> ExchangeResult<TransferStatus> {
        Err(ExchangeError::InvalidRequest(
            "Fund transfers not implemented for Binance.us connector".to_string(),
        ))
    }

    async fn get_candles(
        &self,
        _symbol: &str,
        _timeframe: crate::Timeframe,
        _start: Option<chrono::DateTime<chrono::Utc>>,
        _end: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ExchangeResult<Vec<crate::Candle>> {
        Err(ExchangeError::InvalidRequest(
            "Historical candles not implemented for Binance.us connector".to_string(),
        ))
    }
}

fn build_symbol_mapping(symbols: &[String]) -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    for symbol in symbols {
        let stream_key = canonical_symbol(symbol);
        mapping.insert(stream_key.to_uppercase(), symbol.clone());
    }
    mapping
}

fn build_subscription_params(symbols: &[String]) -> Vec<String> {
    let mut params = Vec::with_capacity(symbols.len() * 2);
    for symbol in symbols {
        let stream_key = canonical_symbol(symbol);
        params.push(format!("{}@bookTicker", stream_key));
        params.push(format!("{}@depth@100ms", stream_key));
    }
    params
}

fn canonical_symbol(symbol: &str) -> String {
    symbol
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

async fn run_binance_market_stream(
    ws_url: Url,
    symbol_mapping: Arc<HashMap<String, String>>,
    subscriptions: Arc<Vec<String>>,
    sender: mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let mut attempt: u32 = 0;
    loop {
        attempt = attempt.saturating_add(1);
        debug!(attempt, url = %ws_url, "connecting to Binance.us websocket");

        match connect_async(ws_url.clone()).await {
            Ok((mut stream, _)) => {
                info!("binance.us websocket connected");
                attempt = 0;
                let subscribe = serde_json::json!({
                    "method": "SUBSCRIBE",
                    "params": subscriptions.as_slice(),
                    "id": chrono::Utc::now().timestamp_millis(),
                });
                if let Err(err) = stream.send(Message::Text(subscribe.to_string())).await {
                    warn!(%err, "failed to send Binance.us subscription");
                    continue;
                }

                while let Some(msg) = stream.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if text.contains("\"result\"") {
                                continue;
                            }
                            if let Err(err) =
                                handle_binance_payload(&text, &symbol_mapping, &sender)
                            {
                                warn!(%err, "failed to process Binance.us payload");
                            }
                        }
                        Ok(Message::Binary(bin)) => {
                            if let Ok(text) = String::from_utf8(bin) {
                                if let Err(err) =
                                    handle_binance_payload(&text, &symbol_mapping, &sender)
                                {
                                    warn!(%err, "failed to process Binance.us payload");
                                }
                            }
                        }
                        Ok(Message::Ping(payload)) => {
                            if let Err(err) = stream.send(Message::Pong(payload)).await {
                                warn!(%err, "failed to pong Binance.us");
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("binance.us websocket closed by peer");
                            break;
                        }
                        Err(err) => {
                            warn!(%err, "error on Binance.us websocket");
                            break;
                        }
                        _ => {}
                    }

                    if sender.is_closed() {
                        debug!("binance.us subscriber dropped channel; terminating stream");
                        return Ok(());
                    }
                }
            }
            Err(err) => {
                warn!(%err, "Binance.us websocket connection failed");
            }
        }

        if sender.is_closed() {
            debug!("binance.us subscriber dropped channel; terminating reconnect loop");
            return Ok(());
        }

        let delay = backoff_delay(attempt);
        warn!(
            ?delay,
            attempt, "reconnecting to Binance.us websocket after backoff"
        );
        sleep(delay).await;
    }
}

fn handle_binance_payload(
    payload: &str,
    symbol_mapping: &HashMap<String, String>,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let value: serde_json::Value = serde_json::from_str(payload)
        .map_err(|err| ExchangeError::Network(format!("invalid Binance.us payload: {err}")))?;

    let stream_name = match value.get("stream").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return Ok(()),
    };

    let data = match value.get("data") {
        Some(data) => data,
        None => return Ok(()),
    };

    if stream_name.ends_with("@bookTicker") {
        emit_book_ticker(data, symbol_mapping, sender)?;
    } else if stream_name.ends_with("@depth@100ms") {
        emit_depth_updates(data, symbol_mapping, sender)?;
    }

    Ok(())
}

fn emit_book_ticker(
    data: &serde_json::Value,
    symbol_mapping: &HashMap<String, String>,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let symbol = data
        .get("s")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing symbol in bookTicker".into()))?;
    let display_symbol = symbol_mapping
        .get(symbol)
        .cloned()
        .unwrap_or_else(|| symbol.to_string());

    let bid_price = parse_decimal(data.get("b"))?;
    let ask_price = parse_decimal(data.get("a"))?;
    let last_price = parse_decimal(data.get("c")).unwrap_or(ask_price);
    let volume = parse_decimal(data.get("B")).unwrap_or(Decimal::ZERO);

    let tick = MarketTick {
        symbol: display_symbol,
        bid: bid_price,
        ask: ask_price,
        last: last_price,
        volume_24h: volume,
        timestamp: chrono::Utc::now(),
    };

    let _ = sender.send(StreamMessage::Tick(tick));
    Ok(())
}

fn emit_depth_updates(
    data: &serde_json::Value,
    symbol_mapping: &HashMap<String, String>,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let symbol = data
        .get("s")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing symbol in depth update".into()))?;
    let display_symbol = symbol_mapping
        .get(symbol)
        .cloned()
        .unwrap_or_else(|| symbol.to_string());

    let event_time = data
        .get("E")
        .and_then(|v| v.as_u64())
        .map(timestamp_from_ms)
        .unwrap_or_else(chrono::Utc::now);

    if let Some(bids) = data.get("b").and_then(|v| v.as_array()) {
        for level in bids {
            let Some(items) = level.as_array() else {
                continue;
            };
            if items.len() < 2 {
                continue;
            }
            let price = parse_decimal(items.get(0))?;
            let quantity = parse_decimal(items.get(1))?;
            emit_order_event(
                sender,
                &display_symbol,
                OrderSide::Buy,
                price,
                quantity,
                event_time,
            );
        }
    }

    if let Some(asks) = data.get("a").and_then(|v| v.as_array()) {
        for level in asks {
            let Some(items) = level.as_array() else {
                continue;
            };
            if items.len() < 2 {
                continue;
            }
            let price = parse_decimal(items.get(0))?;
            let quantity = parse_decimal(items.get(1))?;
            emit_order_event(
                sender,
                &display_symbol,
                OrderSide::Sell,
                price,
                quantity,
                event_time,
            );
        }
    }

    Ok(())
}

fn emit_order_event(
    sender: &mpsc::UnboundedSender<StreamMessage>,
    symbol: &str,
    side: OrderSide,
    price: Decimal,
    quantity: Decimal,
    timestamp: chrono::DateTime<chrono::Utc>,
) {
    let side_tag = match side {
        OrderSide::Buy => "bid",
        OrderSide::Sell => "ask",
    };
    let order_id = format!(
        "depth-{}-{}-{}",
        symbol.replace(['-', '_'], "").to_lowercase(),
        side_tag,
        timestamp.timestamp_nanos_opt().unwrap_or(0)
    );

    let fill = Fill {
        id: format!("{}-fill", order_id),
        order_id: order_id.clone(),
        price,
        quantity,
        fee: Decimal::ZERO,
        timestamp,
    };

    let order = ExchangeOrder {
        id: order_id,
        exchange_id: ExchangeId::BinanceUs,
        symbol: symbol.to_string(),
        side,
        order_type: OrderType::Limit,
        quantity,
        price: Some(price),
        status: OrderStatus::Open,
        timestamp,
        fills: vec![fill],
    };

    let _ = sender.send(StreamMessage::OrderUpdate(order));
}

fn parse_decimal(value: Option<&serde_json::Value>) -> Result<Decimal, ExchangeError> {
    let Some(v) = value.and_then(|v| v.as_str()) else {
        return Ok(Decimal::ZERO);
    };
    Decimal::from_str(v).map_err(|err| {
        ExchangeError::Network(format!(
            "failed to parse decimal '{v}' from Binance.us payload: {err}"
        ))
    })
}

fn timestamp_from_ms(ms: u64) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp_millis(ms as i64).unwrap_or_else(chrono::Utc::now)
}

fn backoff_delay(attempt: u32) -> Duration {
    let capped_attempt = attempt.min(10);
    let millis = (500.0 * 1.5_f64.powi(capped_attempt as i32)).min(15_000.0);
    Duration::from_millis(millis as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalises_symbol() {
        assert_eq!(canonical_symbol("BTC-USD"), "btcusd");
        assert_eq!(canonical_symbol("eth_usd"), "ethusd");
    }

    #[test]
    fn build_subscriptions_contains_depth_and_ticker() {
        let params = build_subscription_params(&["BTC-USD".to_string()]);
        assert!(params.iter().any(|p| p == "btcusd@bookTicker"));
        assert!(params.iter().any(|p| p == "btcusd@depth@100ms"));
    }
}

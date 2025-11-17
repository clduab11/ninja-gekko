//! Binance.us API Connector (exchange spot markets)
//!
//! Implements full REST API and WebSocket streaming for Binance.US
//! with HMAC-SHA256 authentication following November 2025 standards.

use crate::{
    Balance, ExchangeConnector, ExchangeError, ExchangeId, ExchangeOrder, ExchangeResult, Fill,
    MarketTick, OrderSide, OrderStatus, OrderType, StreamMessage, TransferRequest, TransferStatus,
    TradingPair,
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use parking_lot::RwLock;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
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

type HmacSha256 = Hmac<Sha256>;

/// Configuration for Binance.US API connector
#[derive(Debug, Clone)]
pub struct BinanceUsConfig {
    pub api_key: String,
    pub api_secret: String,
    pub sandbox: bool,
}

/// Full-featured Binance.US connector with REST API and WebSocket streaming
pub struct BinanceUsConnector {
    inner: Arc<BinanceInner>,
}

struct BinanceInner {
    connected: AtomicBool,
    ws_url: Url,
    rest_url: Url,
    client: Client,
    credentials: RwLock<Option<BinanceUsCredentials>>,
}

#[derive(Clone)]
struct BinanceUsCredentials {
    api_key: String,
    api_secret: String,
}

impl BinanceUsConnector {
    /// Create new connector without credentials (streaming only)
    pub fn new() -> Self {
        Self::new_with_credentials(None)
    }

    /// Create connector with API credentials for trading
    pub fn with_config(config: BinanceUsConfig) -> Self {
        let credentials = BinanceUsCredentials {
            api_key: config.api_key,
            api_secret: config.api_secret,
        };
        Self::new_with_credentials(Some(credentials))
    }

    fn new_with_credentials(credentials: Option<BinanceUsCredentials>) -> Self {
        Self {
            inner: Arc::new(BinanceInner {
                connected: AtomicBool::new(false),
                ws_url: Url::parse(BINANCE_US_WS_URL).expect("valid Binance.us ws url"),
                rest_url: Url::parse(BINANCE_US_REST_URL).expect("valid Binance.us rest url"),
                client: Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .expect("valid HTTP client"),
                credentials: RwLock::new(credentials),
            }),
        }
    }

    /// Set or update API credentials after construction
    pub fn set_credentials(&self, api_key: String, api_secret: String) {
        *self.inner.credentials.write() = Some(BinanceUsCredentials {
            api_key,
            api_secret,
        });
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

    async fn get_trading_pairs(&self) -> ExchangeResult<Vec<TradingPair>> {
        debug!("fetching Binance.US trading pairs");

        let url = format!("{}/api/v3/exchangeInfo", self.inner.rest_url);
        let response = self.inner.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch exchange info: {}", e)))?;

        if !response.status().is_success() {
            return Err(map_binance_error(response.status().as_u16(), "Failed to fetch exchange info").await);
        }

        let exchange_info: BinanceExchangeInfo = response.json().await
            .map_err(|e| ExchangeError::Network(format!("Failed to parse exchange info: {}", e)))?;

        let pairs = exchange_info.symbols.into_iter()
            .filter(|s| s.status == "TRADING" && s.is_spot_trading_allowed.unwrap_or(false))
            .map(|s| TradingPair {
                base: s.base_asset,
                quote: s.quote_asset,
                symbol: s.symbol,
            })
            .collect();

        Ok(pairs)
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        debug!("fetching Binance.US account balances");

        let credentials = self.inner.credentials.read().clone()
            .ok_or_else(|| ExchangeError::Authentication("API credentials not configured".to_string()))?;

        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("timestamp={}", timestamp);
        let signature = sign_request(&credentials.api_secret, &query);

        let url = format!("{}/api/v3/account?{}&signature={}", self.inner.rest_url, query, signature);

        let response = self.inner.client
            .get(&url)
            .header("X-MBX-APIKEY", &credentials.api_key)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch balances: {}", e)))?;

        if !response.status().is_success() {
            return Err(map_binance_error(response.status().as_u16(), "Failed to fetch balances").await);
        }

        let account: BinanceAccount = response.json().await
            .map_err(|e| ExchangeError::Network(format!("Failed to parse account data: {}", e)))?;

        let balances = account.balances.into_iter()
            .filter_map(|b| {
                let free = Decimal::from_str(&b.free).ok()?;
                let locked = Decimal::from_str(&b.locked).ok()?;
                let total = free + locked;

                if total > Decimal::ZERO {
                    Some(Balance {
                        currency: b.asset,
                        available: free,
                        total,
                        hold: locked,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(balances)
    }

    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> ExchangeResult<ExchangeOrder> {
        debug!(?side, ?order_type, %quantity, "placing Binance.US order");

        let credentials = self.inner.credentials.read().clone()
            .ok_or_else(|| ExchangeError::Authentication("API credentials not configured".to_string()))?;

        let timestamp = chrono::Utc::now().timestamp_millis();
        let symbol_clean = symbol.replace(['-', '_'], "").to_uppercase();
        let side_str = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };
        let type_str = match order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit => "LIMIT",
            _ => return Err(ExchangeError::InvalidRequest(format!("Unsupported order type: {:?}", order_type))),
        };

        let mut query = format!("symbol={}&side={}&type={}&quantity={}&timestamp={}",
            symbol_clean, side_str, type_str, quantity, timestamp);

        if let Some(p) = price {
            if order_type == OrderType::Limit {
                query.push_str(&format!("&price={}&timeInForce=GTC", p));
            }
        }

        let signature = sign_request(&credentials.api_secret, &query);
        let url = format!("{}/api/v3/order?{}&signature={}", self.inner.rest_url, query, signature);

        let response = self.inner.client
            .post(&url)
            .header("X-MBX-APIKEY", &credentials.api_key)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to place order: {}", e)))?;

        if !response.status().is_success() {
            return Err(map_binance_error(response.status().as_u16(), "Failed to place order").await);
        }

        let binance_order: BinanceOrder = response.json().await
            .map_err(|e| ExchangeError::Network(format!("Failed to parse order response: {}", e)))?;

        convert_binance_order(binance_order, symbol)
    }

    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        debug!(%order_id, "canceling Binance.US order");

        let credentials = self.inner.credentials.read().clone()
            .ok_or_else(|| ExchangeError::Authentication("API credentials not configured".to_string()))?;

        // Parse order_id format: "symbol:order_id" (e.g., "BTCUSD:123456")
        let parts: Vec<&str> = order_id.split(':').collect();
        if parts.len() != 2 {
            return Err(ExchangeError::InvalidRequest("Order ID must be in format 'SYMBOL:ID'".to_string()));
        }
        let symbol = parts[0];
        let binance_order_id = parts[1];

        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("symbol={}&orderId={}&timestamp={}", symbol, binance_order_id, timestamp);
        let signature = sign_request(&credentials.api_secret, &query);
        let url = format!("{}/api/v3/order?{}&signature={}", self.inner.rest_url, query, signature);

        let response = self.inner.client
            .delete(&url)
            .header("X-MBX-APIKEY", &credentials.api_key)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to cancel order: {}", e)))?;

        if !response.status().is_success() {
            return Err(map_binance_error(response.status().as_u16(), "Failed to cancel order").await);
        }

        let binance_order: BinanceOrder = response.json().await
            .map_err(|e| ExchangeError::Network(format!("Failed to parse cancel response: {}", e)))?;

        convert_binance_order(binance_order, symbol)
    }

    async fn get_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        debug!(%order_id, "fetching Binance.US order");

        let credentials = self.inner.credentials.read().clone()
            .ok_or_else(|| ExchangeError::Authentication("API credentials not configured".to_string()))?;

        // Parse order_id format: "symbol:order_id"
        let parts: Vec<&str> = order_id.split(':').collect();
        if parts.len() != 2 {
            return Err(ExchangeError::InvalidRequest("Order ID must be in format 'SYMBOL:ID'".to_string()));
        }
        let symbol = parts[0];
        let binance_order_id = parts[1];

        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("symbol={}&orderId={}&timestamp={}", symbol, binance_order_id, timestamp);
        let signature = sign_request(&credentials.api_secret, &query);
        let url = format!("{}/api/v3/order?{}&signature={}", self.inner.rest_url, query, signature);

        let response = self.inner.client
            .get(&url)
            .header("X-MBX-APIKEY", &credentials.api_key)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch order: {}", e)))?;

        if !response.status().is_success() {
            return Err(map_binance_error(response.status().as_u16(), "Failed to fetch order").await);
        }

        let binance_order: BinanceOrder = response.json().await
            .map_err(|e| ExchangeError::Network(format!("Failed to parse order: {}", e)))?;

        convert_binance_order(binance_order, symbol)
    }

    async fn get_market_data(&self, symbol: &str) -> ExchangeResult<MarketTick> {
        debug!(%symbol, "fetching Binance.US market data");

        let symbol_clean = symbol.replace(['-', '_'], "").to_uppercase();
        let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.inner.rest_url, symbol_clean);

        let response = self.inner.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch market data: {}", e)))?;

        if !response.status().is_success() {
            return Err(map_binance_error(response.status().as_u16(), "Failed to fetch market data").await);
        }

        let ticker: BinanceTicker = response.json().await
            .map_err(|e| ExchangeError::Network(format!("Failed to parse ticker: {}", e)))?;

        Ok(MarketTick {
            symbol: symbol.to_string(),
            bid: Decimal::from_str(&ticker.bid_price).unwrap_or(Decimal::ZERO),
            ask: Decimal::from_str(&ticker.ask_price).unwrap_or(Decimal::ZERO),
            last: Decimal::from_str(&ticker.last_price).unwrap_or(Decimal::ZERO),
            volume_24h: Decimal::from_str(&ticker.volume).unwrap_or(Decimal::ZERO),
            timestamp: chrono::Utc::now(),
        })
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

/// Generate HMAC-SHA256 signature for Binance.US API requests
fn sign_request(secret: &str, query: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(query.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Map Binance.US HTTP status codes to ExchangeError
async fn map_binance_error(status_code: u16, context: &str) -> ExchangeError {
    match status_code {
        401 => ExchangeError::Authentication(format!("{}: Invalid API key", context)),
        403 => ExchangeError::Authentication(format!("{}: API key permissions insufficient", context)),
        429 => ExchangeError::RateLimit(format!("{}: Rate limit exceeded", context)),
        400 | 404 => ExchangeError::InvalidRequest(format!("{}: Bad request or not found", context)),
        500 | 502 | 503 => ExchangeError::Network(format!("{}: Server error", context)),
        _ => ExchangeError::Api {
            code: status_code.to_string(),
            message: context.to_string(),
        },
    }
}

/// Convert Binance.US order response to ExchangeOrder
fn convert_binance_order(order: BinanceOrder, symbol: &str) -> ExchangeResult<ExchangeOrder> {
    let side = match order.side.as_str() {
        "BUY" => OrderSide::Buy,
        "SELL" => OrderSide::Sell,
        _ => return Err(ExchangeError::Network(format!("Unknown order side: {}", order.side))),
    };

    let order_type = match order.order_type.as_str() {
        "MARKET" => OrderType::Market,
        "LIMIT" => OrderType::Limit,
        _ => OrderType::Limit, // Default to limit
    };

    let status = match order.status.as_str() {
        "NEW" => OrderStatus::Pending,
        "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
        "FILLED" => OrderStatus::Filled,
        "CANCELED" => OrderStatus::Cancelled,
        "REJECTED" => OrderStatus::Rejected,
        _ => OrderStatus::Pending,
    };

    let quantity = Decimal::from_str(&order.orig_qty)
        .map_err(|e| ExchangeError::Network(format!("Invalid quantity: {}", e)))?;

    let price = if !order.price.is_empty() && order.price != "0.00" {
        Some(Decimal::from_str(&order.price)
            .map_err(|e| ExchangeError::Network(format!("Invalid price: {}", e)))?)
    } else {
        None
    };

    let fills = order.fills.unwrap_or_default().into_iter()
        .filter_map(|f| {
            let fill_price = Decimal::from_str(&f.price).ok()?;
            let fill_qty = Decimal::from_str(&f.qty).ok()?;
            let commission = Decimal::from_str(&f.commission).ok()?;

            Some(Fill {
                id: format!("{}:fill", order.order_id),
                order_id: format!("{}:{}", symbol, order.order_id),
                price: fill_price,
                quantity: fill_qty,
                fee: commission,
                timestamp: chrono::Utc::now(),
            })
        })
        .collect();

    Ok(ExchangeOrder {
        id: format!("{}:{}", symbol, order.order_id),
        exchange_id: ExchangeId::BinanceUs,
        symbol: symbol.to_string(),
        side,
        order_type,
        quantity,
        price,
        status,
        timestamp: chrono::Utc::now(),
        fills,
    })
}

// Binance.US API response structures

#[derive(Debug, Deserialize)]
struct BinanceExchangeInfo {
    symbols: Vec<BinanceSymbol>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceSymbol {
    symbol: String,
    status: String,
    base_asset: String,
    quote_asset: String,
    is_spot_trading_allowed: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct BinanceAccount {
    balances: Vec<BinanceBalance>,
}

#[derive(Debug, Deserialize)]
struct BinanceBalance {
    asset: String,
    free: String,
    locked: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceOrder {
    symbol: String,
    order_id: u64,
    #[serde(rename = "type")]
    order_type: String,
    side: String,
    price: String,
    orig_qty: String,
    executed_qty: String,
    status: String,
    time_in_force: Option<String>,
    fills: Option<Vec<BinanceFill>>,
}

#[derive(Debug, Deserialize)]
struct BinanceFill {
    price: String,
    qty: String,
    commission: String,
    #[serde(rename = "commissionAsset")]
    commission_asset: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceTicker {
    symbol: String,
    price_change: String,
    last_price: String,
    bid_price: String,
    ask_price: String,
    volume: String,
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

//! Kraken API Connector
//!
//! Implements the ExchangeConnector trait for Kraken Spot API.
//! Validated against official documentation:
//! - Auth: HMAC-SHA512(path + SHA256(nonce + post_data), b64_decode(secret))
//! - Nonce: Always increasing u64 (Unix timestamp ms)
//! - Endpoints: /0/private/Balance, /0/private/AddOrder, /0/private/CancelOrder

use crate::credentials::ExchangeCredentials;
use crate::{
    utils::decimal_to_string, Balance, Candle, ExchangeConnector, ExchangeError, ExchangeId,
    ExchangeOrder, ExchangeResult, Fill, MarketTick, OrderSide, OrderStatus, OrderType,
    RateLimiter, StreamMessage, Timeframe, TradingPair, TransferRequest, TransferStatus,
};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::{Client, Method, RequestBuilder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use url::Url;

/// Kraken API URLs
const KRAKEN_API_URL: &str = "https://api.kraken.com";
const KRAKEN_WS_URL: &str = "wss://ws.kraken.com";

pub struct KrakenConnector {
    credentials: ExchangeCredentials,
    client: Client,
    rate_limiter: RateLimiter,
    base_url: String,
    ws_url: String,
}

impl KrakenConnector {
    pub fn new(credentials: ExchangeCredentials) -> Self {
        let client = Client::new();
        // Kraken has tiered limits, conservative start at 1 req/s or so,
        // but let's go with 5 for now as limits are often counter based.
        let rate_limiter = RateLimiter::new(5);

        Self {
            credentials,
            client,
            rate_limiter,
            base_url: KRAKEN_API_URL.to_string(),
            ws_url: KRAKEN_WS_URL.to_string(),
        }
    }

    /// Generate Kraken signature
    /// API-Sign = Message signature using HMAC-SHA512 of (URI path + SHA256(nonce + POST data)) and base64 decoded secret API key
    fn generate_signature(
        &self,
        path: &str,
        nonce: &str,
        post_data: &str,
    ) -> ExchangeResult<String> {
        let secret = self.credentials.api_secret();

        // 1. Decode API Secret
        let secret_bytes = STANDARD.decode(secret).map_err(|e| {
            ExchangeError::Configuration(format!("Invalid Kraken API Secret (Base64): {}", e))
        })?;

        // 2. SHA256(nonce + post_data)
        let mut sha256 = Sha256::new();
        sha256.update(nonce.as_bytes());
        sha256.update(post_data.as_bytes());
        let sha256_hash = sha256.finalize();

        // 3. HMAC-SHA512(path + sha256_hash, secret_bytes)
        let mut hmac = Hmac::<Sha512>::new_from_slice(&secret_bytes)
            .map_err(|_| ExchangeError::Configuration("Invalid HMAC key length".to_string()))?;

        hmac.update(path.as_bytes());
        hmac.update(&sha256_hash);

        let signature = hmac.finalize();
        let signature_b64 = STANDARD.encode(signature.into_bytes());

        Ok(signature_b64)
    }

    async fn send_public_request<T>(&self, path: &str, params: &[(&str, &str)]) -> ExchangeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.rate_limiter.acquire().await?;

        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        self.handle_response(response).await
    }

    async fn send_private_request<T>(
        &self,
        path: &str,
        params: &mut HashMap<&str, String>,
    ) -> ExchangeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.rate_limiter.acquire().await?;

        let nonce = chrono::Utc::now().timestamp_millis().to_string();
        params.insert("nonce", nonce.clone());

        // Serialize params to x-www-form-urlencoded format for signature and body
        let post_data = serde_urlencoded::to_string(&params).map_err(|e| {
            ExchangeError::InvalidRequest(format!("Failed to serialize params: {}", e))
        })?;

        let signature = self.generate_signature(path, &nonce, &post_data)?;

        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .post(&url)
            .header("API-Key", self.credentials.api_key())
            .header("API-Sign", signature)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(post_data) // reqwest usually handles form urlencoding but we need strict control for signature
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        self.handle_response(response).await
    }

    async fn handle_response<T>(&self, response: reqwest::Response) -> ExchangeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(ExchangeError::Api {
                code: status.as_u16().to_string(),
                message: text,
            });
        }

        // Kraken returns { "error": [], "result": {} }
        // Errors are in the "error" array.
        let json: Value = serde_json::from_str(&text).map_err(|e| {
            ExchangeError::InvalidRequest(format!("JSON parse error: {} Body: {}", e, text))
        })?;

        if let Some(errors) = json.get("error").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                let error_msgs: Vec<String> = errors
                    .iter()
                    .map(|v| v.as_str().unwrap_or("unknown").to_string())
                    .collect();
                return Err(ExchangeError::Api {
                    code: "200".to_string(), // HTTP was 200, but logic error
                    message: error_msgs.join(", "),
                });
            }
        }

        let result = json.get("result").ok_or_else(|| ExchangeError::Api {
            code: "MissingResult".to_string(),
            message: "Response missing 'result' field".to_string(),
        })?;

        serde_json::from_value(result.clone())
            .map_err(|e| ExchangeError::InvalidRequest(format!("Failed to map result: {}", e)))
    }
}

#[async_trait]
impl ExchangeConnector for KrakenConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Kraken
    }

    async fn connect(&mut self) -> ExchangeResult<()> {
        info!("Connecting to Kraken...");
        // Validate credentials by checking balance (private endpoint)
        // or just server time (public) if we want lazy connection.
        // Let's do a public check first to verify connectivity.
        let _: Value = self.send_public_request("/0/public/Time", &[]).await?;

        // If keys are present, try a private call as smoke test
        if !self.credentials.api_key().is_empty() {
            let _: HashMap<String, Value> = self
                .send_private_request("/0/private/Balance", &mut HashMap::new())
                .await?;
        }

        info!("Successfully connected to Kraken");
        Ok(())
    }

    async fn disconnect(&mut self) -> ExchangeResult<()> {
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        // Stateless REST, theoretically always connected if internet is up
        true
    }

    async fn get_trading_pairs(&self) -> ExchangeResult<Vec<TradingPair>> {
        let result: HashMap<String, Value> = self
            .send_public_request("/0/public/AssetPairs", &[])
            .await?;

        let mut pairs = Vec::new();
        for (name, info) in result {
            if let Some(base) = info.get("base").and_then(|v| v.as_str()) {
                if let Some(quote) = info.get("quote").and_then(|v| v.as_str()) {
                    // Filter out dark pools or other weird stuff if necessary
                    // Kraken uses XBT for BTC, let's keep it as is or normalize later
                    pairs.push(TradingPair {
                        base: base.to_string(),
                        quote: quote.to_string(),
                        symbol: info
                            .get("altname")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&name)
                            .to_string(),
                    });
                }
            }
        }
        Ok(pairs)
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        let balances_map: HashMap<String, String> = self
            .send_private_request("/0/private/Balance", &mut HashMap::new())
            .await?;

        let mut balances = Vec::new();
        for (asset, amount_str) in balances_map {
            let total = Decimal::from_str(&amount_str).unwrap_or_default();
            // Kraken doesn't easily give "available" vs "hold" in the simple Balance endpoint
            // It just gives total balance. TradeBalance endpoint gives holistic view but not per-asset hold.
            // For now, assume total = available for simplicity unless we query open orders to deduct.
            balances.push(Balance {
                currency: asset,
                total,
                available: total, // Approximate
                hold: Decimal::ZERO,
            });
        }
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
        let mut params = HashMap::new();
        params.insert("pair", symbol.to_string());
        params.insert(
            "type",
            match side {
                OrderSide::Buy => "buy".to_string(),
                OrderSide::Sell => "sell".to_string(),
            },
        );
        params.insert(
            "ordertype",
            match order_type {
                OrderType::Market => "market".to_string(),
                OrderType::Limit => "limit".to_string(),
                OrderType::Stop => "stop-loss".to_string(), // Simplified mapping
                OrderType::StopLimit => "stop-loss-limit".to_string(),
            },
        );
        params.insert("volume", quantity.to_string());

        if let Some(p) = price {
            params.insert("price", p.to_string());
        }

        // Add 'userref' if we want to track by custom ID?
        // For now rely on Kraken returning TXID.

        let result: PlaceOrderResponse = self
            .send_private_request("/0/private/AddOrder", &mut params)
            .await?;

        // Kraken returns txid list (one per order if multiple)
        let id = result.txid.first().cloned().unwrap_or_default();

        Ok(ExchangeOrder {
            id,
            exchange_id: ExchangeId::Kraken,
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity,
            price,
            status: OrderStatus::Open, // Assumed open/pending
            timestamp: chrono::Utc::now(),
            fills: vec![], // Details not returned in AddOrder response directly
        })
    }

    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        let mut params = HashMap::new();
        params.insert("txid", order_id.to_string());

        let result: CancelOrderResponse = self
            .send_private_request("/0/private/CancelOrder", &mut params)
            .await?;

        Ok(ExchangeOrder {
            id: order_id.to_string(),
            exchange_id: ExchangeId::Kraken,
            symbol: "".to_string(),        // Unknown context without lookup
            side: OrderSide::Buy,          // Dummy
            order_type: OrderType::Market, // Dummy
            quantity: Decimal::ZERO,
            price: None,
            status: OrderStatus::Cancelled,
            timestamp: chrono::Utc::now(),
            fills: vec![],
        })
    }

    async fn get_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        let mut params = HashMap::new();
        params.insert("txid", order_id.to_string());
        params.insert("trades", "true".to_string());

        let result: HashMap<String, Value> = self
            .send_private_request("/0/private/QueryOrders", &mut params)
            .await?;

        // Parse complex result to ExchangeOrder...
        // This is complex because result is map of txid -> order info
        Err(ExchangeError::InvalidRequest(
            "Not fully implemented yet".to_string(),
        ))
    }

    async fn get_market_data(&self, symbol: &str) -> ExchangeResult<MarketTick> {
        let result: HashMap<String, Value> = self
            .send_public_request("/0/public/Ticker", &[("pair", symbol)])
            .await?;

        // Result keys are pair names, which might differ slightly from requested symbol if alias used
        // We just take the first value
        let (_key, val) = result
            .iter()
            .next()
            .ok_or_else(|| ExchangeError::OrderNotFound("No ticker data".to_string()))?;

        // Parsing ["price", "whole lot volume", "volume"] arrays...
        // a = ask array [price, whole lot vol, vol]
        // b = bid array [price, whole lot vol, vol]
        // c = last trade closed [price, lot vol]
        // v = volume [today, 24h]

        let ask = val
            .get("a")
            .and_then(|a| a.get(0))
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let bid = val
            .get("b")
            .and_then(|b| b.get(0))
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let last = val
            .get("c")
            .and_then(|c| c.get(0))
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let vol = val
            .get("v")
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_str())
            .unwrap_or("0"); // 24h vol

        Ok(MarketTick {
            symbol: symbol.to_string(),
            bid: Decimal::from_str(ask).unwrap_or_default(),
            ask: Decimal::from_str(bid).unwrap_or_default(),
            last: Decimal::from_str(last).unwrap_or_default(),
            volume_24h: Decimal::from_str(vol).unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn start_market_stream(
        &self,
        _symbols: Vec<String>,
    ) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        Err(ExchangeError::InvalidRequest(
            "Kraken WS not implemented yet".to_string(),
        ))
    }

    async fn start_order_stream(&self) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        Err(ExchangeError::InvalidRequest(
            "Kraken WS not implemented yet".to_string(),
        ))
    }

    async fn transfer_funds(&self, _request: TransferRequest) -> ExchangeResult<String> {
        Err(ExchangeError::InvalidRequest(
            "Transfer not supported".to_string(),
        ))
    }

    async fn get_transfer_status(&self, _transfer_id: &str) -> ExchangeResult<TransferStatus> {
        Err(ExchangeError::InvalidRequest(
            "Transfer status not supported".to_string(),
        ))
    }

    async fn get_candles(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        _start: Option<chrono::DateTime<chrono::Utc>>,
        _end: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ExchangeResult<Vec<Candle>> {
        // Interval in minutes
        let interval = match timeframe {
            Timeframe::OneMinute => "1",
            Timeframe::FiveMinutes => "5",
            Timeframe::FifteenMinutes => "15",
            Timeframe::OneHour => "60",
            Timeframe::FourHours => "240",
            Timeframe::OneDay => "1440",
        };

        let result: HashMap<String, Value> = self
            .send_public_request(
                "/0/public/OHLC",
                &[("pair", symbol), ("interval", interval)],
            )
            .await?;

        // Similar to ticker, key is pair name
        // "last" is also in the root object usually, but let's find the array

        let mut candles = Vec::new();
        for (key, val) in result {
            if key == "last" {
                continue;
            }
            if let Some(arr) = val.as_array() {
                for c in arr {
                    // [int time, string open, string high, string low, string close, string vwap, string vol, int count]
                    if let (Some(time), Some(open), Some(high), Some(low), Some(close), Some(vol)) = (
                        c.get(0).and_then(|v| v.as_i64()),
                        c.get(1).and_then(|v| v.as_str()),
                        c.get(2).and_then(|v| v.as_str()),
                        c.get(3).and_then(|v| v.as_str()),
                        c.get(4).and_then(|v| v.as_str()),
                        c.get(6).and_then(|v| v.as_str()),
                    ) {
                        candles.push(Candle {
                            start_time: chrono::DateTime::from_timestamp(time, 0)
                                .unwrap_or_else(chrono::Utc::now),
                            open: Decimal::from_str(open).unwrap_or_default(),
                            high: Decimal::from_str(high).unwrap_or_default(),
                            low: Decimal::from_str(low).unwrap_or_default(),
                            close: Decimal::from_str(close).unwrap_or_default(),
                            volume: Decimal::from_str(vol).unwrap_or_default(),
                        });
                    }
                }
            }
        }

        Ok(candles)
    }
}

#[derive(Deserialize)]
struct PlaceOrderResponse {
    descr: OrderDescription,
    txid: Vec<String>,
}

#[derive(Deserialize)]
struct OrderDescription {
    order: String,
}

#[derive(Deserialize)]
struct CancelOrderResponse {
    count: i32,
    pending: Option<bool>,
}

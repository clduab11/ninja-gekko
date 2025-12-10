//! Coinbase Pro/Advanced Trade API Connector
//!
//! Implements the ExchangeConnector trait for Coinbase Pro and Advanced Trade APIs.
//! Supports:
use crate::{
    Balance, ExchangeConnector, ExchangeError, ExchangeId, ExchangeOrder, ExchangeResult, Fill,
    MarketTick, OrderSide, OrderStatus, OrderType, RateLimiter, StreamMessage, TradingPair,
    TransferRequest, TransferStatus,
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, Method, RequestBuilder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use url::Url;
use rand::Rng;
use p256::ecdsa::{SigningKey, signature::{Signer, Signature}};
use p256::pkcs8::DecodePrivateKey;
use sec1::DecodeEcPrivateKey;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Coinbase Pro API URLs
const COINBASE_PRO_API_URL: &str = "https://api.pro.coinbase.com";
const COINBASE_PRO_SANDBOX_URL: &str = "https://api-public.sandbox.pro.coinbase.com";
const COINBASE_PRO_WS_URL: &str = "wss://ws-feed.pro.coinbase.com";
const COINBASE_PRO_WS_SANDBOX_URL: &str = "wss://ws-feed-public.sandbox.pro.coinbase.com";

/// Coinbase Advanced Trade API URLs
const COINBASE_ADVANCED_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage";
const COINBASE_ADVANCED_WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

/// JWT authentication constants
const JWT_CLOCK_SKEW_BUFFER_SECONDS: i64 = 5;

#[derive(Debug, Clone)]
pub struct CoinbaseConfig {
    pub api_key_name: String,
    pub private_key: String,
    pub sandbox: bool,
    pub use_advanced_trade: bool, // Use Advanced Trade API vs Pro API
}

/// Coinbase Pro/Advanced Trade connector
pub struct CoinbaseConnector {
    config: CoinbaseConfig,
    client: Client,
    rate_limiter: RateLimiter,
    base_url: String,
    ws_url: String,
    connected: bool,
}

impl CoinbaseConnector {
    pub fn new(config: CoinbaseConfig) -> Self {
        let base_url = if config.use_advanced_trade {
            COINBASE_ADVANCED_API_URL.to_string()
        } else if config.sandbox {
            COINBASE_PRO_SANDBOX_URL.to_string()
        } else {
            COINBASE_PRO_API_URL.to_string()
        };

        let ws_url = if config.use_advanced_trade {
            COINBASE_ADVANCED_WS_URL.to_string()
        } else if config.sandbox {
            COINBASE_PRO_WS_SANDBOX_URL.to_string()
        } else {
            COINBASE_PRO_WS_URL.to_string()
        };

        let client = Client::new();
        let rate_limiter = RateLimiter::new(10); // 10 requests per second limit

        Self {
            config,
            client,
            rate_limiter,
            base_url,
            ws_url,
            connected: false,
        }
    }

    /// Generate JWT for Coinbase CDP API
    fn generate_jwt(&self, method: &str, path: &str) -> ExchangeResult<String> {
        let key_name = &self.config.api_key_name;
        // Handle potential double-escaped newlines from .env and surrounding quotes
        let private_key_pem = self.config.private_key.trim();
        let private_key_pem = if private_key_pem.starts_with('"') && private_key_pem.ends_with('"') {
            &private_key_pem[1..private_key_pem.len()-1]
        } else {
            private_key_pem
        };
        
        // Unescape newlines: replace literal "\n" with actual newline character
        let private_key_pem = private_key_pem.replace("\\n", "\n");
        let private_key_pem = private_key_pem.trim();
        
        if private_key_pem.len() > 20 {
            debug!("PEM start: '{}'...", &private_key_pem[..20]);
        } else {
            debug!("PEM too short: '{}'", private_key_pem);
        }

        // 1. Create Header
        let nonce = format!("{:016x}", rand::thread_rng().gen::<u64>());
        let header = json!({
            "alg": "ES256",
            "kid": key_name,
            "nonce": nonce,
            "typ": "JWT"
        });

        // 2. Create Claims
        let uri = format!("{} {}{}", method, self.base_url.trim_start_matches("https://"), path);
        
        let base_url_parsed = Url::parse(&self.base_url)
            .map_err(|e| ExchangeError::Configuration(format!("Invalid base URL: {}", e)))?;
        
        let host = base_url_parsed.host_str().unwrap_or("api.coinbase.com");
        let full_path = format!("{}{}", base_url_parsed.path().trim_end_matches('/'), path);
        
        let jwt_uri = format!("{} {}{}", method, host, full_path);

        let now = chrono::Utc::now().timestamp();
        
        let claims = json!({
            "iss": "cdp",
            "nbf": now - JWT_CLOCK_SKEW_BUFFER_SECONDS,
            "exp": now + 120, // 2 minutes
            "sub": key_name,
            "uri": jwt_uri,
        });

        // 3. Encode Header and Claims
        let header_json = serde_json::to_string(&header)
            .map_err(|e| ExchangeError::Configuration(format!("Failed to serialize header: {}", e)))?;
        let claims_json = serde_json::to_string(&claims)
            .map_err(|e| ExchangeError::Configuration(format!("Failed to serialize claims: {}", e)))?;

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json);
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json);

        let message = format!("{}.{}", header_b64, claims_b64);

        // 4. Sign
        // Try parsing as SEC1 (EC PRIVATE KEY) first, then PKCS#8
        debug!("Parsing private key PEM (len: {})", private_key_pem.len());
        let signing_key = SigningKey::from_sec1_pem(private_key_pem)
            .or_else(|e| {
                debug!("Failed to parse as SEC1: {}", e);
                SigningKey::from_pkcs8_pem(private_key_pem)
            })
            .map_err(|e| ExchangeError::Authentication(format!("Invalid private key: {}", e)))?;

        let signature: p256::ecdsa::Signature = signing_key.sign(message.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.as_bytes());

        Ok(format!("{}.{}", message, signature_b64))
    }

    /// Create authenticated request for Coinbase CDP API
    fn create_authenticated_request(
        &self,
        method: Method,
        path: &str,
        _body: &str, // Body is not used in JWT signature for CDP, but might be needed for legacy if we kept it
    ) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        
        // Generate JWT
        match self.generate_jwt(method.as_str(), path) {
            Ok(token) => {
                self.client
                    .request(method, &url)
                    .header("Authorization", format!("Bearer {}", token))
                    .header("Content-Type", "application/json")
            },
            Err(e) => {
                error!("Failed to generate JWT: {}", e);
                // Return a builder that will likely fail, or we should have returned Result<RequestBuilder>
                // Since the signature is RequestBuilder, we'll log error and return a request that will fail auth
                self.client.request(method, &url)
            }
        }
    }

    /// Handle API response and convert errors
    async fn handle_response<T>(&self, response: reqwest::Response) -> ExchangeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        debug!("Coinbase API response: {} - {}", status, response_text);

        if status.is_success() {
            serde_json::from_str(&response_text)
                .map_err(|e| ExchangeError::InvalidRequest(format!("JSON parse error: {}", e)))
        } else {
            // Parse error response
            if let Ok(error_response) =
                serde_json::from_str::<CoinbaseErrorResponse>(&response_text)
            {
                Err(ExchangeError::Api {
                    code: status.as_u16().to_string(),
                    message: error_response.message,
                })
            } else {
                Err(ExchangeError::Api {
                    code: status.as_u16().to_string(),
                    message: response_text,
                })
            }
        }
    }
}

#[async_trait]
impl ExchangeConnector for CoinbaseConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Coinbase
    }

    async fn connect(&mut self) -> ExchangeResult<()> {
        info!(
            "Connecting to Coinbase {}...",
            if self.config.use_advanced_trade {
                "Advanced Trade"
            } else {
                "Pro"
            }
        );

        // Test connection by fetching account info
        self.rate_limiter.acquire().await?;

        let request = self.create_authenticated_request(Method::GET, "/accounts", "");
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        // Just check if we get a successful response
        if response.status().is_success() {
            self.connected = true;
            info!("Successfully connected to Coinbase");
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to connect to Coinbase: {}", error_text);
            Err(ExchangeError::Authentication(error_text))
        }
    }

    async fn disconnect(&mut self) -> ExchangeResult<()> {
        self.connected = false;
        info!("Disconnected from Coinbase");
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn get_trading_pairs(&self) -> ExchangeResult<Vec<TradingPair>> {
        self.rate_limiter.acquire().await?;

        let request = self.client.get(&format!("{}/products", self.base_url));
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let products: Vec<CoinbaseProduct> = self.handle_response(response).await?;

        let trading_pairs = products
            .into_iter()
            .filter(|p| p.status == "online" && !p.trading_disabled)
            .map(|p| TradingPair {
                base: p.base_currency,
                quote: p.quote_currency,
                symbol: p.id,
            })
            .collect();

        Ok(trading_pairs)
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        self.rate_limiter.acquire().await?;

        let request = self.create_authenticated_request(Method::GET, "/accounts", "");
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let accounts: Vec<CoinbaseAccount> = self.handle_response(response).await?;

        let balances = accounts
            .into_iter()
            .map(|acc| Balance {
                currency: acc.currency,
                available: acc.available.parse().unwrap_or_default(),
                total: acc.balance.parse().unwrap_or_default(),
                hold: acc.hold.parse().unwrap_or_default(),
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
        self.rate_limiter.acquire().await?;

        let coinbase_side = match side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };

        let coinbase_type = match order_type {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::Stop => "stop",
            OrderType::StopLimit => "stop_limit",
        };

        let mut order_request = CoinbaseOrderRequest {
            product_id: symbol.to_string(),
            side: coinbase_side.to_string(),
            order_type: coinbase_type.to_string(),
            size: Some(quantity.to_string()),
            price: price.map(|p| p.to_string()),
            ..Default::default()
        };

        // For market orders, use funds instead of size for buys
        if order_type == OrderType::Market && side == OrderSide::Buy {
            if let Some(p) = price {
                order_request.funds = Some((quantity * p).to_string());
                order_request.size = None;
            }
        }

        let body = serde_json::to_string(&order_request)
            .map_err(|e| ExchangeError::InvalidRequest(e.to_string()))?;

        let request = self
            .create_authenticated_request(Method::POST, "/orders", &body)
            .body(body);

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let coinbase_order: CoinbaseOrder = self.handle_response(response).await?;

        Ok(convert_coinbase_order(coinbase_order))
    }

    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        self.rate_limiter.acquire().await?;

        let path = format!("/orders/{}", order_id);
        let request = self.create_authenticated_request(Method::DELETE, &path, "");

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let coinbase_order: CoinbaseOrder = self.handle_response(response).await?;

        Ok(convert_coinbase_order(coinbase_order))
    }

    async fn get_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        self.rate_limiter.acquire().await?;

        let path = format!("/orders/{}", order_id);
        let request = self.create_authenticated_request(Method::GET, &path, "");

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let coinbase_order: CoinbaseOrder = self.handle_response(response).await?;

        Ok(convert_coinbase_order(coinbase_order))
    }

    async fn get_market_data(&self, symbol: &str) -> ExchangeResult<MarketTick> {
        self.rate_limiter.acquire().await?;

        let url = format!("{}/products/{}/ticker", self.base_url, symbol);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let ticker: CoinbaseTicker = self.handle_response(response).await?;

        Ok(MarketTick {
            symbol: symbol.to_string(),
            bid: ticker.bid.parse().unwrap_or_default(),
            ask: ticker.ask.parse().unwrap_or_default(),
            last: ticker.price.parse().unwrap_or_default(),
            volume_24h: ticker.volume.parse().unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn start_market_stream(
        &self,
        symbols: Vec<String>,
    ) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        if symbols.is_empty() {
            return Err(ExchangeError::InvalidRequest(
                "at least one product id must be supplied for Coinbase streams".into(),
            ));
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let ws_url = self.ws_url.clone();
        let products = symbols.clone();

        tokio::spawn(async move {
            if let Err(err) = run_coinbase_market_stream(ws_url, products, tx.clone()).await {
                error!(%err, "coinbase market stream terminated");
            }
        });

        Ok(rx)
    }

    async fn start_order_stream(&self) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        // WebSocket implementation would go here
        // For now, return a placeholder channel
        let (_tx, rx) = mpsc::unbounded_channel();
        warn!("Coinbase WebSocket order stream not yet implemented");
        Ok(rx)
    }

    async fn transfer_funds(&self, _request: TransferRequest) -> ExchangeResult<String> {
        // Coinbase doesn't support direct transfers to other exchanges via API
        Err(ExchangeError::InvalidRequest(
            "Direct fund transfers not supported by Coinbase API".to_string(),
        ))
    }

    async fn get_transfer_status(&self, _transfer_id: &str) -> ExchangeResult<TransferStatus> {
        Err(ExchangeError::InvalidRequest(
            "Transfer status not supported by Coinbase API".to_string(),
        ))
    }

    async fn get_candles(
        &self,
        symbol: &str,
        timeframe: crate::Timeframe,
        start: Option<chrono::DateTime<chrono::Utc>>,
        end: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ExchangeResult<Vec<crate::Candle>> {
        self.rate_limiter.acquire().await?;

        let granularity = match timeframe {
            crate::Timeframe::OneMinute => "ONE_MINUTE",
            crate::Timeframe::FiveMinutes => "FIVE_MINUTE",
            crate::Timeframe::FifteenMinutes => "FIFTEEN_MINUTE",
            crate::Timeframe::OneHour => "ONE_HOUR",
            crate::Timeframe::FourHours => "SIX_HOUR", // Coinbase doesn't have 4h, using 6h or closest
            crate::Timeframe::OneDay => "ONE_DAY",
        };

        let mut url = format!("{}/products/{}/candles?granularity={}", self.base_url, symbol, granularity);
        
        if let Some(s) = start {
            url.push_str(&format!("&start={}", s.timestamp()));
        }
        if let Some(e) = end {
            url.push_str(&format!("&end={}", e.timestamp()));
        }

        let request = self.create_authenticated_request(Method::GET, &format!("/products/{}/candles", symbol), "");
        // Query params need to be added to the URL carefully if using create_authenticated_request which takes a path.
        // Actually, create_authenticated_request builds the URL from path. Let's rebuild to use properties properly.
        
        let mut path = format!("/products/{}/candles?granularity={}", symbol, granularity);
        if let Some(s) = start {
            path.push_str(&format!("&start={}", s.timestamp()));
        }
        if let Some(e) = end {
            path.push_str(&format!("&end={}", e.timestamp()));
        }

        let request = self.create_authenticated_request(Method::GET, &path, "");

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let candles_response: CoinbaseCandlesResponse = self.handle_response(response).await?;
        
        let mut candles = Vec::new();
        for c in candles_response.candles {
            // Coinbase Advanced Trade returns candles as object { start, low, high, open, close, volume }
            // But verify actual response structure. 
            // Docs say: { candles: [{ start, low, high, open, close, volume }, ...] }
            
            // Check if parsing needed or if it's already structured
            candles.push(crate::Candle {
                start_time: chrono::DateTime::from_timestamp(c.start.parse::<i64>().unwrap_or(0), 0)
                    .unwrap_or_else(chrono::Utc::now),
                open: c.open.parse().unwrap_or_default(),
                high: c.high.parse().unwrap_or_default(),
                low: c.low.parse().unwrap_or_default(),
                close: c.close.parse().unwrap_or_default(),
                volume: c.volume.parse().unwrap_or_default(),
            });
        }
        
        // Sort by time ascending
        candles.sort_by_key(|c| c.start_time);

        Ok(candles)
    }
}

#[derive(Debug, Deserialize)]
struct CoinbaseCandlesResponse {
    candles: Vec<CoinbaseCandle>,
}

#[derive(Debug, Deserialize)]
struct CoinbaseCandle {
    start: String,
    low: String,
    high: String,
    open: String,
    close: String,
    volume: String,
}

async fn run_coinbase_market_stream(
    ws_url: String,
    products: Vec<String>,
    sender: mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let url = Url::parse(&ws_url).map_err(|err| ExchangeError::Network(err.to_string()))?;
    let mut attempt: u32 = 0;

    loop {
        attempt = attempt.saturating_add(1);
        debug!(attempt, url = %url, "connecting to Coinbase market websocket");

        match connect_async(url.clone()).await {
            Ok((mut stream, _)) => {
                info!("coinbase websocket connected");
                attempt = 0;
                let subscribe = build_coinbase_subscription(&products);
                if let Err(err) = stream.send(Message::Text(subscribe)).await {
                    warn!(%err, "failed to send Coinbase subscription");
                    continue;
                }

                while let Some(message) = stream.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Err(err) = handle_coinbase_message(&text, &sender) {
                                warn!(%err, "failed to handle Coinbase message");
                            }
                        }
                        Ok(Message::Binary(bin)) => {
                            if let Ok(text) = String::from_utf8(bin) {
                                if let Err(err) = handle_coinbase_message(&text, &sender) {
                                    warn!(%err, "failed to handle Coinbase message");
                                }
                            }
                        }
                        Ok(Message::Ping(payload)) => {
                            if let Err(err) = stream.send(Message::Pong(payload)).await {
                                warn!(%err, "failed to pong Coinbase");
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("coinbase websocket closed by peer");
                            break;
                        }
                        Err(err) => {
                            warn!(%err, "coinbase websocket error");
                            break;
                        }
                        _ => {}
                    }

                    if sender.is_closed() {
                        debug!("coinbase subscriber dropped channel; stopping stream");
                        return Ok(());
                    }
                }
            }
            Err(err) => {
                warn!(%err, "coinbase websocket connection failed");
            }
        }

        if sender.is_closed() {
            debug!("coinbase subscriber dropped channel; terminating reconnect loop");
            return Ok(());
        }

        let delay = websocket_backoff(attempt);
        warn!(?delay, attempt, "reconnecting to Coinbase websocket");
        sleep(delay).await;
    }
}

fn build_coinbase_subscription(products: &[String]) -> String {
    json!({
        "type": "subscribe",
        "product_ids": products,
        "channels": [
            { "name": "level2", "product_ids": products },
            { "name": "ticker", "product_ids": products },
            "matches"
        ]
    })
    .to_string()
}

fn handle_coinbase_message(
    payload: &str,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let value: serde_json::Value = serde_json::from_str(payload)
        .map_err(|err| ExchangeError::Network(format!("invalid coinbase payload: {err}")))?;

    let Some(message_type) = value.get("type").and_then(|v| v.as_str()) else {
        return Ok(());
    };

    match message_type {
        "ticker" => emit_coinbase_ticker(&value, sender)?,
        "l2update" => emit_coinbase_l2update(&value, sender)?,
        "snapshot" => emit_coinbase_snapshot(&value, sender)?,
        "error" => {
            let err_msg = value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            warn!("coinbase stream error: {err_msg}");
        }
        "subscriptions" => {}
        _ => {}
    }

    Ok(())
}

fn emit_coinbase_ticker(
    value: &serde_json::Value,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let product_id = value
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing product_id in ticker".into()))?;

    let bid = parse_decimal_opt(value.get("best_bid"))
        .or_else(|_| parse_decimal_opt(value.get("bid")))?;
    let ask = parse_decimal_opt(value.get("best_ask"))
        .or_else(|_| parse_decimal_opt(value.get("ask")))?;
    let last = parse_decimal_opt(value.get("price"))?;
    let volume = parse_decimal_opt(value.get("volume_24h")).unwrap_or_else(|_| Decimal::ZERO);
    let timestamp = parse_timestamp(value.get("time"));

    let tick = MarketTick {
        symbol: product_id.to_string(),
        bid,
        ask,
        last,
        volume_24h: volume,
        timestamp,
    };

    let _ = sender.send(StreamMessage::Tick(tick));
    Ok(())
}

fn emit_coinbase_l2update(
    value: &serde_json::Value,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let product_id = value
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing product_id in l2update".into()))?;
    let timestamp = parse_timestamp(value.get("time"));

    if let Some(changes) = value.get("changes").and_then(|v| v.as_array()) {
        for change in changes {
            let Some(entries) = change.as_array() else {
                continue;
            };
            if entries.len() < 3 {
                continue;
            }
            let Some(side_str) = entries.get(0).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(price_str) = entries.get(1).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(size_str) = entries.get(2).and_then(|v| v.as_str()) else {
                continue;
            };

            let side = match side_str {
                "buy" => OrderSide::Buy,
                "sell" => OrderSide::Sell,
                _ => continue,
            };

            let price = Decimal::from_str(price_str).map_err(|err| {
                ExchangeError::Network(format!("invalid price in l2update: {err}"))
            })?;
            let quantity = Decimal::from_str(size_str).map_err(|err| {
                ExchangeError::Network(format!("invalid size in l2update: {err}"))
            })?;

            emit_coinbase_order_event(sender, product_id, side, price, quantity, timestamp);
        }
    }

    Ok(())
}

fn emit_coinbase_snapshot(
    value: &serde_json::Value,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let product_id = value
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing product_id in snapshot".into()))?;
    let timestamp = chrono::Utc::now();

    if let Some(bids) = value.get("bids").and_then(|v| v.as_array()) {
        for level in bids {
            let Some(entries) = level.as_array() else {
                continue;
            };
            if entries.len() < 2 {
                continue;
            }
            let Some(price_str) = entries.get(0).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(size_str) = entries.get(1).and_then(|v| v.as_str()) else {
                continue;
            };
            let price = Decimal::from_str(price_str).map_err(|err| {
                ExchangeError::Network(format!("invalid price in snapshot: {err}"))
            })?;
            let quantity = Decimal::from_str(size_str).map_err(|err| {
                ExchangeError::Network(format!("invalid size in snapshot: {err}"))
            })?;

            emit_coinbase_order_event(
                sender,
                product_id,
                OrderSide::Buy,
                price,
                quantity,
                timestamp,
            );
        }
    }

    if let Some(asks) = value.get("asks").and_then(|v| v.as_array()) {
        for level in asks {
            let Some(entries) = level.as_array() else {
                continue;
            };
            if entries.len() < 2 {
                continue;
            }
            let Some(price_str) = entries.get(0).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(size_str) = entries.get(1).and_then(|v| v.as_str()) else {
                continue;
            };
            let price = Decimal::from_str(price_str).map_err(|err| {
                ExchangeError::Network(format!("invalid price in snapshot: {err}"))
            })?;
            let quantity = Decimal::from_str(size_str).map_err(|err| {
                ExchangeError::Network(format!("invalid size in snapshot: {err}"))
            })?;

            emit_coinbase_order_event(
                sender,
                product_id,
                OrderSide::Sell,
                price,
                quantity,
                timestamp,
            );
        }
    }

    Ok(())
}

fn emit_coinbase_order_event(
    sender: &mpsc::UnboundedSender<StreamMessage>,
    product_id: &str,
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
        "{}-{}-{}",
        product_id.replace(['-', '_'], ""),
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
        exchange_id: ExchangeId::Coinbase,
        symbol: product_id.to_string(),
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

fn parse_decimal_opt(value: Option<&serde_json::Value>) -> Result<Decimal, ExchangeError> {
    let Some(raw) = value.and_then(|v| v.as_str()) else {
        return Ok(Decimal::ZERO);
    };
    Decimal::from_str(raw)
        .map_err(|err| ExchangeError::Network(format!("invalid decimal value '{raw}': {err}")))
}

fn parse_timestamp(value: Option<&serde_json::Value>) -> chrono::DateTime<chrono::Utc> {
    value
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now)
}

fn websocket_backoff(attempt: u32) -> Duration {
    let millis = (400.0 * 1.6_f64.powi(attempt.min(8) as i32)).min(10_000.0);
    Duration::from_millis(millis as u64)
}

// Coinbase API response structures
#[derive(Debug, Deserialize)]
struct CoinbaseErrorResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
struct CoinbaseProduct {
    id: String,
    base_currency: String,
    quote_currency: String,
    status: String,
    trading_disabled: bool,
}

#[derive(Debug, Deserialize)]
struct CoinbaseAccount {
    id: String,
    currency: String,
    balance: String,
    available: String,
    hold: String,
}

#[derive(Debug, Default, Serialize)]
struct CoinbaseOrderRequest {
    product_id: String,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    size: Option<String>,
    price: Option<String>,
    funds: Option<String>,
    time_in_force: Option<String>,
    cancel_after: Option<String>,
    post_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CoinbaseOrder {
    id: String,
    product_id: String,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    status: String,
    size: String,
    price: Option<String>,
    filled_size: String,
    executed_value: String,
    created_at: String,
    fill_fees: String,
}

#[derive(Debug, Deserialize)]
struct CoinbaseTicker {
    price: String,
    bid: String,
    ask: String,
    volume: String,
}

/// Convert Coinbase order to our ExchangeOrder format
fn convert_coinbase_order(coinbase_order: CoinbaseOrder) -> ExchangeOrder {
    let side = match coinbase_order.side.as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };

    let order_type = match coinbase_order.order_type.as_str() {
        "market" => OrderType::Market,
        "limit" => OrderType::Limit,
        "stop" => OrderType::Stop,
        "stop_limit" => OrderType::StopLimit,
        _ => OrderType::Market,
    };

    let status = match coinbase_order.status.as_str() {
        "pending" => OrderStatus::Pending,
        "open" => OrderStatus::Open,
        "active" => OrderStatus::Open,
        "done" => {
            let filled_size: Decimal = coinbase_order.filled_size.parse().unwrap_or_default();
            let size: Decimal = coinbase_order.size.parse().unwrap_or_default();
            if filled_size >= size {
                OrderStatus::Filled
            } else if filled_size > Decimal::ZERO {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Cancelled
            }
        }
        "cancelled" => OrderStatus::Cancelled,
        "rejected" => OrderStatus::Rejected,
        _ => OrderStatus::Pending,
    };

    let timestamp = chrono::DateTime::parse_from_rfc3339(&coinbase_order.created_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    ExchangeOrder {
        id: coinbase_order.id,
        exchange_id: ExchangeId::Coinbase,
        symbol: coinbase_order.product_id,
        side,
        order_type,
        quantity: coinbase_order.size.parse().unwrap_or_default(),
        price: coinbase_order.price.as_ref().and_then(|p| p.parse().ok()),
        status,
        timestamp,
        fills: vec![], // Would need separate API call to get fills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coinbase_connector_creation() {
        let config = CoinbaseConfig {
            api_key_name: "test_key".to_string(),
            private_key: "test_private_key".to_string(),
            sandbox: true,
            use_advanced_trade: false,
        };

        let connector = CoinbaseConnector::new(config);
        assert_eq!(connector.exchange_id(), ExchangeId::Coinbase);
        assert!(!connector.connected);
    }

    #[test]
    fn test_convert_coinbase_order() {
        let coinbase_order = CoinbaseOrder {
            id: "test-order-id".to_string(),
            product_id: "BTC-USD".to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            status: "open".to_string(),
            size: "1.0".to_string(),
            price: Some("50000.00".to_string()),
            filled_size: "0.0".to_string(),
            executed_value: "0.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            fill_fees: "0.0".to_string(),
        };

        let exchange_order = convert_coinbase_order(coinbase_order);

        assert_eq!(exchange_order.id, "test-order-id");
        assert_eq!(exchange_order.symbol, "BTC-USD");
        assert_eq!(exchange_order.side, OrderSide::Buy);
        assert_eq!(exchange_order.order_type, OrderType::Limit);
        assert_eq!(exchange_order.status, OrderStatus::Open);
        assert_eq!(exchange_order.quantity, Decimal::new(1, 0));
        assert_eq!(exchange_order.price, Some(Decimal::new(50000, 0)));
    }

    #[test]
    fn test_jwt_generation() {
        // Valid P-256 EC private key in PEM format (SEC1) - generated for testing only
        let test_private_key = r#"-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIFhWmy1vKAM0kdUENj+LriBPAp2TICRW/T7Ykh7iIpzwoAoGCCqGSM49
AwEHoUQDQgAEDk/z1l4dj9B/zW5S8EkAkPDdXo09vs07WT0Kl+3IWhvaQfjqguPF
fIC3Smd78+WzrwCo6c9qbnvoskng6jdLwg==
-----END EC PRIVATE KEY-----"#;

        let config = CoinbaseConfig {
            api_key_name: "test_key_name".to_string(),
            private_key: test_private_key.to_string(),
            sandbox: false,
            use_advanced_trade: true,
        };

        let connector = CoinbaseConnector::new(config);

        // Test JWT generation
        let jwt_result = connector.generate_jwt("GET", "/accounts");
        
        assert!(jwt_result.is_ok(), "JWT generation should succeed");
        
        let jwt = jwt_result.unwrap();
        
        // JWT should have three parts separated by dots
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 parts (header.payload.signature)");
        
        // Decode and verify header
        let header_json = String::from_utf8(
            URL_SAFE_NO_PAD.decode(parts[0]).expect("Header should be valid base64")
        ).expect("Header should be valid UTF-8");
        
        let header: serde_json::Value = serde_json::from_str(&header_json)
            .expect("Header should be valid JSON");
        
        assert_eq!(header["alg"], "ES256", "Algorithm should be ES256");
        assert_eq!(header["kid"], "test_key_name", "Key ID should match");
        assert_eq!(header["typ"], "JWT", "Type should be JWT");
        assert!(header["nonce"].is_string(), "Nonce should be present");
        
        // Verify nonce is in hex format (16 characters)
        let nonce = header["nonce"].as_str().unwrap();
        assert_eq!(nonce.len(), 16, "Nonce should be 16 hex characters");
        assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()), "Nonce should be hex");
        
        // Decode and verify claims
        let claims_json = String::from_utf8(
            URL_SAFE_NO_PAD.decode(parts[1]).expect("Claims should be valid base64")
        ).expect("Claims should be valid UTF-8");
        
        let claims: serde_json::Value = serde_json::from_str(&claims_json)
            .expect("Claims should be valid JSON");
        
        assert_eq!(claims["iss"], "cdp", "Issuer should be 'cdp'");
        assert_eq!(claims["sub"], "test_key_name", "Subject should match key name");
        assert!(claims["nbf"].is_number(), "Not before should be a timestamp");
        assert!(claims["exp"].is_number(), "Expiration should be a timestamp");
        assert!(claims["uri"].is_string(), "URI should be present");
        
        // Verify clock skew buffer is applied
        let now = chrono::Utc::now().timestamp();
        let nbf = claims["nbf"].as_i64().unwrap();
        let exp = claims["exp"].as_i64().unwrap();
        
        // nbf should be 5 seconds before now (with some tolerance)
        assert!(
            nbf <= now && nbf >= now - JWT_CLOCK_SKEW_BUFFER_SECONDS - 1,
            "Not before should include clock skew buffer"
        );
        
        // exp should be 120 seconds after now (with some tolerance)
        assert!(
            exp >= now + 119 && exp <= now + 121,
            "Expiration should be ~120 seconds from now"
        );
        
        // Verify URI format
        let uri = claims["uri"].as_str().unwrap();
        assert!(
            uri.starts_with("GET api.coinbase.com"),
            "URI should start with method and host"
        );
        
        // Signature should be 64 bytes (raw r||s format for ES256)
        let signature_bytes = URL_SAFE_NO_PAD.decode(parts[2])
            .expect("Signature should be valid base64");
        assert_eq!(
            signature_bytes.len(),
            64,
            "ES256 signature should be 64 bytes (32-byte r + 32-byte s)"
        );
    }
}

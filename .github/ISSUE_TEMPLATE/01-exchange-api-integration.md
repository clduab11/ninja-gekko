---
name: Exchange API Integration - Complete Implementation
about: Implement real exchange API connectivity for Coinbase, Binance.US, and OANDA
title: "[MILESTONE 1] Complete Exchange API Integration for Live Trading"
labels: critical, exchange-integration, trading-core, ai-agent
assignees: ''
---

## 🤖 GitHub Copilot Context Prompt

**Copy this into your Copilot chat before starting implementation:**

```
You are implementing MILESTONE 1: Exchange API Integration for a production Rust trading system. Follow November 2025 standards: use Rust 2021 edition with async/await, tokio 1.0+, comprehensive error handling via thiserror, structured logging with tracing spans, no unsafe code, trait-based abstractions. Implement REST APIs and WebSocket streams for Coinbase Advanced Trade, Binance.US, and OANDA with HMAC-SHA256 auth. Each function needs rustdoc comments, all errors use Result<T, ExchangeError>, rate limiters enforce exchange policies, reconnection uses exponential backoff. Write integration tests with wiremock, unit tests for parsers, achieve >80% coverage. Target <100ms order placement latency. Reference existing traits in crates/exchange-connectors/src/lib.rs. Use reqwest 0.11 for HTTP, tokio-tungstenite 0.21 for WebSocket, serde for JSON. Code must pass cargo clippy --all-targets -- -D warnings and cargo fmt. Follow event-driven patterns: emit events to event bus, parse WebSocket to StreamMessage enums. Security: no hardcoded credentials, use env vars, validate all inputs, audit log all API calls. This enables real trading—precision and correctness are critical.
```

---

## Overview

**Milestone**: Exchange API Integration
**Priority**: CRITICAL - Blocks all real trading functionality
**Implementation Scope**: Foundation for live trading operations
**Dependencies**: None - Base milestone for trading system
**AI Agent Role**: Complete implementation of REST + WebSocket connectivity for 3 exchanges

## Problem Statement

The exchange connector infrastructure exists with trait definitions and authentication scaffolding, but lacks actual REST API request execution and WebSocket message parsing. This blocks all real trading operations.

**Current State**:
- ✅ `ExchangeConnector` trait defined in `crates/exchange-connectors/src/lib.rs`
- ✅ Rate limiter infrastructure in place
- ✅ Authentication scaffolding (HMAC-SHA256 signatures)
- ❌ REST API request bodies not constructed
- ❌ WebSocket message parsing not implemented
- ❌ Order lifecycle management incomplete

**Target State**: Fully functional exchange connectivity enabling real order placement, cancellation, status tracking, and live market data streaming.

---

## Implementation Checklist

### Phase 1: Coinbase Advanced Trade API

**Files to Modify**: `crates/exchange-connectors/src/coinbase.rs`

#### 1.1 REST API Request Construction

- [ ] **Add request body structures following November 2025 Rust patterns**
  ```rust
  // Add to coinbase.rs
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, Serialize)]
  #[serde(rename_all = "snake_case")]
  struct PlaceOrderRequest {
      client_order_id: String,
      product_id: String,
      side: OrderSide,
      #[serde(flatten)]
      order_configuration: OrderConfiguration,
  }

  #[derive(Debug, Clone, Serialize)]
  #[serde(rename_all = "snake_case", untagged)]
  enum OrderConfiguration {
      Market {
          #[serde(rename = "market_market_ioc")]
          market_market_ioc: MarketOrderConfig
      },
      Limit {
          #[serde(rename = "limit_limit_gtc")]
          limit_limit_gtc: LimitOrderConfig
      },
  }

  #[derive(Debug, Clone, Serialize)]
  struct MarketOrderConfig {
      quote_size: String,
  }

  #[derive(Debug, Clone, Serialize)]
  struct LimitOrderConfig {
      base_size: String,
      limit_price: String,
      post_only: bool,
  }
  ```

- [ ] **Implement HMAC-SHA256 signature generation with 2025 crypto standards**
  ```rust
  use hmac::{Hmac, Mac};
  use sha2::Sha256;
  use hex;

  fn generate_signature(
      timestamp: u64,
      method: &str,
      request_path: &str,
      body: &str,
      secret: &str,
  ) -> Result<String, ExchangeError> {
      let message = format!("{}{}{}{}", timestamp, method, request_path, body);

      let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
          .map_err(|e| ExchangeError::AuthenticationError(e.to_string()))?;

      mac.update(message.as_bytes());
      Ok(hex::encode(mac.finalize().into_bytes()))
  }
  ```

- [ ] **Implement authenticated request helper with proper error handling**
  ```rust
  async fn send_authenticated_request<T, R>(
      &self,
      method: reqwest::Method,
      endpoint: &str,
      body: Option<&T>,
  ) -> ExchangeResult<R>
  where
      T: Serialize,
      R: DeserializeOwned,
  {
      let timestamp = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .map_err(|e| ExchangeError::SystemError(e.to_string()))?
          .as_secs();

      let body_str = if let Some(b) = body {
          serde_json::to_string(b)
              .map_err(|e| ExchangeError::SerializationError(e.to_string()))?
      } else {
          String::new()
      };

      let signature = self.generate_signature(
          timestamp,
          method.as_str(),
          endpoint,
          &body_str,
          &self.api_secret,
      )?;

      // Apply rate limiter before request
      self.rate_limiter.acquire().await?;

      let response = self.http_client
          .request(method.clone(), &format!("{}{}", self.base_url, endpoint))
          .header("CB-ACCESS-KEY", &self.api_key)
          .header("CB-ACCESS-SIGN", signature)
          .header("CB-ACCESS-TIMESTAMP", timestamp.to_string())
          .header("Content-Type", "application/json")
          .body(body_str)
          .send()
          .await
          .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;

      let status = response.status();
      let response_text = response.text().await
          .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;

      if !status.is_success() {
          return Err(ExchangeError::ApiError {
              status_code: status.as_u16(),
              message: response_text,
          });
      }

      serde_json::from_str(&response_text)
          .map_err(|e| ExchangeError::ParseError(e.to_string()))
  }
  ```

#### 1.2 Order Management Implementation

- [ ] **Implement `place_order()` method with comprehensive error handling**
  ```rust
  #[tracing::instrument(skip(self), fields(symbol, side, quantity))]
  async fn place_order(
      &self,
      symbol: &str,
      side: OrderSide,
      order_type: OrderType,
      quantity: Decimal,
      price: Option<Decimal>,
  ) -> ExchangeResult<ExchangeOrder> {
      let client_order_id = Uuid::new_v4().to_string();

      let order_config = match order_type {
          OrderType::Market => OrderConfiguration::Market {
              market_market_ioc: MarketOrderConfig {
                  quote_size: quantity.to_string(),
              }
          },
          OrderType::Limit => {
              let limit_price = price.ok_or_else(||
                  ExchangeError::InvalidParameter("Limit orders require price".into())
              )?;

              OrderConfiguration::Limit {
                  limit_limit_gtc: LimitOrderConfig {
                      base_size: quantity.to_string(),
                      limit_price: limit_price.to_string(),
                      post_only: false,
                  }
              }
          },
      };

      let request = PlaceOrderRequest {
          client_order_id: client_order_id.clone(),
          product_id: symbol.to_string(),
          side,
          order_configuration: order_config,
      };

      tracing::debug!(
          client_order_id = %client_order_id,
          "Placing order on Coinbase"
      );

      let response: CoinbaseOrderResponse = self
          .send_authenticated_request(
              reqwest::Method::POST,
              "/api/v3/brokerage/orders",
              Some(&request),
          )
          .await?;

      tracing::info!(
          order_id = %response.order_id,
          client_order_id = %client_order_id,
          "Order placed successfully"
      );

      Ok(ExchangeOrder {
          id: response.order_id,
          client_order_id: Some(client_order_id),
          symbol: symbol.to_string(),
          side,
          order_type,
          quantity,
          price,
          status: parse_order_status(&response.status),
          filled_quantity: response.filled_size.parse().unwrap_or(Decimal::ZERO),
          average_price: response.average_filled_price.and_then(|p| p.parse().ok()),
          timestamp: Utc::now(),
      })
  }
  ```

- [ ] **Implement `cancel_order()` method**
- [ ] **Implement `get_order_status()` method**
- [ ] **Implement `get_account_balances()` method**

#### 1.3 WebSocket Market Data Streaming (November 2025 async patterns)

- [ ] **Implement WebSocket connection with authentication**
  ```rust
  use tokio_tungstenite::{connect_async, tungstenite::Message};
  use futures_util::{StreamExt, SinkExt};

  #[tracing::instrument(skip(self))]
  async fn start_market_stream(
      &mut self,
      symbols: Vec<String>,
  ) -> ExchangeResult<Receiver<StreamMessage>> {
      let (tx, rx) = mpsc::unbounded_channel();

      let url = "wss://advanced-trade-ws.coinbase.com";
      let (ws_stream, _) = connect_async(url)
          .await
          .map_err(|e| ExchangeError::WebSocketError(e.to_string()))?;

      let (mut write, mut read) = ws_stream.split();

      // Send subscription message
      let subscribe_msg = serde_json::json!({
          "type": "subscribe",
          "product_ids": symbols,
          "channels": ["level2", "ticker", "matches"]
      });

      write.send(Message::Text(subscribe_msg.to_string()))
          .await
          .map_err(|e| ExchangeError::WebSocketError(e.to_string()))?;

      // Spawn message handler task
      let tx_clone = tx.clone();
      tokio::spawn(async move {
          let mut reconnect_attempts = 0u32;
          let max_reconnect_attempts = 10;

          loop {
              match read.next().await {
                  Some(Ok(Message::Text(text))) => {
                      reconnect_attempts = 0; // Reset on successful message

                      match parse_ws_message(&text) {
                          Ok(parsed) => {
                              if tx_clone.send(parsed).is_err() {
                                  tracing::warn!("Receiver dropped, closing WebSocket");
                                  break;
                              }
                          }
                          Err(e) => {
                              tracing::error!(error = %e, "Failed to parse WebSocket message");
                          }
                      }
                  }
                  Some(Ok(Message::Ping(payload))) => {
                      if let Err(e) = write.send(Message::Pong(payload)).await {
                          tracing::error!(error = %e, "Failed to send pong");
                          break;
                      }
                  }
                  Some(Ok(Message::Close(_))) | None => {
                      tracing::warn!("WebSocket connection closed");

                      // Attempt reconnection with exponential backoff
                      if reconnect_attempts < max_reconnect_attempts {
                          let backoff_ms = 2u64.pow(reconnect_attempts) * 1000;
                          reconnect_attempts += 1;

                          tracing::info!(
                              attempt = reconnect_attempts,
                              backoff_ms = backoff_ms,
                              "Reconnecting WebSocket"
                          );

                          tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

                          // TODO: Implement reconnection logic
                          // For now, break and require restart
                          break;
                      } else {
                          tracing::error!("Max reconnection attempts reached");
                          break;
                      }
                  }
                  Some(Err(e)) => {
                      tracing::error!(error = %e, "WebSocket error");
                      break;
                  }
                  _ => {}
              }
          }

          tracing::info!("WebSocket handler task exiting");
      });

      Ok(rx)
  }
  ```

- [ ] **Implement WebSocket message parser with November 2025 pattern matching**
  ```rust
  fn parse_ws_message(text: &str) -> Result<StreamMessage, ExchangeError> {
      let value: serde_json::Value = serde_json::from_str(text)
          .map_err(|e| ExchangeError::ParseError(e.to_string()))?;

      let msg_type = value["type"]
          .as_str()
          .ok_or_else(|| ExchangeError::ParseError("Missing type field".into()))?;

      match msg_type {
          "ticker" => {
              Ok(StreamMessage::Ticker {
                  symbol: value["product_id"].as_str()
                      .ok_or_else(|| ExchangeError::ParseError("Missing product_id".into()))?
                      .to_string(),
                  price: value["price"].as_str()
                      .ok_or_else(|| ExchangeError::ParseError("Missing price".into()))?
                      .parse()
                      .map_err(|e| ExchangeError::ParseError(format!("Invalid price: {}", e)))?,
                  volume: value["volume_24h"].as_str()
                      .ok_or_else(|| ExchangeError::ParseError("Missing volume_24h".into()))?
                      .parse()
                      .map_err(|e| ExchangeError::ParseError(format!("Invalid volume: {}", e)))?,
                  timestamp: Utc::now(),
              })
          }
          "l2update" => {
              Ok(StreamMessage::OrderBookUpdate {
                  symbol: value["product_id"].as_str()
                      .ok_or_else(|| ExchangeError::ParseError("Missing product_id".into()))?
                      .to_string(),
                  bids: parse_price_levels(&value["changes"])?,
                  asks: parse_price_levels(&value["changes"])?,
              })
          }
          "match" => {
              Ok(StreamMessage::Trade {
                  symbol: value["product_id"].as_str()
                      .ok_or_else(|| ExchangeError::ParseError("Missing product_id".into()))?
                      .to_string(),
                  price: value["price"].as_str()
                      .ok_or_else(|| ExchangeError::ParseError("Missing price".into()))?
                      .parse()
                      .map_err(|e| ExchangeError::ParseError(format!("Invalid price: {}", e)))?,
                  quantity: value["size"].as_str()
                      .ok_or_else(|| ExchangeError::ParseError("Missing size".into()))?
                      .parse()
                      .map_err(|e| ExchangeError::ParseError(format!("Invalid size: {}", e)))?,
                  side: match value["side"].as_str() {
                      Some("buy") => OrderSide::Buy,
                      Some("sell") => OrderSide::Sell,
                      _ => return Err(ExchangeError::ParseError("Invalid side".into())),
                  },
                  timestamp: Utc::now(),
              })
          }
          _ => Err(ExchangeError::ParseError(format!("Unknown message type: {}", msg_type))),
      }
  }
  ```

- [ ] **Add WebSocket reconnection logic with exponential backoff (completed above)**
- [ ] **Implement heartbeat/ping-pong handling (completed above)**

#### 1.4 Error Handling & Testing (November 2025 Standards)

- [ ] **Add Coinbase-specific error types using thiserror**
  ```rust
  use thiserror::Error;

  #[derive(Debug, Error)]
  pub enum CoinbaseError {
      #[error("Invalid API credentials")]
      InvalidCredentials,

      #[error("Rate limit exceeded, retry after {retry_after} seconds")]
      RateLimitExceeded { retry_after: u64 },

      #[error("Insufficient funds for order")]
      InsufficientFunds,

      #[error("Order not found: {0}")]
      OrderNotFound(String),

      #[error("Invalid order parameter: {0}")]
      InvalidOrderParameter(String),

      #[error("API error (status {status}): {message}")]
      ApiError { status: u16, message: String },

      #[error("Network error: {0}")]
      NetworkError(String),

      #[error("Parse error: {0}")]
      ParseError(String),
  }
  ```

- [ ] **Write integration tests following November 2025 async testing patterns**
  ```rust
  // Create crates/exchange-connectors/tests/coinbase_integration.rs
  use exchange_connectors::*;
  use wiremock::{MockServer, Mock, ResponseTemplate};
  use wiremock::matchers::{method, path, header};

  #[tokio::test]
  async fn test_place_order_success() {
      let mock_server = MockServer::start().await;

      Mock::given(method("POST"))
          .and(path("/api/v3/brokerage/orders"))
          .and(header("CB-ACCESS-KEY", "test_key"))
          .respond_with(ResponseTemplate::new(200)
              .set_body_json(serde_json::json!({
                  "order_id": "test-order-123",
                  "client_order_id": "client-123",
                  "status": "OPEN",
                  "filled_size": "0",
                  "average_filled_price": null
              })))
          .mount(&mock_server)
          .await;

      let mut connector = CoinbaseConnector::new_with_url(
          "test_key".into(),
          "test_secret".into(),
          mock_server.uri(),
      );

      let result = connector.place_order(
          "BTC-USD",
          OrderSide::Buy,
          OrderType::Market,
          Decimal::from_str("0.001").unwrap(),
          None,
      ).await;

      assert!(result.is_ok());
      let order = result.unwrap();
      assert_eq!(order.id, "test-order-123");
  }

  #[tokio::test]
  async fn test_rate_limit_error() {
      let mock_server = MockServer::start().await;

      Mock::given(method("POST"))
          .and(path("/api/v3/brokerage/orders"))
          .respond_with(ResponseTemplate::new(429)
              .set_body_json(serde_json::json!({
                  "error": "rate_limit_exceeded",
                  "retry_after": 60
              })))
          .mount(&mock_server)
          .await;

      let mut connector = CoinbaseConnector::new_with_url(
          "test_key".into(),
          "test_secret".into(),
          mock_server.uri(),
      );

      let result = connector.place_order(
          "BTC-USD",
          OrderSide::Buy,
          OrderType::Market,
          Decimal::from_str("0.001").unwrap(),
          None,
      ).await;

      assert!(result.is_err());
      match result.unwrap_err() {
          ExchangeError::RateLimitExceeded { .. } => {},
          e => panic!("Expected RateLimitExceeded, got {:?}", e),
      }
  }

  #[tokio::test]
  #[ignore] // Run only with `cargo test -- --ignored` to avoid hitting real API
  async fn test_real_api_connection() {
      // Load credentials from environment
      let api_key = std::env::var("COINBASE_API_KEY")
          .expect("COINBASE_API_KEY not set");
      let api_secret = std::env::var("COINBASE_API_SECRET")
          .expect("COINBASE_API_SECRET not set");

      let mut connector = CoinbaseConnector::new(api_key, api_secret);
      connector.connect().await.unwrap();

      // Test balance retrieval
      let balances = connector.get_account_balances().await.unwrap();
      assert!(!balances.is_empty());
  }
  ```

- [ ] **Add unit tests for signature generation**
- [ ] **Add unit tests for WebSocket message parsing**
- [ ] **Add property-based tests using proptest**

---

### Phase 2: Binance.US API

**Files to Modify**: `crates/exchange-connectors/src/binance_us.rs`

#### 2.1 Core Infrastructure

- [ ] **Set up API constants and base URLs**
  ```rust
  const BINANCE_US_API_BASE: &str = "https://api.binance.us";
  const BINANCE_US_WS_BASE: &str = "wss://stream.binance.us:9443";

  #[derive(Debug, Clone)]
  pub struct BinanceUsConnector {
      api_key: String,
      api_secret: String,
      http_client: reqwest::Client,
      rate_limiter: RateLimiter,
      request_weight_tracker: Arc<Mutex<RequestWeightTracker>>,
  }
  ```

- [ ] **Implement HMAC-SHA256 authentication (Binance-specific)**
  ```rust
  fn sign_request(&self, query_string: &str) -> String {
      use hmac::{Hmac, Mac};
      use sha2::Sha256;

      let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes())
          .expect("HMAC can take key of any size");
      mac.update(query_string.as_bytes());
      hex::encode(mac.finalize().into_bytes())
  }
  ```

- [ ] **Add timestamp synchronization** (Binance requires server time sync)
  ```rust
  #[tracing::instrument(skip(self))]
  async fn get_server_time(&self) -> ExchangeResult<i64> {
      let response: ServerTimeResponse = self.http_client
          .get(&format!("{}/api/v3/time", BINANCE_US_API_BASE))
          .send()
          .await
          .map_err(|e| ExchangeError::NetworkError(e.to_string()))?
          .json()
          .await
          .map_err(|e| ExchangeError::ParseError(e.to_string()))?;

      Ok(response.server_time)
  }

  async fn get_timestamp_with_offset(&self) -> ExchangeResult<i64> {
      // Cache server time offset to avoid excessive calls
      let local_time = Utc::now().timestamp_millis();
      let server_time = self.get_server_time().await?;
      let offset = server_time - local_time;

      Ok(local_time + offset)
  }
  ```

#### 2.2 Order Management

- [ ] **Implement `place_order()` with Binance order types**
- [ ] **Implement `cancel_order()` supporting both orderId and origClientOrderId**
- [ ] **Implement `get_order_status()`**
- [ ] **Implement `get_account_balances()` using `/api/v3/account` endpoint**

#### 2.3 WebSocket Streaming (November 2025 Combined Streams Pattern)

- [ ] **Implement combined streams for multiple symbols**
  ```rust
  // Binance uses combined stream format:
  // wss://stream.binance.us:9443/stream?streams=btcusdt@trade/btcusdt@depth

  async fn start_market_stream(
      &mut self,
      symbols: Vec<String>,
  ) -> ExchangeResult<Receiver<StreamMessage>> {
      let streams: Vec<String> = symbols.iter()
          .flat_map(|s| {
              let symbol_lower = s.replace("-", "").to_lowercase();
              vec![
                  format!("{}@trade", symbol_lower),
                  format!("{}@depth20", symbol_lower),
              ]
          })
          .collect();

      let url = format!(
          "{}/ stream?streams={}",
          BINANCE_US_WS_BASE,
          streams.join("/")
      );

      // ... rest of WebSocket implementation
  }
  ```

- [ ] **Parse trade stream messages**
- [ ] **Parse depth stream (order book) messages**
- [ ] **Implement user data stream for order updates**

#### 2.4 Rate Limiting (Weight-Based System)

- [ ] **Implement request weight tracking** (Binance uses weight-based limits)
  ```rust
  use std::collections::VecDeque;

  #[derive(Debug)]
  struct RequestWeightTracker {
      requests: VecDeque<(Instant, u32)>, // (timestamp, weight)
      window_duration: Duration,
      max_weight: u32,
  }

  impl RequestWeightTracker {
      fn new() -> Self {
          Self {
              requests: VecDeque::new(),
              window_duration: Duration::from_secs(60),
              max_weight: 1200, // Binance.US limit
          }
      }

      async fn can_make_request(&mut self, weight: u32) -> bool {
          self.cleanup_old_requests();

          let current_weight: u32 = self.requests.iter()
              .map(|(_, w)| w)
              .sum();

          current_weight + weight <= self.max_weight
      }

      fn record_request(&mut self, weight: u32) {
          self.requests.push_back((Instant::now(), weight));
      }

      fn cleanup_old_requests(&mut self) {
          let cutoff = Instant::now() - self.window_duration;

          while let Some(&(timestamp, _)) = self.requests.front() {
              if timestamp < cutoff {
                  self.requests.pop_front();
              } else {
                  break;
              }
          }
      }
  }
  ```

- [ ] **Add weight-based rate limiter with backpressure**

#### 2.5 Testing

- [ ] **Write integration tests using Binance.US testnet**
- [ ] **Add unit tests for signature generation**
- [ ] **Test weight calculation and rate limiting**

---

### Phase 3: OANDA API

**Files to Modify**: `crates/exchange-connectors/src/oanda.rs`

#### 3.1 Core Setup

- [ ] **Set up practice and live API URLs**
  ```rust
  const OANDA_PRACTICE_API: &str = "https://api-fxpractice.oanda.com";
  const OANDA_LIVE_API: &str = "https://api-fxtrade.oanda.com";
  const OANDA_STREAM_PRACTICE: &str = "https://stream-fxpractice.oanda.com";

  #[derive(Debug, Clone)]
  pub struct OandaConnector {
      api_token: String,
      account_id: String,
      base_url: String,
      stream_url: String,
      http_client: reqwest::Client,
      is_practice: bool,
  }
  ```

- [ ] **Implement Bearer token authentication (November 2025 pattern)**
  ```rust
  #[tracing::instrument(skip(self, body))]
  async fn send_authenticated_request<R>(
      &self,
      method: reqwest::Method,
      endpoint: &str,
      body: Option<String>,
  ) -> ExchangeResult<R>
  where
      R: DeserializeOwned,
  {
      let url = format!("{}{}", self.base_url, endpoint);

      let mut request = self.http_client
          .request(method.clone(), &url)
          .header("Authorization", format!("Bearer {}", self.api_token))
          .header("Content-Type", "application/json")
          .header("Accept-Datetime-Format", "RFC3339");

      if let Some(body_str) = body {
          request = request.body(body_str);
      }

      let response = request.send()
          .await
          .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;

      let status = response.status();
      let response_text = response.text().await
          .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;

      if !status.is_success() {
          return Err(ExchangeError::ApiError {
              status_code: status.as_u16(),
              message: response_text,
          });
      }

      serde_json::from_str(&response_text)
          .map_err(|e| ExchangeError::ParseError(e.to_string()))
  }
  ```

#### 3.2 Forex Order Management

- [ ] **Implement `place_order()` for forex market orders**
  ```rust
  // OANDA uses different order structure for forex
  #[derive(Debug, Serialize)]
  struct OandaOrderRequest {
      order: OandaOrder,
  }

  #[derive(Debug, Serialize)]
  #[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
  enum OandaOrder {
      Market {
          instrument: String,
          units: String,
          time_in_force: String,
          position_fill: String,
      },
      Limit {
          instrument: String,
          units: String,
          price: String,
          time_in_force: String,
      },
  }
  ```

- [ ] **Implement limit orders with take-profit and stop-loss**
- [ ] **Implement `cancel_order()`**
- [ ] **Implement `get_order_status()`**
- [ ] **Implement `get_account_balances()` for margin account**

#### 3.3 Streaming Pricing Data (HTTP Streaming, not WebSocket)

- [ ] **Implement pricing stream subscription**
  ```rust
  use futures_util::StreamExt;

  // OANDA uses HTTP streaming (not WebSocket)
  async fn start_pricing_stream(
      &self,
      instruments: Vec<String>,
  ) -> ExchangeResult<Receiver<StreamMessage>> {
      let (tx, rx) = mpsc::unbounded_channel();

      let url = format!(
          "{}/v3/accounts/{}/pricing/stream?instruments={}",
          self.stream_url,
          self.account_id,
          instruments.join(",")
      );

      let response = self.http_client
          .get(&url)
          .header("Authorization", format!("Bearer {}", self.api_token))
          .send()
          .await
          .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;

      let mut stream = response.bytes_stream();

      tokio::spawn(async move {
          let mut buffer = Vec::new();

          while let Some(chunk) = stream.next().await {
              match chunk {
                  Ok(bytes) => {
                      buffer.extend_from_slice(&bytes);

                      // Process newline-delimited JSON
                      while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                          let line = buffer.drain(..=newline_pos).collect::<Vec<_>>();
                          let line_str = String::from_utf8_lossy(&line);

                          if let Ok(msg) = parse_oanda_pricing(&line_str) {
                              if tx.send(msg).is_err() {
                                  return; // Receiver dropped
                              }
                          }
                      }
                  }
                  Err(e) => {
                      tracing::error!(error = %e, "Stream error");
                      break;
                  }
              }
          }
      });

      Ok(rx)
  }
  ```

- [ ] **Parse streaming price updates**
- [ ] **Implement transaction stream for order fills**

#### 3.4 Position Management (Forex-Specific)

- [ ] **Add position tracking for forex pairs**
- [ ] **Implement position close endpoints**
- [ ] **Handle partial position closes**

#### 3.5 Testing

- [ ] **Write integration tests using OANDA practice account**
- [ ] **Test forex-specific order types**
- [ ] **Validate streaming price data parsing**

---

### Phase 4: Unified Testing & Validation

**Files to Create**: `crates/exchange-connectors/tests/integration_tests.rs`

- [ ] **Create mock exchange server using wiremock**
  ```rust
  use wiremock::{MockServer, Mock, ResponseTemplate};
  use wiremock::matchers::{method, path, header};

  async fn setup_mock_exchange() -> MockServer {
      let mock_server = MockServer::start().await;

      // Mock order placement
      Mock::given(method("POST"))
          .and(path("/orders"))
          .respond_with(ResponseTemplate::new(200)
              .set_body_json(serde_json::json!({
                  "order_id": "test-123",
                  "status": "FILLED",
                  "filled_quantity": "1.0",
                  "average_price": "50000.0"
              })))
          .mount(&mock_server)
          .await;

      // Mock order status
      Mock::given(method("GET"))
          .and(path("/orders/test-123"))
          .respond_with(ResponseTemplate::new(200)
              .set_body_json(serde_json::json!({
                  "order_id": "test-123",
                  "status": "FILLED"
              })))
          .mount(&mock_server)
          .await;

      mock_server
  }
  ```

- [ ] **Add end-to-end order lifecycle tests**
  ```rust
  #[tokio::test]
  async fn test_full_order_lifecycle_all_exchanges() {
      let exchanges = vec!["coinbase", "binance_us", "oanda"];

      for exchange_name in exchanges {
          // Test sequence: connect -> place -> status -> cancel -> verify
          let mut connector = create_connector(exchange_name).await;

          // Connect
          connector.connect().await.expect("Failed to connect");

          // Place order
          let order = connector.place_order(
              "BTC-USD",
              OrderSide::Buy,
              OrderType::Limit,
              Decimal::from_str("0.001").unwrap(),
              Some(Decimal::from_str("30000").unwrap()),
          ).await.expect("Failed to place order");

          assert!(!order.id.is_empty());

          // Get status
          let status = connector.get_order_status(&order.id)
              .await
              .expect("Failed to get status");

          assert!(matches!(status, OrderStatus::Open | OrderStatus::Filled));

          // Cancel if still open
          if matches!(status, OrderStatus::Open) {
              connector.cancel_order(&order.id)
                  .await
                  .expect("Failed to cancel");

              // Verify cancellation
              let final_status = connector.get_order_status(&order.id)
                  .await
                  .expect("Failed to verify cancellation");

              assert_eq!(final_status, OrderStatus::Cancelled);
          }
      }
  }
  ```

- [ ] **Test rate limiter activation under load**
- [ ] **Validate WebSocket reconnection scenarios**
- [ ] **Add latency measurement tests** (verify <100ms target)
- [ ] **Create paper trading mode validation suite**

---

## Acceptance Criteria

### Functional Requirements (November 2025 Standards)

- [ ] All three exchanges can successfully authenticate using modern async patterns
- [ ] Orders can be placed, cancelled, and status retrieved for each exchange
- [ ] WebSocket/streaming market data works with automatic reconnection
- [ ] Account balances can be retrieved with proper error handling
- [ ] Rate limiting prevents API violations using exchange-specific strategies
- [ ] Reconnection logic handles network failures with exponential backoff
- [ ] All async operations use tokio 1.0+ runtime
- [ ] Structured logging via tracing with spans for request correlation

### Performance Requirements

- [ ] Order placement latency <100ms (measured via criterion benchmarks)
- [ ] WebSocket messages parsed within <10ms
- [ ] Rate limiters operate with <1ms overhead
- [ ] Memory usage remains stable under continuous streaming

### Code Quality (November 2025 Best Practices)

- [ ] All functions have comprehensive rustdoc comments with examples
- [ ] Zero `cargo clippy --all-targets -- -D warnings` violations
- [ ] Code formatted with `cargo fmt` (2021 edition style)
- [ ] Error types use `thiserror` crate properly
- [ ] All async functions use proper error propagation (`?` operator)
- [ ] No `unwrap()` or `expect()` in production paths (only in tests)
- [ ] Integration test coverage >80% (measured with cargo-tarpaulin)
- [ ] All public APIs have doc tests that compile and run

### Security & Reliability

- [ ] No hardcoded credentials (use environment variables)
- [ ] All inputs validated before use
- [ ] All API calls logged for audit trail
- [ ] WebSocket messages validated against schema
- [ ] Rate limiter prevents accidental API abuse
- [ ] Circuit breaker pattern for exchange connectivity
- [ ] Secrets never logged or exposed in errors

---

## Implementation Notes for AI Agents

### Implementation Phase Order

**Phase Sequence**:
1. Coinbase (most straightforward REST API)
2. Binance.US (adds weight-based rate limiting complexity)
3. OANDA (different paradigm: forex + HTTP streaming)

**Within Each Phase**:
1. Authentication layer first
2. Then REST API operations
3. Then streaming data
4. Finally comprehensive testing

### Key Patterns (November 2025 Standards)

**Authentication Pattern**:
```rust
// Modern async auth flow:
1. Generate timestamp (handle timezone correctly)
2. Construct message to sign (exchange-specific format)
3. Sign with HMAC-SHA256 using hmac/sha2 crates
4. Add proper headers (Authorization, API-KEY, etc.)
5. Send request with reqwest 0.11+
6. Parse response with serde, handle all error cases
```

**Error Handling Pattern**:
```rust
// Use thiserror for all error types:
#[derive(Debug, Error)]
pub enum ExchangeError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    // Contextualized errors
    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },
}

// Never use unwrap/expect in production:
response.json::<T>()
    .await
    .map_err(|e| ExchangeError::ParseError(e.to_string()))
```

**Async/Await Pattern**:
```rust
// Use tokio::spawn for background tasks:
tokio::spawn(async move {
    // Long-running task
});

// Use select! for timeout/cancellation:
tokio::select! {
    result = operation() => handle_result(result),
    _ = tokio::time::sleep(timeout) => handle_timeout(),
}
```

**WebSocket Pattern (2025)**:
```rust
// Use tokio-tungstenite with split pattern:
let (mut write, mut read) = ws_stream.split();

// Spawn separate tasks for read/write:
tokio::spawn(async move {
    while let Some(msg) = read.next().await {
        // Handle message
    }
});
```

### Testing Strategy

**Test Pyramid**:
1. **Unit tests** (60%): Individual functions, parsers, validators
2. **Integration tests** (30%): Mock servers with wiremock
3. **Manual tests** (10%): Real exchange testnets with `#[ignore]`

**Coverage Target**: >80% line coverage via cargo-tarpaulin

### Dependencies (November 2025 Versions)

```toml
[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# HTTP client
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# WebSocket
tokio-tungstenite = "0.21"
futures-util = "0.3"

# Cryptography
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1.33", features = ["serde"] }

[dev-dependencies]
wiremock = "0.6"
tokio-test = "0.4"
proptest = "1.4"
criterion = "0.5"
```

---

## Verification Commands

```bash
# Unit tests
cargo test -p exchange-connectors

# Integration tests with mock servers
cargo test -p exchange-connectors --test integration_tests

# Integration tests with REAL APIs (requires credentials)
COINBASE_API_KEY=xxx COINBASE_API_SECRET=xxx \
BINANCE_US_API_KEY=xxx BINANCE_US_API_SECRET=xxx \
OANDA_API_TOKEN=xxx OANDA_ACCOUNT_ID=xxx \
cargo test -p exchange-connectors -- --ignored

# Linting
cargo clippy -p exchange-connectors --all-targets -- -D warnings

# Format check
cargo fmt -p exchange-connectors -- --check

# Code coverage
cargo tarpaulin -p exchange-connectors --out Html

# Benchmarks
cargo bench -p exchange-connectors

# Doc tests
cargo test -p exchange-connectors --doc

# Build documentation
cargo doc -p exchange-connectors --no-deps --open
```

---

## Milestone Completion Criteria

This milestone is **COMPLETE** when:

✅ All checkboxes in all phases are checked
✅ All tests pass (`cargo test --all`)
✅ Clippy shows zero warnings
✅ Code coverage >80%
✅ Documentation builds without warnings
✅ Can successfully place, cancel, and track orders on all 3 exchanges
✅ WebSocket streams deliver real-time data with <10ms parse latency
✅ Rate limiters prevent API violations
✅ Integration tests validate end-to-end order lifecycle

**Next Milestone**: Once complete, proceed to MILESTONE 2 (Arbitrage Engine)

---

## Related Files & References

**Trait Definition**: `crates/exchange-connectors/src/lib.rs:1`
**Configuration**: `config/arbitrage.toml:20`
**Event Bus Integration**: `crates/event-bus/src/lib.rs`
**Database Layer**: `database/src/connection.rs`

**External Documentation**:
- [Coinbase Advanced Trade API](https://docs.cdp.coinbase.com/advanced-trade/docs/welcome)
- [Binance.US API Documentation](https://docs.binance.us/)
- [OANDA v20 REST API](https://developer.oanda.com/rest-live-v20/introduction/)
- [Rust Async Book (2025)](https://rust-lang.github.io/async-book/)
- [tokio Documentation](https://docs.rs/tokio/latest/tokio/)

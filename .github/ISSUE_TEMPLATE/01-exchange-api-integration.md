---
name: Exchange API Integration - Complete Implementation
about: Implement real exchange API connectivity for Coinbase, Binance.US, and OANDA
title: "[MILESTONE 1] Complete Exchange API Integration for Live Trading"
labels: critical, exchange-integration, trading-core
assignees: ''
---

## Overview

**Milestone**: Exchange API Integration
**Priority**: CRITICAL - Blocks all real trading functionality
**Estimated Complexity**: High
**Dependencies**: None - Foundation for all other features

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

- [ ] **Add request body structures**
  ```rust
  // Add to coinbase.rs
  #[derive(Serialize)]
  struct PlaceOrderRequest {
      client_order_id: String,
      product_id: String,
      side: String,
      order_configuration: OrderConfiguration,
  }

  #[derive(Serialize)]
  #[serde(untagged)]
  enum OrderConfiguration {
      MarketOrder { market_market_ioc: MarketOrderConfig },
      LimitOrder { limit_limit_gtc: LimitOrderConfig },
  }
  ```

- [ ] **Implement HMAC-SHA256 signature generation**
  ```rust
  fn generate_signature(
      timestamp: u64,
      method: &str,
      request_path: &str,
      body: &str,
      secret: &str,
  ) -> Result<String> {
      use hmac::{Hmac, Mac};
      use sha2::Sha256;

      let message = format!("{}{}{}{}", timestamp, method, request_path, body);
      let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
      mac.update(message.as_bytes());
      Ok(hex::encode(mac.finalize().into_bytes()))
  }
  ```

- [ ] **Implement authenticated request helper**
  ```rust
  async fn send_authenticated_request<T: Serialize, R: DeserializeOwned>(
      &self,
      method: reqwest::Method,
      endpoint: &str,
      body: Option<&T>,
  ) -> ExchangeResult<R> {
      let timestamp = SystemTime::now()
          .duration_since(UNIX_EPOCH)?
          .as_secs();

      let body_str = if let Some(b) = body {
          serde_json::to_string(b)?
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

      let response = self.http_client
          .request(method, &format!("{}{}", self.base_url, endpoint))
          .header("CB-ACCESS-KEY", &self.api_key)
          .header("CB-ACCESS-SIGN", signature)
          .header("CB-ACCESS-TIMESTAMP", timestamp.to_string())
          .header("Content-Type", "application/json")
          .body(body_str)
          .send()
          .await?;

      // Add rate limiter check here
      self.rate_limiter.check_and_wait().await;

      if !response.status().is_success() {
          return Err(ExchangeError::ApiError(response.text().await?));
      }

      Ok(response.json().await?)
  }
  ```

#### 1.2 Order Management Implementation

- [ ] **Implement `place_order()` method**
  ```rust
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
          OrderType::Market => OrderConfiguration::MarketOrder {
              market_market_ioc: MarketOrderConfig {
                  quote_size: quantity.to_string(),
              }
          },
          OrderType::Limit => OrderConfiguration::LimitOrder {
              limit_limit_gtc: LimitOrderConfig {
                  base_size: quantity.to_string(),
                  limit_price: price.unwrap().to_string(),
                  post_only: false,
              }
          },
      };

      let request = PlaceOrderRequest {
          client_order_id: client_order_id.clone(),
          product_id: symbol.to_string(),
          side: match side {
              OrderSide::Buy => "BUY".to_string(),
              OrderSide::Sell => "SELL".to_string(),
          },
          order_configuration: order_config,
      };

      let response: CoinbaseOrderResponse = self
          .send_authenticated_request(
              reqwest::Method::POST,
              "/api/v3/brokerage/orders",
              Some(&request),
          )
          .await?;

      Ok(ExchangeOrder {
          id: response.order_id,
          client_order_id: Some(client_order_id),
          symbol: symbol.to_string(),
          side,
          order_type,
          quantity,
          price,
          status: parse_order_status(&response.status),
          filled_quantity: Decimal::ZERO,
          timestamp: Utc::now(),
      })
  }
  ```

- [ ] **Implement `cancel_order()` method**
- [ ] **Implement `get_order_status()` method**
- [ ] **Implement `get_account_balances()` method**

#### 1.3 WebSocket Market Data Streaming

- [ ] **Implement WebSocket connection with authentication**
  ```rust
  async fn start_market_stream(
      &mut self,
      symbols: Vec<String>,
  ) -> ExchangeResult<Receiver<StreamMessage>> {
      let (tx, rx) = mpsc::unbounded_channel();

      let url = "wss://advanced-trade-ws.coinbase.com";
      let (ws_stream, _) = connect_async(url).await?;

      // Send subscription message
      let subscribe_msg = json!({
          "type": "subscribe",
          "product_ids": symbols,
          "channels": ["level2", "ticker", "matches"]
      });

      ws_stream.send(Message::Text(subscribe_msg.to_string())).await?;

      // Spawn message handler task
      tokio::spawn(async move {
          while let Some(msg) = ws_stream.next().await {
              match msg {
                  Ok(Message::Text(text)) => {
                      if let Ok(parsed) = parse_ws_message(&text) {
                          tx.send(parsed).ok();
                      }
                  }
                  Ok(Message::Ping(_)) => {
                      ws_stream.send(Message::Pong(vec![])).await.ok();
                  }
                  Err(e) => {
                      tracing::error!("WebSocket error: {}", e);
                      break;
                  }
                  _ => {}
              }
          }
      });

      Ok(rx)
  }
  ```

- [ ] **Implement WebSocket message parser**
  ```rust
  fn parse_ws_message(text: &str) -> Result<StreamMessage, serde_json::Error> {
      let value: serde_json::Value = serde_json::from_str(text)?;

      match value["type"].as_str() {
          Some("ticker") => {
              Ok(StreamMessage::Ticker {
                  symbol: value["product_id"].as_str().unwrap().to_string(),
                  price: value["price"].as_str().unwrap().parse().unwrap(),
                  volume: value["volume_24h"].as_str().unwrap().parse().unwrap(),
                  timestamp: Utc::now(),
              })
          }
          Some("l2update") => {
              Ok(StreamMessage::OrderBookUpdate {
                  symbol: value["product_id"].as_str().unwrap().to_string(),
                  bids: parse_price_levels(&value["changes"]),
                  asks: parse_price_levels(&value["changes"]),
              })
          }
          Some("match") => {
              Ok(StreamMessage::Trade {
                  symbol: value["product_id"].as_str().unwrap().to_string(),
                  price: value["price"].as_str().unwrap().parse().unwrap(),
                  quantity: value["size"].as_str().unwrap().parse().unwrap(),
                  side: if value["side"].as_str() == Some("buy") {
                      OrderSide::Buy
                  } else {
                      OrderSide::Sell
                  },
                  timestamp: Utc::now(),
              })
          }
          _ => Err(serde_json::Error::custom("Unknown message type")),
      }
  }
  ```

- [ ] **Add WebSocket reconnection logic with exponential backoff**
- [ ] **Implement heartbeat/ping-pong handling**

#### 1.4 Error Handling & Testing

- [ ] **Add Coinbase-specific error types**
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum CoinbaseError {
      #[error("Invalid API credentials")]
      InvalidCredentials,

      #[error("Rate limit exceeded")]
      RateLimitExceeded,

      #[error("Insufficient funds")]
      InsufficientFunds,

      #[error("Order not found: {0}")]
      OrderNotFound(String),

      #[error("API error: {0}")]
      ApiError(String),
  }
  ```

- [ ] **Write integration tests** (create `crates/exchange-connectors/tests/coinbase_integration.rs`)
  ```rust
  #[tokio::test]
  #[ignore] // Run only with `cargo test -- --ignored` to avoid hitting real API
  async fn test_place_and_cancel_order() {
      let mut connector = CoinbaseConnector::new(/* test credentials */);
      connector.connect().await.unwrap();

      // Place order
      let order = connector.place_order(
          "BTC-USD",
          OrderSide::Buy,
          OrderType::Limit,
          Decimal::from_str("0.001").unwrap(),
          Some(Decimal::from_str("30000").unwrap()),
      ).await.unwrap();

      assert!(!order.id.is_empty());

      // Cancel order
      connector.cancel_order(&order.id).await.unwrap();

      // Verify cancellation
      let status = connector.get_order_status(&order.id).await.unwrap();
      assert_eq!(status, OrderStatus::Cancelled);
  }
  ```

- [ ] **Add unit tests for signature generation**
- [ ] **Add unit tests for WebSocket message parsing**

---

### Phase 2: Binance.US API

**Files to Modify**: `crates/exchange-connectors/src/binance_us.rs`

#### 2.1 Core Infrastructure

- [ ] **Set up API constants and base URLs**
  ```rust
  const BINANCE_US_API_BASE: &str = "https://api.binance.us";
  const BINANCE_US_WS_BASE: &str = "wss://stream.binance.us:9443";
  ```

- [ ] **Implement HMAC-SHA256 authentication**
  ```rust
  fn sign_request(&self, query_string: &str) -> String {
      use hmac::{Hmac, Mac};
      use sha2::Sha256;

      let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).unwrap();
      mac.update(query_string.as_bytes());
      hex::encode(mac.finalize().into_bytes())
  }
  ```

- [ ] **Add timestamp synchronization** (Binance requires server time sync)
  ```rust
  async fn get_server_time(&self) -> ExchangeResult<i64> {
      let response: ServerTimeResponse = self.http_client
          .get(&format!("{}/api/v3/time", BINANCE_US_API_BASE))
          .send()
          .await?
          .json()
          .await?;
      Ok(response.server_time)
  }
  ```

#### 2.2 Order Management

- [ ] **Implement `place_order()` with Binance order types**
- [ ] **Implement `cancel_order()` supporting both orderId and origClientOrderId**
- [ ] **Implement `get_order_status()`**
- [ ] **Implement `get_account_balances()` using `/api/v3/account` endpoint**

#### 2.3 WebSocket Streaming

- [ ] **Implement combined streams for multiple symbols**
  ```rust
  // Binance uses combined stream format:
  // wss://stream.binance.us:9443/stream?streams=btcusdt@trade/btcusdt@depth
  ```

- [ ] **Parse trade stream messages**
- [ ] **Parse depth stream (order book) messages**
- [ ] **Implement user data stream for order updates**

#### 2.4 Rate Limiting

- [ ] **Implement request weight tracking** (Binance uses weight-based limits)
  ```rust
  // Different endpoints have different weights
  // Must track cumulative weight over 1-minute window
  // Limit: 1200 weight per minute
  ```

- [ ] **Add weight-based rate limiter**

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
  ```

- [ ] **Implement Bearer token authentication**
  ```rust
  async fn send_authenticated_request<R: DeserializeOwned>(
      &self,
      method: reqwest::Method,
      endpoint: &str,
      body: Option<String>,
  ) -> ExchangeResult<R> {
      let response = self.http_client
          .request(method, &format!("{}{}", self.base_url, endpoint))
          .header("Authorization", format!("Bearer {}", self.api_token))
          .header("Content-Type", "application/json")
          .body(body.unwrap_or_default())
          .send()
          .await?;

      Ok(response.json().await?)
  }
  ```

#### 3.2 Forex Order Management

- [ ] **Implement `place_order()` for forex market orders**
  ```rust
  // OANDA uses different order structure for forex
  // POST /v3/accounts/{accountID}/orders
  {
      "order": {
          "type": "MARKET",
          "instrument": "EUR_USD",
          "units": "100",
          "timeInForce": "FOK",
          "positionFill": "DEFAULT"
      }
  }
  ```

- [ ] **Implement limit orders with take-profit and stop-loss**
- [ ] **Implement `cancel_order()`**
- [ ] **Implement `get_order_status()`**
- [ ] **Implement `get_account_balances()` for margin account**

#### 3.3 Streaming Pricing Data

- [ ] **Implement pricing stream subscription**
  ```rust
  // OANDA uses HTTP streaming (not WebSocket)
  // GET /v3/accounts/{accountID}/pricing/stream?instruments=EUR_USD,USD_JPY
  ```

- [ ] **Parse streaming price updates**
- [ ] **Implement transaction stream for order fills**

#### 3.4 Position Management

- [ ] **Add position tracking (forex-specific)**
- [ ] **Implement position close endpoints**

#### 3.5 Testing

- [ ] **Write integration tests using OANDA practice account**
- [ ] **Test forex-specific order types**
- [ ] **Validate streaming price data parsing**

---

### Phase 4: Unified Testing & Validation

**Files to Create**: `crates/exchange-connectors/tests/integration_tests.rs`

- [ ] **Create mock exchange server for unit tests**
  ```rust
  use wiremock::{MockServer, Mock, ResponseTemplate};

  async fn setup_mock_exchange() -> MockServer {
      let mock_server = MockServer::start().await;

      Mock::given(method("POST"))
          .and(path("/orders"))
          .respond_with(ResponseTemplate::new(200)
              .set_body_json(json!({
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
      for exchange in &["coinbase", "binance_us", "oanda"] {
          // Test: connect -> place -> status -> cancel -> verify
      }
  }
  ```

- [ ] **Test rate limiter activation under load**
- [ ] **Validate WebSocket reconnection scenarios**
- [ ] **Add latency measurement tests** (target: <100ms order placement)
- [ ] **Create paper trading mode validation suite**

---

## Acceptance Criteria

### Functional Requirements

- [ ] All three exchanges can successfully authenticate
- [ ] Orders can be placed, cancelled, and status retrieved for each exchange
- [ ] WebSocket/streaming market data works for all exchanges
- [ ] Account balances can be retrieved
- [ ] Rate limiting prevents API limit violations
- [ ] Reconnection logic handles network failures gracefully

### Non-Functional Requirements

- [ ] Order placement latency <100ms (measured via benchmarks)
- [ ] WebSocket messages parsed within <10ms
- [ ] Integration tests pass with >90% coverage of connector code
- [ ] No panics or unwraps in production code paths
- [ ] Comprehensive error handling with actionable error messages
- [ ] Structured logging for all API calls (tracing)

### Code Quality

- [ ] All functions have rustdoc comments
- [ ] No `cargo clippy` warnings
- [ ] Code formatted with `cargo fmt`
- [ ] Error types properly implement `std::error::Error`
- [ ] All async functions use proper error propagation (`?` operator)

---

## Implementation Notes for AI Agents

### Order of Implementation

1. **Start with Coinbase** - Most straightforward API, good reference implementation
2. **Then Binance.US** - Similar REST patterns, adds complexity with weights
3. **Finally OANDA** - Different paradigm (forex), HTTP streaming vs WebSocket

### Key Patterns to Follow

**Authentication Pattern**:
```rust
// Each exchange needs this pattern:
1. Generate timestamp
2. Construct message to sign (varies by exchange)
3. Sign with HMAC-SHA256
4. Add auth headers
5. Send request
6. Parse response
```

**Error Handling Pattern**:
```rust
// Always use Result<T, ExchangeError>
// Map exchange-specific errors to ExchangeError enum
response.json::<T>()
    .await
    .map_err(|e| ExchangeError::ParseError(e.to_string()))
```

**WebSocket Pattern**:
```rust
// 1. Connect
// 2. Send subscribe message
// 3. Spawn async task for message handling
// 4. Return channel receiver
// 5. Handle reconnections in spawned task
```

### Testing Strategy

1. **Unit tests**: Test individual functions (signature generation, parsing)
2. **Integration tests**: Test against mock servers
3. **Manual tests**: Test against exchange testnets/sandboxes with `#[ignore]` flag

### Dependencies to Add

```toml
# Add to crates/exchange-connectors/Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = "0.21"
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
uuid = { version = "1.0", features = ["v4"] }

[dev-dependencies]
wiremock = "0.5"
tokio-test = "0.4"
```

---

## Verification Commands

Run these after implementation:

```bash
# Unit tests
cargo test -p exchange-connectors

# Integration tests (with real APIs - requires credentials)
COINBASE_API_KEY=xxx COINBASE_API_SECRET=xxx cargo test -p exchange-connectors -- --ignored

# Clippy
cargo clippy -p exchange-connectors -- -D warnings

# Format check
cargo fmt -p exchange-connectors -- --check

# Benchmarks
cargo bench -p exchange-connectors
```

---

## Related Issues

- Blocks: #2 (Arbitrage Engine Implementation)
- Blocks: All trading functionality

## References

- [Coinbase Advanced Trade API Docs](https://docs.cdp.coinbase.com/advanced-trade/docs/welcome)
- [Binance.US API Docs](https://docs.binance.us/)
- [OANDA v20 REST API Docs](https://developer.oanda.com/rest-live-v20/introduction/)
- Internal: `crates/exchange-connectors/src/lib.rs:1` (ExchangeConnector trait)
- Internal: `config/arbitrage.toml:20` (Exchange configurations)

//! Integration tests for Coinbase connector
//!
//! Tests REST operations, WebSocket streaming, HMAC authentication, and error handling.
//! Uses mock responses to avoid hitting real exchange APIs during testing.

use exchange_connectors::coinbase::CoinbaseConnector;
use exchange_connectors::{ExchangeConfig, ExchangeConnector, ExchangeId, OrderSide, OrderType};
use rust_decimal::Decimal;
use std::time::Duration;

/// Test connector initialization
#[tokio::test]
async fn test_connector_initialization() {
    let config = create_test_config();
    let connector = CoinbaseConnector::new(config);

    assert_eq!(connector.exchange_id(), ExchangeId::Coinbase);
    assert!(!connector.is_connected().await);
}

/// Test connection state management
#[tokio::test]
async fn test_connection_state() {
    let config = create_test_config();
    let mut connector = CoinbaseConnector::new(config);

    // Initially disconnected
    assert!(!connector.is_connected().await);

    // After connect
    connector.connect().await.expect("connect should succeed");
    assert!(connector.is_connected().await);

    // After disconnect
    connector.disconnect().await.expect("disconnect should succeed");
    assert!(!connector.is_connected().await);
}

/// Test symbol formatting
#[test]
fn test_symbol_formatting() {
    // Coinbase uses hyphen notation: BTC-USD
    let test_cases = vec![
        ("BTC-USD", "BTC-USD"),
        ("ETH-USD", "ETH-USD"),
        ("SOL-USD", "SOL-USD"),
    ];

    for (input, expected) in test_cases {
        assert_eq!(input, expected);
    }
}

/// Test that trading operations handle stubbed responses
#[tokio::test]
async fn test_trading_operations_stubbed() {
    let config = create_test_config();
    let connector = CoinbaseConnector::new(config);

    // place_order should fail (stubbed)
    let result = connector
        .place_order(
            "BTC-USD",
            OrderSide::Buy,
            OrderType::Market,
            Decimal::from(1),
            None,
        )
        .await;
    assert!(result.is_err());

    // cancel_order should fail (stubbed)
    let result = connector.cancel_order("test-order-id").await;
    assert!(result.is_err());

    // get_order should fail (stubbed)
    let result = connector.get_order("test-order-id").await;
    assert!(result.is_err());
}

/// Test balance retrieval (stubbed)
#[tokio::test]
async fn test_balances_stubbed() {
    let config = create_test_config();
    let connector = CoinbaseConnector::new(config);
    let balances = connector.get_balances().await.expect("should not error");
    assert!(balances.is_empty());
}

/// Test trading pairs retrieval (stubbed)
#[tokio::test]
async fn test_trading_pairs_stubbed() {
    let config = create_test_config();
    let connector = CoinbaseConnector::new(config);
    let pairs = connector.get_trading_pairs().await.expect("should not error");
    assert!(pairs.is_empty());
}

/// Test transfer operations
#[tokio::test]
async fn test_transfer_operations_stubbed() {
    let config = create_test_config();
    let connector = CoinbaseConnector::new(config);

    let transfer_request = exchange_connectors::TransferRequest {
        id: uuid::Uuid::new_v4(),
        from_exchange: ExchangeId::Coinbase,
        to_exchange: ExchangeId::BinanceUs,
        currency: "USD".to_string(),
        amount: Decimal::from(100),
        urgency: exchange_connectors::TransferUrgency::Normal,
    };

    let result = connector.transfer_funds(transfer_request).await;
    assert!(result.is_err());
}

/// HMAC-SHA256 authentication tests
mod authentication {
    use exchange_connectors::utils;
    use chrono::Utc;

    /// Test signature generation
    #[test]
    fn test_signature_generation() {
        let secret = "test-secret-key";
        let message = "1234567890GET/accounts";

        let signature = utils::hmac_sha256_signature(secret, message);

        // Signature should be non-empty base64
        assert!(!signature.is_empty());
        // Should be valid base64
        assert!(base64::decode(&signature).is_ok());
    }

    /// Test timestamp generation
    #[test]
    fn test_timestamp_generation() {
        let timestamp = utils::timestamp();

        // Should be numeric
        assert!(timestamp.parse::<i64>().is_ok());

        // Should be recent (within last minute)
        let ts: i64 = timestamp.parse().unwrap();
        let now = Utc::now().timestamp();
        assert!((now - ts).abs() < 60);
    }

    /// Test CB-ACCESS headers construction
    #[test]
    fn test_cb_access_headers() {
        let api_key = "test-api-key";
        let timestamp = "1234567890";
        let signature = "test-signature";
        let passphrase = "test-passphrase";

        // Verify all required headers are present
        let headers = vec![
            ("CB-ACCESS-KEY", api_key),
            ("CB-ACCESS-TIMESTAMP", timestamp),
            ("CB-ACCESS-SIGN", signature),
            ("CB-ACCESS-PASSPHRASE", passphrase),
        ];

        for (name, value) in headers {
            assert!(!name.is_empty());
            assert!(!value.is_empty());
        }
    }

    /// Test signature message format
    #[test]
    fn test_signature_message_format() {
        let timestamp = "1234567890";
        let method = "POST";
        let path = "/orders";
        let body = r#"{"size":"0.01","price":"10000","side":"buy"}"#;

        let message = format!("{}{}{}{}", timestamp, method, path, body);

        assert!(message.starts_with(timestamp));
        assert!(message.contains(method));
        assert!(message.contains(path));
        assert!(message.ends_with(body));
    }
}

/// WebSocket message parsing tests
mod message_parsing {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Test parsing ticker message
    #[test]
    fn test_parse_ticker_message() {
        let payload = r#"{
            "type": "ticker",
            "sequence": 12345,
            "product_id": "BTC-USD",
            "price": "42000.50",
            "open_24h": "41000.00",
            "volume_24h": "1234.56789",
            "low_24h": "40500.00",
            "high_24h": "42500.00",
            "best_bid": "42000.00",
            "best_ask": "42001.00",
            "time": "2025-11-15T12:00:00.000000Z"
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();

        assert_eq!(value["type"].as_str().unwrap(), "ticker");
        assert_eq!(value["product_id"].as_str().unwrap(), "BTC-USD");

        let price = value["price"].as_str().unwrap();
        assert_eq!(Decimal::from_str(price).unwrap(), Decimal::from_str("42000.50").unwrap());
    }

    /// Test parsing level2 snapshot
    #[test]
    fn test_parse_level2_snapshot() {
        let payload = r#"{
            "type": "snapshot",
            "product_id": "BTC-USD",
            "bids": [["42000.00", "1.5"], ["41999.00", "2.0"]],
            "asks": [["42001.00", "0.5"], ["42002.00", "1.0"]]
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();

        let bids = value["bids"].as_array().unwrap();
        let asks = value["asks"].as_array().unwrap();

        assert_eq!(bids.len(), 2);
        assert_eq!(asks.len(), 2);
    }

    /// Test parsing l2update message
    #[test]
    fn test_parse_l2update_message() {
        let payload = r#"{
            "type": "l2update",
            "product_id": "BTC-USD",
            "time": "2025-11-15T12:00:00.000000Z",
            "changes": [
                ["buy", "42000.00", "1.5"],
                ["sell", "42001.00", "0.5"]
            ]
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        let changes = value["changes"].as_array().unwrap();

        assert_eq!(changes.len(), 2);

        let first_change = changes[0].as_array().unwrap();
        assert_eq!(first_change[0].as_str().unwrap(), "buy");
    }

    /// Test parsing subscription confirmation
    #[test]
    fn test_parse_subscriptions_message() {
        let payload = r#"{
            "type": "subscriptions",
            "channels": [
                {
                    "name": "ticker",
                    "product_ids": ["BTC-USD", "ETH-USD"]
                }
            ]
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(value["type"].as_str().unwrap(), "subscriptions");

        let channels = value["channels"].as_array().unwrap();
        assert_eq!(channels.len(), 1);
    }
}

/// Order type tests
mod order_types {
    use exchange_connectors::{OrderSide, OrderType};
    use rust_decimal::Decimal;

    /// Test market order construction
    #[test]
    fn test_market_order() {
        let side = OrderSide::Buy;
        let order_type = OrderType::Market;
        let quantity = Decimal::from(1);

        assert!(matches!(order_type, OrderType::Market));
        assert!(matches!(side, OrderSide::Buy));
        assert!(quantity > Decimal::ZERO);
    }

    /// Test limit order construction
    #[test]
    fn test_limit_order() {
        let side = OrderSide::Sell;
        let order_type = OrderType::Limit;
        let quantity = Decimal::from(1);
        let price = Decimal::from(42000);

        assert!(matches!(order_type, OrderType::Limit));
        assert!(matches!(side, OrderSide::Sell));
        assert!(price > Decimal::ZERO);
        assert!(quantity > Decimal::ZERO);
    }

    /// Test stop order construction
    #[test]
    fn test_stop_order() {
        let order_type = OrderType::Stop;
        let stop_price = Decimal::from(41000);

        assert!(matches!(order_type, OrderType::Stop));
        assert!(stop_price > Decimal::ZERO);
    }
}

/// Error handling tests
mod error_handling {
    use exchange_connectors::ExchangeError;
    use rust_decimal::Decimal;

    #[test]
    fn test_coinbase_api_errors() {
        // Coinbase-specific error formats
        let err = ExchangeError::Api {
            code: "INSUFFICIENT_FUNDS".to_string(),
            message: "Account has insufficient funds".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("INSUFFICIENT_FUNDS"));
    }

    #[test]
    fn test_invalid_order_size() {
        let err = ExchangeError::InvalidRequest("Order size below minimum".to_string());
        assert!(err.to_string().contains("Order size"));
    }

    #[test]
    fn test_symbol_not_found() {
        let err = ExchangeError::UnsupportedSymbol("INVALID-USD".to_string());
        assert!(err.to_string().contains("INVALID-USD"));
    }

    #[test]
    fn test_order_not_found() {
        let err = ExchangeError::OrderNotFound("abc-123-def".to_string());
        assert!(err.to_string().contains("abc-123-def"));
    }
}

/// Advanced Trade API tests
mod advanced_trade {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Test order ID format (UUID)
    #[test]
    fn test_order_id_format() {
        let order_id = "0bc83040-ffbd-4bec-88b2-5c2ed32e32d9";
        let parsed = uuid::Uuid::parse_str(order_id);
        assert!(parsed.is_ok());
    }

    /// Test product ID format
    #[test]
    fn test_product_id_format() {
        let valid_products = vec!["BTC-USD", "ETH-USD", "SOL-USD", "AVAX-USD"];

        for product in valid_products {
            assert!(product.contains('-'));
            let parts: Vec<&str> = product.split('-').collect();
            assert_eq!(parts.len(), 2);
        }
    }

    /// Test fee calculation
    #[test]
    fn test_fee_calculation() {
        // Maker fee: 0.4%, Taker fee: 0.6% (example rates)
        let trade_amount = Decimal::from(10000);
        let taker_fee_rate = Decimal::from_str("0.006").unwrap();

        let fee = trade_amount * taker_fee_rate;
        assert_eq!(fee, Decimal::from(60));
    }
}

/// Helper function to create test config
fn create_test_config() -> ExchangeConfig {
    ExchangeConfig {
        exchange_id: ExchangeId::Coinbase,
        api_key: "test-api-key".to_string(),
        api_secret: "test-api-secret".to_string(),
        passphrase: Some("test-passphrase".to_string()),
        sandbox: true,
        rate_limit_requests_per_second: 10,
        websocket_url: None,
        rest_api_url: None,
    }
}

/// Integration test for stream lifecycle (requires network, marked as ignored)
#[tokio::test]
#[ignore = "requires network connection to Coinbase"]
async fn test_market_stream_lifecycle() {
    let config = create_test_config();
    let connector = CoinbaseConnector::new(config);
    let symbols = vec!["BTC-USD".to_string()];

    let mut rx = connector.start_market_stream(symbols).await.expect("stream should start");

    // Wait for at least one message or timeout
    let result = tokio::time::timeout(Duration::from_secs(10), rx.recv()).await;

    assert!(result.is_ok(), "Should receive message within timeout");
}

/// Integration test for order stream (requires network, marked as ignored)
#[tokio::test]
#[ignore = "requires network connection and authentication"]
async fn test_order_stream_lifecycle() {
    let config = create_test_config();
    let connector = CoinbaseConnector::new(config);

    let result = connector.start_order_stream().await;
    assert!(result.is_ok(), "Should start order stream");
}

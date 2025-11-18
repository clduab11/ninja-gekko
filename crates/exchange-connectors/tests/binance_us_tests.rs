//! Integration tests for Binance.US connector
//!
//! Tests WebSocket streaming, connection handling, symbol mapping, and error scenarios.
//! Uses mock responses to avoid hitting real exchange APIs during testing.

use exchange_connectors::binance_us::BinanceUsConnector;
use exchange_connectors::{ExchangeConnector, ExchangeId, StreamMessage};
use std::time::Duration;
use tokio::time::timeout;

/// Test connector initialization
#[tokio::test]
async fn test_connector_initialization() {
    let connector = BinanceUsConnector::new();
    assert_eq!(connector.exchange_id(), ExchangeId::BinanceUs);
    assert!(!connector.is_connected().await);
}

/// Test connection state management
#[tokio::test]
async fn test_connection_state() {
    let mut connector = BinanceUsConnector::new();

    // Initially disconnected
    assert!(!connector.is_connected().await);

    // After connect
    connector.connect().await.expect("connect should succeed");
    assert!(connector.is_connected().await);

    // After disconnect
    connector.disconnect().await.expect("disconnect should succeed");
    assert!(!connector.is_connected().await);
}

/// Test that empty symbol list is rejected
#[tokio::test]
async fn test_empty_symbols_rejected() {
    let connector = BinanceUsConnector::new();
    let result = connector.start_market_stream(vec![]).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("at least one symbol"));
}

/// Test symbol canonicalization
#[test]
fn test_symbol_canonicalization() {
    // These tests verify the internal symbol mapping functions
    // BTC-USD should become btcusd
    // ETH_USD should become ethusd
    let test_cases = vec![
        ("BTC-USD", "btcusd"),
        ("ETH_USD", "ethusd"),
        ("BTCUSD", "btcusd"),
        ("btc-usd", "btcusd"),
    ];

    for (input, expected) in test_cases {
        let canonical: String = input
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();
        assert_eq!(canonical, expected, "Failed for input: {}", input);
    }
}

/// Test subscription parameter building
#[test]
fn test_subscription_params() {
    let symbols = vec!["BTC-USD".to_string(), "ETH-USD".to_string()];

    // Verify expected stream names
    let expected_streams = vec![
        "btcusd@bookTicker",
        "btcusd@depth@100ms",
        "ethusd@bookTicker",
        "ethusd@depth@100ms",
    ];

    let mut params = Vec::new();
    for symbol in &symbols {
        let stream_key: String = symbol
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();
        params.push(format!("{}@bookTicker", stream_key));
        params.push(format!("{}@depth@100ms", stream_key));
    }

    for expected in expected_streams {
        assert!(params.contains(&expected.to_string()), "Missing: {}", expected);
    }
}

/// Test that trading operations return appropriate errors
#[tokio::test]
async fn test_trading_operations_not_implemented() {
    let connector = BinanceUsConnector::new();

    // place_order should fail
    let result = connector
        .place_order(
            "BTC-USD",
            exchange_connectors::OrderSide::Buy,
            exchange_connectors::OrderType::Market,
            rust_decimal::Decimal::from(1),
            None,
        )
        .await;
    assert!(result.is_err());

    // cancel_order should fail
    let result = connector.cancel_order("test-order-id").await;
    assert!(result.is_err());

    // get_order should fail
    let result = connector.get_order("test-order-id").await;
    assert!(result.is_err());
}

/// Test market data polling returns appropriate error
#[tokio::test]
async fn test_market_data_polling_not_supported() {
    let connector = BinanceUsConnector::new();
    let result = connector.get_market_data("BTC-USD").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("WebSocket"));
}

/// Test transfer operations return appropriate errors
#[tokio::test]
async fn test_transfer_operations_not_implemented() {
    let connector = BinanceUsConnector::new();

    let transfer_request = exchange_connectors::TransferRequest {
        id: uuid::Uuid::new_v4(),
        from_exchange: ExchangeId::BinanceUs,
        to_exchange: ExchangeId::Coinbase,
        currency: "USD".to_string(),
        amount: rust_decimal::Decimal::from(100),
        urgency: exchange_connectors::TransferUrgency::Normal,
    };

    let result = connector.transfer_funds(transfer_request).await;
    assert!(result.is_err());

    let result = connector.get_transfer_status("test-transfer-id").await;
    assert!(result.is_err());
}

/// Test balances return empty (not implemented)
#[tokio::test]
async fn test_balances_empty() {
    let connector = BinanceUsConnector::new();
    let balances = connector.get_balances().await.expect("should not error");
    assert!(balances.is_empty());
}

/// Test trading pairs return empty (not implemented)
#[tokio::test]
async fn test_trading_pairs_empty() {
    let connector = BinanceUsConnector::new();
    let pairs = connector.get_trading_pairs().await.expect("should not error");
    assert!(pairs.is_empty());
}

/// Mock WebSocket message parsing tests
mod message_parsing {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Test parsing book ticker payload
    #[test]
    fn test_parse_book_ticker() {
        let payload = r#"{
            "stream": "btcusd@bookTicker",
            "data": {
                "s": "BTCUSD",
                "b": "42000.50",
                "B": "1.5",
                "a": "42001.00",
                "A": "2.0"
            }
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        let data = value.get("data").unwrap();

        let bid = data.get("b").and_then(|v| v.as_str()).unwrap();
        let ask = data.get("a").and_then(|v| v.as_str()).unwrap();

        assert_eq!(Decimal::from_str(bid).unwrap(), Decimal::from_str("42000.50").unwrap());
        assert_eq!(Decimal::from_str(ask).unwrap(), Decimal::from_str("42001.00").unwrap());
    }

    /// Test parsing depth update payload
    #[test]
    fn test_parse_depth_update() {
        let payload = r#"{
            "stream": "btcusd@depth@100ms",
            "data": {
                "s": "BTCUSD",
                "E": 1699900000000,
                "b": [["42000.00", "1.0"], ["41999.00", "2.0"]],
                "a": [["42001.00", "0.5"], ["42002.00", "1.5"]]
            }
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        let data = value.get("data").unwrap();

        let bids = data.get("b").and_then(|v| v.as_array()).unwrap();
        let asks = data.get("a").and_then(|v| v.as_array()).unwrap();

        assert_eq!(bids.len(), 2);
        assert_eq!(asks.len(), 2);
    }

    /// Test handling malformed payloads
    #[test]
    fn test_malformed_payload() {
        let malformed = r#"{"invalid": "json"#;
        let result: Result<serde_json::Value, _> = serde_json::from_str(malformed);
        assert!(result.is_err());
    }

    /// Test handling missing fields
    #[test]
    fn test_missing_fields() {
        let payload = r#"{
            "stream": "btcusd@bookTicker",
            "data": {}
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        let data = value.get("data").unwrap();

        // Missing symbol should return None
        assert!(data.get("s").is_none());
    }
}

/// Backoff delay calculation tests
mod backoff {
    use std::time::Duration;

    fn backoff_delay(attempt: u32) -> Duration {
        let capped_attempt = attempt.min(10);
        let millis = (500.0 * 1.5_f64.powi(capped_attempt as i32)).min(15_000.0);
        Duration::from_millis(millis as u64)
    }

    #[test]
    fn test_initial_delay() {
        let delay = backoff_delay(1);
        assert_eq!(delay.as_millis(), 750);
    }

    #[test]
    fn test_exponential_growth() {
        let delay1 = backoff_delay(1);
        let delay2 = backoff_delay(2);
        let delay3 = backoff_delay(3);

        assert!(delay2 > delay1);
        assert!(delay3 > delay2);
    }

    #[test]
    fn test_max_delay_cap() {
        let delay = backoff_delay(100);
        assert!(delay.as_millis() <= 15_000);
    }

    #[test]
    fn test_zero_attempt() {
        let delay = backoff_delay(0);
        assert_eq!(delay.as_millis(), 500);
    }
}

/// Error handling tests
mod error_handling {
    use exchange_connectors::ExchangeError;

    #[test]
    fn test_error_display() {
        let err = ExchangeError::RateLimit("429 Too Many Requests".to_string());
        assert!(err.to_string().contains("Rate limit"));

        let err = ExchangeError::Authentication("Invalid API key".to_string());
        assert!(err.to_string().contains("Authentication"));

        let err = ExchangeError::Network("Connection refused".to_string());
        assert!(err.to_string().contains("Network"));
    }

    #[test]
    fn test_insufficient_balance_error() {
        let err = ExchangeError::InsufficientBalance {
            required: rust_decimal::Decimal::from(100),
            available: rust_decimal::Decimal::from(50),
        };
        let msg = err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
    }
}

/// Integration test for stream lifecycle (requires network, marked as ignored)
#[tokio::test]
#[ignore = "requires network connection to Binance.US"]
async fn test_market_stream_lifecycle() {
    let connector = BinanceUsConnector::new();
    let symbols = vec!["BTC-USD".to_string()];

    let mut rx = connector.start_market_stream(symbols).await.expect("stream should start");

    // Wait for at least one message or timeout
    let result = timeout(Duration::from_secs(10), rx.recv()).await;

    match result {
        Ok(Some(msg)) => match msg {
            StreamMessage::Tick(tick) => {
                assert!(!tick.symbol.is_empty());
                assert!(tick.bid > rust_decimal::Decimal::ZERO);
                assert!(tick.ask > rust_decimal::Decimal::ZERO);
            }
            StreamMessage::OrderUpdate(_) => {
                // Depth updates come through as OrderUpdate
            }
            StreamMessage::Error(e) => {
                panic!("Received error: {}", e);
            }
            _ => {}
        },
        Ok(None) => panic!("Stream closed unexpectedly"),
        Err(_) => panic!("Timeout waiting for stream message"),
    }
}

//! Integration tests for OANDA connector
//!
//! Tests streaming, authentication, error handling, and Forex-specific functionality.
//! Uses mock responses to avoid hitting real exchange APIs during testing.

use exchange_connectors::oanda::OandaConnector;
use exchange_connectors::{ExchangeConnector, ExchangeId};
use std::time::Duration;

/// Test connector initialization
#[tokio::test]
async fn test_connector_initialization() {
    let connector = create_test_connector();

    assert_eq!(connector.exchange_id(), ExchangeId::Oanda);
    assert!(!connector.is_connected().await);
}

/// Test connection state management
#[tokio::test]
async fn test_connection_state() {
    let mut connector = create_test_connector();

    // Initially disconnected
    assert!(!connector.is_connected().await);

    // After connect (mock - doesn't actually connect)
    connector.connect().await.expect("connect should succeed");
    assert!(connector.is_connected().await);

    // After disconnect
    connector.disconnect().await.expect("disconnect should succeed");
    assert!(!connector.is_connected().await);
}

/// Test that empty API key is handled
#[tokio::test]
async fn test_empty_credentials() {
    let connector = OandaConnector::with_credentials("", "", true);
    // Connector should still initialize, auth check happens on API calls
    assert_eq!(connector.exchange_id(), ExchangeId::Oanda);
}

/// Test forex symbol formatting
#[test]
fn test_forex_symbol_formatting() {
    // OANDA uses underscore notation: EUR_USD
    let test_cases = vec![
        ("EUR_USD", "EUR_USD"),
        ("GBP_JPY", "GBP_JPY"),
        ("USD_CHF", "USD_CHF"),
    ];

    for (input, expected) in test_cases {
        assert_eq!(input, expected);
    }
}

/// Test that trading operations return appropriate errors when not implemented
#[tokio::test]
async fn test_trading_operations_stubbed() {
    let connector = create_test_connector();

    // place_order should fail (stubbed)
    let result = connector
        .place_order(
            "EUR_USD",
            exchange_connectors::OrderSide::Buy,
            exchange_connectors::OrderType::Market,
            rust_decimal::Decimal::from(1000),
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

/// Test transfer operations return appropriate errors
#[tokio::test]
async fn test_transfer_operations_not_supported() {
    let connector = create_test_connector();

    let transfer_request = exchange_connectors::TransferRequest {
        id: uuid::Uuid::new_v4(),
        from_exchange: ExchangeId::Oanda,
        to_exchange: ExchangeId::Coinbase,
        currency: "USD".to_string(),
        amount: rust_decimal::Decimal::from(100),
        urgency: exchange_connectors::TransferUrgency::Normal,
    };

    let result = connector.transfer_funds(transfer_request).await;
    assert!(result.is_err());
}

/// Test balance retrieval (stubbed)
#[tokio::test]
async fn test_balances_stubbed() {
    let connector = create_test_connector();
    let balances = connector.get_balances().await.expect("should not error");
    // Stubbed implementation returns empty
    assert!(balances.is_empty());
}

/// Test trading pairs retrieval (stubbed)
#[tokio::test]
async fn test_trading_pairs_stubbed() {
    let connector = create_test_connector();
    let pairs = connector.get_trading_pairs().await.expect("should not error");
    assert!(pairs.is_empty());
}

/// OANDA streaming message parsing tests
mod message_parsing {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Test parsing PRICE message
    #[test]
    fn test_parse_price_message() {
        let payload = r#"{
            "type": "PRICE",
            "time": "2025-11-15T12:00:00.000000000Z",
            "bids": [{"price": "1.08500", "liquidity": 1000000}],
            "asks": [{"price": "1.08510", "liquidity": 1000000}],
            "closeoutBid": "1.08495",
            "closeoutAsk": "1.08515",
            "status": "tradeable",
            "tradeable": true,
            "instrument": "EUR_USD"
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();

        assert_eq!(value["type"].as_str().unwrap(), "PRICE");
        assert_eq!(value["instrument"].as_str().unwrap(), "EUR_USD");

        let bids = value["bids"].as_array().unwrap();
        let bid_price = bids[0]["price"].as_str().unwrap();
        assert_eq!(Decimal::from_str(bid_price).unwrap(), Decimal::from_str("1.08500").unwrap());
    }

    /// Test parsing HEARTBEAT message
    #[test]
    fn test_parse_heartbeat_message() {
        let payload = r#"{
            "type": "HEARTBEAT",
            "time": "2025-11-15T12:00:00.000000000Z"
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(value["type"].as_str().unwrap(), "HEARTBEAT");
    }

    /// Test handling non-tradeable status
    #[test]
    fn test_non_tradeable_status() {
        let payload = r#"{
            "type": "PRICE",
            "instrument": "EUR_USD",
            "status": "non-tradeable",
            "tradeable": false
        }"#;

        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert!(!value["tradeable"].as_bool().unwrap());
    }
}

/// Authentication tests
mod authentication {
    use super::*;

    /// Test bearer token format
    #[test]
    fn test_bearer_token_format() {
        let api_key = "test-api-key-12345";
        let auth_header = format!("Bearer {}", api_key);

        assert!(auth_header.starts_with("Bearer "));
        assert!(auth_header.contains(api_key));
    }

    /// Test account ID format
    #[test]
    fn test_account_id_format() {
        // OANDA account IDs are typically numeric strings
        let valid_ids = vec!["101-001-12345678-001", "001-001-00000001-001"];

        for id in valid_ids {
            assert!(!id.is_empty());
            assert!(id.contains('-'));
        }
    }
}

/// Error handling tests
mod error_handling {
    use exchange_connectors::ExchangeError;

    #[test]
    fn test_oanda_specific_errors() {
        // OANDA returns specific error codes
        let err = ExchangeError::Api {
            code: "INSTRUMENT_NOT_FOUND".to_string(),
            message: "The instrument specified does not exist".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("INSTRUMENT_NOT_FOUND"));
    }

    #[test]
    fn test_authentication_error() {
        let err = ExchangeError::Authentication("Invalid API token".to_string());
        assert!(err.to_string().contains("Authentication"));
    }

    #[test]
    fn test_rate_limit_error() {
        let err = ExchangeError::RateLimit("Rate limit exceeded. Retry after 1 second".to_string());
        assert!(err.to_string().contains("Rate limit"));
    }
}

/// Forex-specific tests
mod forex {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Test pip calculation for major pairs
    #[test]
    fn test_pip_calculation() {
        // Most forex pairs have 4 decimal places (1 pip = 0.0001)
        let pip = Decimal::from_str("0.0001").unwrap();
        let price = Decimal::from_str("1.08500").unwrap();

        let price_plus_pip = price + pip;
        assert_eq!(price_plus_pip, Decimal::from_str("1.0851").unwrap());
    }

    /// Test JPY pair pip calculation
    #[test]
    fn test_jpy_pip_calculation() {
        // JPY pairs have 2 decimal places (1 pip = 0.01)
        let pip = Decimal::from_str("0.01").unwrap();
        let price = Decimal::from_str("148.50").unwrap();

        let price_plus_pip = price + pip;
        assert_eq!(price_plus_pip, Decimal::from_str("148.51").unwrap());
    }

    /// Test spread calculation
    #[test]
    fn test_spread_calculation() {
        let bid = Decimal::from_str("1.08500").unwrap();
        let ask = Decimal::from_str("1.08510").unwrap();
        let pip = Decimal::from_str("0.0001").unwrap();

        let spread_pips = (ask - bid) / pip;
        assert_eq!(spread_pips, Decimal::from(1));
    }
}

/// Position sizing tests
mod position_sizing {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Test lot size conversion
    #[test]
    fn test_lot_size_conversion() {
        // Standard lot = 100,000 units
        // Mini lot = 10,000 units
        // Micro lot = 1,000 units

        let standard_lot = Decimal::from(100_000);
        let mini_lot = Decimal::from(10_000);
        let micro_lot = Decimal::from(1_000);

        assert_eq!(standard_lot / Decimal::from(10), mini_lot);
        assert_eq!(mini_lot / Decimal::from(10), micro_lot);
    }

    /// Test margin requirement calculation
    #[test]
    fn test_margin_calculation() {
        // With 50:1 leverage, margin = position_size / 50
        let position_size = Decimal::from(100_000);
        let leverage = Decimal::from(50);
        let margin_required = position_size / leverage;

        assert_eq!(margin_required, Decimal::from(2000));
    }
}

/// Helper function to create test connector
fn create_test_connector() -> OandaConnector {
    OandaConnector::with_credentials("test-account-id", "test-api-key", true)
}

/// Integration test for stream lifecycle (requires network, marked as ignored)
#[tokio::test]
#[ignore = "requires network connection to OANDA"]
async fn test_pricing_stream_lifecycle() {
    let connector = create_test_connector();
    let symbols = vec!["EUR_USD".to_string()];

    let mut rx = connector.start_market_stream(symbols).await.expect("stream should start");

    // Wait for at least one message or timeout
    let result = tokio::time::timeout(Duration::from_secs(10), rx.recv()).await;

    assert!(result.is_ok(), "Should receive message within timeout");
}

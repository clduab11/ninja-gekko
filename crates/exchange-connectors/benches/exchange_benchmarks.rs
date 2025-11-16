//! Performance benchmarks for exchange connectors
//!
//! Validates performance targets from MILESTONE 1:
//! - Order placement latency <100ms
//! - WebSocket message parsing <10ms
//! - Rate limiter overhead <1ms
//!
//! Run with: cargo bench -p exchange-connectors

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use exchange_connectors::*;
use exchange_connectors::utils::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;

/// Benchmark HMAC-SHA256 signature generation
/// Target: Should complete in <1ms
fn bench_hmac_signature(c: &mut Criterion) {
    let secret = "test_secret_key_for_benchmarking";
    let message = "1234567890GET/accounts";

    c.bench_function("hmac_sha256_signature", |b| {
        b.iter(|| {
            black_box(hmac_sha256_signature(
                black_box(secret),
                black_box(message)
            ))
        })
    });
}

/// Benchmark timestamp generation
/// Target: Should complete in <100μs
fn bench_timestamp_generation(c: &mut Criterion) {
    c.bench_function("timestamp_generation", |b| {
        b.iter(|| {
            black_box(timestamp())
        })
    });
}

/// Benchmark decimal to string conversion
/// Target: Should complete in <10μs
fn bench_decimal_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal_conversion");

    for precision in [2, 8, 18].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(precision),
            precision,
            |b, &precision| {
                let value = Decimal::from_str("123.456789012345678").unwrap();
                b.iter(|| {
                    black_box(decimal_to_string(black_box(value), black_box(precision)))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark rate limiter acquisition
/// Target: <1ms overhead
fn bench_rate_limiter(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("rate_limiter_acquire", |b| {
        let rate_limiter = RateLimiter::new(10); // 10 req/sec

        b.to_async(&rt).iter(|| async {
            black_box(rate_limiter.acquire().await.unwrap())
        })
    });
}

/// Benchmark rate limiter under load
/// Target: Maintain <1ms even under contention
fn bench_rate_limiter_concurrent(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("rate_limiter_concurrent_10", |b| {
        let rate_limiter = std::sync::Arc::new(RateLimiter::new(100)); // Higher limit for test

        b.to_async(&rt).iter(|| {
            let limiter = rate_limiter.clone();
            async move {
                let mut handles = vec![];
                for _ in 0..10 {
                    let limiter = limiter.clone();
                    handles.push(tokio::spawn(async move {
                        limiter.acquire().await.unwrap()
                    }));
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            }
        })
    });
}

/// Benchmark order struct serialization
/// Target: <100μs for JSON serialization
fn bench_order_serialization(c: &mut Criterion) {
    use chrono::Utc;

    let order = ExchangeOrder {
        id: "benchmark-order-123".to_string(),
        exchange_id: ExchangeId::Coinbase,
        symbol: "BTC-USD".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: Decimal::from_str("0.01").unwrap(),
        price: Some(Decimal::from_str("50000.00").unwrap()),
        status: OrderStatus::Open,
        timestamp: Utc::now(),
        fills: vec![],
    };

    c.bench_function("order_serialization", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&order)).unwrap())
        })
    });
}

/// Benchmark order struct deserialization
/// Target: <100μs for JSON deserialization
fn bench_order_deserialization(c: &mut Criterion) {
    let order_json = r#"{
        "id": "benchmark-order-123",
        "exchange_id": "Coinbase",
        "symbol": "BTC-USD",
        "side": "Buy",
        "order_type": "Limit",
        "quantity": "0.01",
        "price": "50000.00",
        "status": "Open",
        "timestamp": "2025-11-16T00:00:00Z",
        "fills": []
    }"#;

    c.bench_function("order_deserialization", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<ExchangeOrder>(black_box(order_json)).unwrap())
        })
    });
}

/// Benchmark WebSocket message parsing simulation
/// Target: <10ms per message
fn bench_websocket_message_parse(c: &mut Criterion) {
    let tick_message = r#"{
        "type": "ticker",
        "product_id": "BTC-USD",
        "price": "50000.00",
        "open_24h": "49000.00",
        "volume_24h": "123.456",
        "low_24h": "48500.00",
        "high_24h": "50500.00",
        "volume_30d": "45678.90",
        "best_bid": "49999.50",
        "best_ask": "50000.50",
        "side": "buy",
        "time": "2025-11-16T00:00:00.000000Z",
        "trade_id": 12345,
        "last_size": "0.01"
    }"#;

    c.bench_function("websocket_ticker_parse", |b| {
        b.iter(|| {
            // Simulate parsing ticker message
            black_box(serde_json::from_str::<serde_json::Value>(black_box(tick_message)).unwrap())
        })
    });
}

/// Benchmark batch order creation (simulating high-frequency trading)
/// Target: <1ms for creating 100 order structs
fn bench_batch_order_creation(c: &mut Criterion) {
    use chrono::Utc;

    c.bench_function("create_100_orders", |b| {
        b.iter(|| {
            let mut orders = Vec::with_capacity(100);
            for i in 0..100 {
                orders.push(ExchangeOrder {
                    id: format!("order-{}", i),
                    exchange_id: ExchangeId::Coinbase,
                    symbol: "BTC-USD".to_string(),
                    side: if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell },
                    order_type: OrderType::Limit,
                    quantity: Decimal::from_str("0.01").unwrap(),
                    price: Some(Decimal::from(50000 + i)),
                    status: OrderStatus::Pending,
                    timestamp: Utc::now(),
                    fills: vec![],
                });
            }
            black_box(orders)
        })
    });
}

/// Benchmark error creation and propagation
/// Target: <10μs for error instantiation
fn bench_error_creation(c: &mut Criterion) {
    c.bench_function("exchange_error_creation", |b| {
        b.iter(|| {
            black_box(ExchangeError::Api {
                code: "400".to_string(),
                message: "Bad Request: Invalid parameter".to_string(),
            })
        })
    });
}

/// Configure criterion with proper parameters
fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3))
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets =
        bench_hmac_signature,
        bench_timestamp_generation,
        bench_decimal_conversion,
        bench_rate_limiter,
        bench_rate_limiter_concurrent,
        bench_order_serialization,
        bench_order_deserialization,
        bench_websocket_message_parse,
        bench_batch_order_creation,
        bench_error_creation
}

criterion_main!(benches);

//! Rate limiting tests for exchange connectors
//!
//! Tests the RateLimiter implementation using governor for API call throttling.
//! Ensures correct behavior under various load conditions and performance requirements.

use exchange_connectors::RateLimiter;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Barrier;

/// Test rate limiter initialization
#[tokio::test]
async fn test_rate_limiter_initialization() {
    let limiter = RateLimiter::new(10);

    // Should successfully acquire first permit
    let result = limiter.acquire().await;
    assert!(result.is_ok());
}

/// Test basic rate limiting
#[tokio::test]
async fn test_basic_rate_limiting() {
    let limiter = RateLimiter::new(5); // 5 requests per second

    let start = Instant::now();

    // Make 5 requests - should complete quickly
    for _ in 0..5 {
        limiter.acquire().await.unwrap();
    }

    let elapsed = start.elapsed();
    // First 5 requests should complete within ~1 second
    assert!(elapsed < Duration::from_secs(2), "First batch took too long: {:?}", elapsed);
}

/// Test that rate limiter throttles excess requests
#[tokio::test]
async fn test_throttling() {
    let limiter = RateLimiter::new(10); // 10 requests per second

    let start = Instant::now();

    // Make 15 requests - should take at least 500ms for the extra 5
    for _ in 0..15 {
        limiter.acquire().await.unwrap();
    }

    let elapsed = start.elapsed();
    // Should take at least 400ms (5 extra requests at 10/sec)
    assert!(elapsed >= Duration::from_millis(400), "Throttling not working: {:?}", elapsed);
}

/// Test concurrent access to rate limiter
#[tokio::test]
async fn test_concurrent_access() {
    let limiter = Arc::new(RateLimiter::new(10));
    let barrier = Arc::new(Barrier::new(10));

    let mut handles = vec![];

    for _ in 0..10 {
        let limiter = Arc::clone(&limiter);
        let barrier = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier.wait().await;

            // Each task makes one request
            limiter.acquire().await.unwrap();
        });

        handles.push(handle);
    }

    // All tasks should complete successfully
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Test high concurrency scenario
#[tokio::test]
async fn test_high_concurrency() {
    let limiter = Arc::new(RateLimiter::new(100));

    let mut handles = vec![];

    for _ in 0..50 {
        let limiter = Arc::clone(&limiter);

        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                limiter.acquire().await.unwrap();
            }
        });

        handles.push(handle);
    }

    // All 250 requests should complete
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Exchange-specific rate limit tests
mod exchange_specific {
    use super::*;

    /// Test Binance.US rate limits (1200 requests/min = 20/sec)
    #[tokio::test]
    async fn test_binance_us_rate_limit() {
        let limiter = RateLimiter::new(20);

        let start = Instant::now();

        // Make 20 requests - should complete within 1 second
        for _ in 0..20 {
            limiter.acquire().await.unwrap();
        }

        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_secs(2));
    }

    /// Test OANDA rate limits (typically 120 requests/sec for streaming endpoints)
    #[tokio::test]
    async fn test_oanda_rate_limit() {
        let limiter = RateLimiter::new(120);

        let start = Instant::now();

        // Make 120 requests - should complete within ~1 second
        for _ in 0..120 {
            limiter.acquire().await.unwrap();
        }

        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_secs(2));
    }

    /// Test Coinbase rate limits (10 requests/sec for private endpoints)
    #[tokio::test]
    async fn test_coinbase_rate_limit() {
        let limiter = RateLimiter::new(10);

        let start = Instant::now();

        // Make 10 requests - should complete within 1 second
        for _ in 0..10 {
            limiter.acquire().await.unwrap();
        }

        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_secs(2));
    }
}

/// Performance tests
mod performance {
    use super::*;

    /// Test that rate limiter overhead is minimal (<1ms per call)
    #[tokio::test]
    async fn test_acquisition_overhead() {
        let limiter = RateLimiter::new(1000); // High limit to avoid throttling

        let iterations = 100;
        let start = Instant::now();

        for _ in 0..iterations {
            limiter.acquire().await.unwrap();
        }

        let elapsed = start.elapsed();
        let avg_overhead = elapsed / iterations;

        // Average overhead should be under 1ms
        assert!(
            avg_overhead < Duration::from_millis(1),
            "Overhead too high: {:?}",
            avg_overhead
        );
    }

    /// Benchmark rate limiter throughput
    #[tokio::test]
    async fn test_throughput() {
        let limiter = RateLimiter::new(1000);

        let start = Instant::now();
        let requests = 1000;

        for _ in 0..requests {
            limiter.acquire().await.unwrap();
        }

        let elapsed = start.elapsed();
        let throughput = requests as f64 / elapsed.as_secs_f64();

        // Should achieve close to configured rate
        assert!(
            throughput >= 500.0,
            "Throughput too low: {} req/sec",
            throughput
        );
    }
}

/// Integration tests
mod integration {
    use super::*;
    use exchange_connectors::{ExchangeConfig, ExchangeId};

    /// Test rate limiter with exchange config
    #[test]
    fn test_config_rate_limit() {
        let config = ExchangeConfig {
            exchange_id: ExchangeId::BinanceUs,
            api_key: "test".to_string(),
            api_secret: "test".to_string(),
            passphrase: None,
            sandbox: true,
            rate_limit_requests_per_second: 20,
            websocket_url: None,
            rest_api_url: None,
        };

        let limiter = RateLimiter::new(config.rate_limit_requests_per_second);
        assert!(std::mem::size_of_val(&limiter) > 0);
    }

    /// Test different rate limits for different exchanges
    #[tokio::test]
    async fn test_multi_exchange_rate_limits() {
        let configs = vec![
            ("binance_us", 20),
            ("coinbase", 10),
            ("oanda", 120),
        ];

        for (name, rate) in configs {
            let limiter = RateLimiter::new(rate);

            // Verify can acquire without error
            let result = limiter.acquire().await;
            assert!(result.is_ok(), "Failed for {}", name);
        }
    }
}

/// Error handling tests
mod error_handling {
    use super::*;

    /// Test rate limiter with minimum rate
    #[tokio::test]
    async fn test_minimum_rate() {
        let limiter = RateLimiter::new(1);

        let start = Instant::now();

        // Make 3 requests with 1/sec limit
        for _ in 0..3 {
            limiter.acquire().await.unwrap();
        }

        let elapsed = start.elapsed();
        // Should take at least 2 seconds
        assert!(elapsed >= Duration::from_secs(2));
    }

    /// Test rate limiter under burst conditions
    #[tokio::test]
    async fn test_burst_handling() {
        let limiter = Arc::new(RateLimiter::new(10));

        // Create burst of 50 concurrent requests
        let mut handles = vec![];

        for _ in 0..50 {
            let limiter = Arc::clone(&limiter);
            let handle = tokio::spawn(async move {
                limiter.acquire().await
            });
            handles.push(handle);
        }

        // All should eventually complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}

/// Stress tests
mod stress {
    use super::*;

    /// Stress test with many concurrent tasks
    #[tokio::test]
    async fn test_many_concurrent_tasks() {
        let limiter = Arc::new(RateLimiter::new(100));
        let task_count = 100;
        let requests_per_task = 10;

        let mut handles = vec![];

        for _ in 0..task_count {
            let limiter = Arc::clone(&limiter);
            let handle = tokio::spawn(async move {
                for _ in 0..requests_per_task {
                    limiter.acquire().await.unwrap();
                }
            });
            handles.push(handle);
        }

        // All 1000 requests should complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    /// Test sustained load
    #[tokio::test]
    async fn test_sustained_load() {
        let limiter = RateLimiter::new(50);

        let start = Instant::now();
        let duration = Duration::from_secs(2);
        let mut count = 0;

        while start.elapsed() < duration {
            limiter.acquire().await.unwrap();
            count += 1;
        }

        // Should have processed approximately 100 requests (50/sec * 2 sec)
        assert!(
            count >= 80 && count <= 150,
            "Unexpected request count: {}",
            count
        );
    }
}

/// Fairness tests
mod fairness {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Test that all tasks get fair access
    #[tokio::test]
    async fn test_fair_access() {
        let limiter = Arc::new(RateLimiter::new(100));
        let counters: Vec<Arc<AtomicUsize>> = (0..5).map(|_| Arc::new(AtomicUsize::new(0))).collect();

        let mut handles = vec![];

        for (i, counter) in counters.iter().enumerate() {
            let limiter = Arc::clone(&limiter);
            let counter = Arc::clone(counter);

            let handle = tokio::spawn(async move {
                for _ in 0..20 {
                    limiter.acquire().await.unwrap();
                    counter.fetch_add(1, Ordering::SeqCst);
                    tokio::task::yield_now().await;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Each task should have completed 20 requests
        for (i, counter) in counters.iter().enumerate() {
            let count = counter.load(Ordering::SeqCst);
            assert_eq!(count, 20, "Task {} completed {} requests", i, count);
        }
    }
}

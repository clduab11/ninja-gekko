//! Test script for verifying Kraken integration
//!
//! Usage:
//!     cargo run --example test_kraken
//!
//! This script will:
//! 1. Connect to Kraken (public endpoint check)
//! 2. Fetch trading pairs (public)
//! 3. Authenticate using env vars and fetch balances (private)
//! 4. Store/Log results

use exchange_connectors::{
    credentials::ExchangeCredentials, kraken::KrakenConnector, ExchangeConnector, ExchangeId,
};
use tracing::{error, info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting Kraken Verification...");

    // 1. Load Credentials
    dotenv::dotenv().ok();

    let api_key = std::env::var("KRAKEN_API_KEY").ok();
    let api_secret = std::env::var("KRAKEN_API_SECRET").ok();

    if let Some(k) = &api_key {
        info!(
            "Loaded KRAKEN_API_KEY: Len={} Prefix={}",
            k.len(),
            k.chars().take(4).collect::<String>()
        );
    } else {
        warn!("KRAKEN_API_KEY not found in env");
    }

    if let Some(s) = &api_secret {
        info!("Loaded KRAKEN_API_SECRET: Len={}", s.len());
    } else {
        warn!("KRAKEN_API_SECRET not found in env");
    }

    if api_key.is_none() || api_secret.is_none() {
        warn!("Missing KRAKEN_API_KEY or KRAKEN_API_SECRET. Skipping private endpoint tests.");
    }

    let creds = ExchangeCredentials::new(
        ExchangeId::Kraken,
        api_key.clone().unwrap_or_default(),
        api_secret.clone().unwrap_or_default(),
        None,
        None,
        false,
        None,
    );

    // 2. Initialize Connector
    let mut connector = KrakenConnector::new(creds);

    // 3. Connect (Public + Private Smoke Test)
    info!("Connecting...");
    // Connect might try private if keys are present, but our impl checks for empty keys
    match connector.connect().await {
        Ok(_) => info!("✅ Connectivity established"),
        Err(e) => {
            warn!(
                "⚠️ Connection/Auth check failed: {}. Proceeding to check public endpoints...",
                e
            );
            // Do not return, let's try public endpoints
        }
    }

    // 4. Fetch Trading Pairs
    info!("Fetching Trading Pairs...");
    match connector.get_trading_pairs().await {
        Ok(pairs) => {
            info!("✅ Fetched {} trading pairs", pairs.len());
            if let Some(pair) = pairs.first() {
                info!(
                    "   Example pair: {} ({}/{})",
                    pair.symbol, pair.base, pair.quote
                );
            }
        }
        Err(e) => error!("❌ Failed to fetch pairs: {}", e),
    }

    // 5. Fetch Balances (Private Auth Check)
    if api_key.is_some() && api_secret.is_some() {
        info!("Fetching Balances (Auth Check)...");
        match connector.get_balances().await {
            Ok(balances) => {
                info!("✅ Fetched {} balances", balances.len());
                for b in balances {
                    if b.total > rust_decimal::Decimal::ZERO {
                        info!("   - {}: {}", b.currency, b.total);
                    }
                }
            }
            Err(e) => error!("❌ Failed to fetch balances: {}", e),
        }
    } else {
        info!("⚠️  Skipping Balance check (no credentials)");
    }

    // 6. Ticker Check
    info!("Fetching Ticker for XBTUSD...");
    match connector.get_market_data("XBTUSD").await {
        Ok(tick) => info!("✅ Ticker: Last={} Vol={}", tick.last, tick.volume_24h),
        Err(e) => {
            // Try different symbol format if XBTUSD fails
            warn!("   Failed XBTUSD, trying XXBTZUSD...");
            match connector.get_market_data("XXBTZUSD").await {
                Ok(tick) => info!("✅ Ticker (XXBTZUSD): Last={}", tick.last),
                Err(e2) => error!("❌ Failed ticker: {}", e2),
            }
        }
    }

    info!("Kraken Verification Complete.");
    Ok(())
}

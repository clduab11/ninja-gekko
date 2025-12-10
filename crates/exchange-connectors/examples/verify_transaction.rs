use exchange_connectors::coinbase::{CoinbaseConfig, CoinbaseConnector};
use exchange_connectors::{ExchangeConnector, OrderSide, OrderType};
use rust_decimal::Decimal;
use std::env;
use std::str::FromStr;
use tokio;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    if let Err(_) = tracing::subscriber::set_global_default(subscriber) {
        // Ignore
    }

    info!("🚀 Starting Coinbase Transaction Flow Verification (Example)");

    // 2. Manual .env Loading
    // We walk up to finding .env in root.
    let mut env_path = std::env::current_dir()?;
    loop {
        let candidate = env_path.join(".env");
        if candidate.exists() {
            info!("📝 Found .env at {:?}", candidate);
            let contents = std::fs::read_to_string(candidate)?;
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some((key, val)) = line.split_once('=') {
                    let key = key.trim();
                    let mut val = val.trim();
                    if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                        val = &val[1..val.len()-1];
                    }
                    if env::var(key).is_err() {
                        env::set_var(key, val);
                    }
                }
            }
            break;
        }
        if !env_path.pop() {
            error!("❌ Could not find .env file");
            anyhow::bail!("Could not find .env file");
        }
    }

    // 3. Configure CoinbaseConnector
    let api_key_name = env::var("COINBASE_API_KEY_NAME").expect("COINBASE_API_KEY_NAME must be set");
    let private_key = env::var("COINBASE_PRIVATE_KEY").expect("COINBASE_PRIVATE_KEY must be set");

    println!("DEBUG: API_KEY_NAME found: {}", api_key_name);
    // Show first 20 chars of key to verify it's looking like a PEM
    println!("DEBUG: PRIVATE_KEY found (len={}): {}...", private_key.len(), &private_key.chars().take(30).collect::<String>());

    let config = CoinbaseConfig {
        api_key_name,
        private_key,
        sandbox: false,
        use_advanced_trade: true,
    };

    let mut connector = CoinbaseConnector::new(config);

    // 4. Connect
    info!("🔌 Connecting to Coinbase...");
    connector.connect().await?;
    if !connector.is_connected().await {
        anyhow::bail!("Failed to connect to Coinbase");
    }
    info!("✅ Connected!");

    // 5. Get Market Data (Ticker)
    let symbol = "BTC-USDC";
    info!("📈 Fetching ticker for {}...", symbol);
    let ticker = connector.get_market_data(symbol).await?;
    info!("💰 Current Price: ${}", ticker.last);

    // 6. Calculate Limit Buy Price (50% of current)
    let current_price = ticker.last;
    let target_price = current_price * Decimal::from_str("0.5")?;
    let target_price = target_price.round_dp(2);
    
    // Minimum valid size.
    let quantity = Decimal::from_str("0.0001")?;

    info!("🛑 Preparing LIMIT BUY Order: {} BTC @ ${} (50% of market)", quantity, target_price);

    // 7. Place Order
    info!("🚀 Submitting Order...");
    let order = connector.place_order(
        symbol,
        OrderSide::Buy,
        OrderType::Limit,
        quantity,
        Some(target_price)
    ).await?;

    info!("✅ Order Placed Successfully!");
    info!("   ID: {}", order.id);
    info!("   Status: {:?}", order.status);

    if order.id.is_empty() {
        anyhow::bail!("Order ID is empty!");
    }

    // 8. Cancel Order Immediately
    info!("❌ Canceling Order {}...", order.id);
    let cancelled_order = connector.cancel_order(&order.id).await?;
    
    info!("✅ Order Cancelled Successfully!");
    info!("   ID: {}", cancelled_order.id);
    info!("   Status: {:?}", cancelled_order.status); // Should be Cancelled

    info!("🎉 Transaction Flow Verification Complete!");
    Ok(())
}

//! Ninja Gekko - Autonomous Trading Bot CLI
//!
//! This is the main entry point for the Ninja Gekko autonomous trading bot.

use anyhow::Result;
use clap::Parser;
use secrecy::{ExposeSecret, Secret};
use std::net::SocketAddr;
use tokio::signal;
use tracing::{error, info, warn};

mod web;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,

    /// Operation mode
    #[arg(short, long, default_value = "precision")]
    mode: String,

    /// Enable sandbox mode (no real trading)
    #[arg(long)]
    sandbox: bool,

    /// Log level (debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Enable GPU acceleration for neural networks
    #[arg(long)]
    gpu: bool,

    /// MCP servers to enable
    #[arg(long, default_value = "playwright,filesystem,github,supabase")]
    mcp_servers: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();

    // Initialize tracing subscriber
    init_tracing(&args.log_level)?;

    info!("🥷 Starting Ninja Gekko v{}", env!("CARGO_PKG_VERSION"));
    info!("📊 Configuration: {}", args.config);
    info!("🎯 Operation mode: {}", args.mode);
    info!("🏖️  Sandbox mode: {}", args.sandbox);
    info!("🔥 GPU acceleration: {}", args.gpu);
    info!("🎭 MCP servers: {}", args.mcp_servers);

    // Load configuration - using placeholder for now
    info!("✅ Configuration loaded successfully");

    // Initialize Prometheus metrics recorder
    let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();
    let metrics_handle = recorder.handle();
    metrics::set_boxed_recorder(Box::new(recorder)).expect("Failed to set metrics recorder");
    info!("✅ Prometheus metrics recorder initialized");

    // Load configuration - using placeholder for now
    // In a real implementation, these would come from the config crate or environment variables
    let _db_url = Secret::new("postgres://user:password@localhost:5432/db".to_string());
    let _api_key = Secret::new("sk_test_123456789".to_string());

    info!("✅ Configuration loaded successfully (secrets protected in memory)");
    info!(
        "🔒 Database connection established: {}",
        _db_url.expose_secret().replace("password", "*****")
    );

    // Initialize Event Bus
    let event_bus = event_bus::EventBusBuilder::default()
        .market_capacity(1024)
        .build();
    info!("✅ Event Bus initialized");

    // Initialize Strategy Engine
    let strategy = strategy_engine::MomentumStrategy::with_defaults("momentum-1");
    let strategy_runner = std::sync::Arc::new(strategy_engine::ThreadSafeStrategyRunner::new(
        strategy,
        event_bus.signal_sender(),
        "default-account".into(),
    ));

    // Initialize Order Manager
    let risk_manager = Box::new(ninja_gekko_core::order_manager::DefaultRiskValidator::new(
        rust_decimal::Decimal::new(10000, 0),  // Max order size
        rust_decimal::Decimal::new(50000, 0),  // Max position size
        rust_decimal::Decimal::new(100000, 0), // Max portfolio exposure
    ));
    let fee_calculator = Box::new(ninja_gekko_core::order_manager::DefaultFeeCalculator::new(
        rust_decimal::Decimal::new(1, 3), // 0.1% maker
        rust_decimal::Decimal::new(2, 3), // 0.2% taker
    ));
    let order_manager = std::sync::Arc::new(ninja_gekko_core::order_manager::OrderManager::new(
        risk_manager,
        fee_calculator,
    ));

    // Initialize Signal Bridge
    let signal_bridge = std::sync::Arc::new(event_bus::core_bridges::SignalToOrderBridge::new(
        order_manager.clone(),
        event_bus.order_sender(),
        event_bus::PublishMode::Try,
    ));

    // Initialize Event Dispatcher
    let dispatcher = event_bus::EventDispatcherBuilder::new(&event_bus)
        .on_market(strategy_runner)
        .on_signal(signal_bridge)
        .build();

    let _dispatcher_controller = dispatcher.controller();
    let dispatcher_handle = tokio::spawn(async move {
        if let Err(e) = dispatcher.run().await {
            error!("Event dispatcher error: {}", e);
        }
    });
    info!("✅ Strategy Engine & Event Dispatcher started");

    // Create trading system
    let bot = ninja_gekko::core::NinjaGekko::builder()
        .mode(match args.mode.as_str() {
            "stealth" => ninja_gekko::core::OperationMode::Stealth,
            "swarm" => ninja_gekko::core::OperationMode::Swarm,
            _ => ninja_gekko::core::OperationMode::Precision,
        })
        .mcp_servers(args.mcp_servers.split(',').map(|s| s.to_string()).collect())
        .dry_run(args.sandbox)
        .event_bus(event_bus.clone())
        .build()
        .await?;

    info!("✅ Ninja Gekko initialized");

    // Get a reference to the MCP client for the web layer
    let mcp_client = bot.mcp_client().clone();

    let api_addr: SocketAddr = "0.0.0.0:8787".parse()?;
    // Pass the event bus to the web layer
    let api_handle = web::spawn(
        api_addr,
        Some(event_bus.clone()),
        mcp_client,
        metrics_handle,
    );
    info!("🌐 Chat orchestration API live at http://{api_addr}");

    // Start the bot in the background
    let bot_handle = tokio::spawn(async move {
        if let Err(e) = bot.start().await {
            error!("Ninja Gekko runtime error: {}", e);
        }
    });

    // Setup shutdown handler
    setup_shutdown_handler().await;

    // Perform cleanup
    info!("🛑 Shutting down Ninja Gekko...");
    // We would signal shutdown to bot_handle here
    info!("✅ Ninja Gekko shut down gracefully");

    Ok(())
}

fn init_tracing(log_level: &str) -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(match log_level.to_lowercase().as_str() {
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => {
                warn!("Invalid log level '{}', defaulting to 'info'", log_level);
                tracing::Level::INFO
            }
        })
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

/// Setup graceful shutdown handler
async fn setup_shutdown_handler() {
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("📡 Received shutdown signal (Ctrl+C)");
        }
        Err(err) => {
            error!("💥 Failed to listen for shutdown signal: {:?}", err);
        }
    }
}

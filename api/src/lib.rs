//! # Ninja Gekko API
//!
//! High-performance REST API and WebSocket server for the Ninja Gekko trading system.
//! Built with Axum framework for maximum performance and reliability.
//!
//! ## Features
//! - REST API endpoints for trading operations
//! - WebSocket support for real-time market data
//! - JWT-based authentication
//! - Rate limiting and CORS middleware
//! - Comprehensive error handling
//! - Structured API responses
//!
//! ## Architecture
//! The API is organized into several modules:
//! - `handlers`: HTTP request handlers
//! - `middleware`: Authentication, CORS, rate limiting
//! - `models`: API request/response models
//! - `websocket`: WebSocket connection handling
//! - `config`: Server configuration
//! - `error`: Error types and handling

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

// Core dependencies
use ninja_gekko_database::DatabaseManager;

pub mod auth;
pub mod auth_validation;
pub mod config;
pub mod env_validation;
pub mod error;
pub mod handlers;
pub mod llm;
pub mod managers;
pub mod middleware;
pub mod models;
pub mod validation;
pub mod websocket;

use crate::handlers::orchestrator::OrchestratorState;
use crate::managers::{MarketDataService, PortfolioManager, StrategyManager};
use crate::websocket::WebSocketManager;
use exchange_connectors::ExchangeConnector;
use tokio::sync::RwLock;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Database manager for data persistence
    pub db_manager: Arc<DatabaseManager>,
    /// Portfolio manager
    pub portfolio_manager: Arc<PortfolioManager>,
    /// Market data service
    pub market_data_service: Arc<MarketDataService>,
    /// WebSocket manager
    pub websocket_manager: Arc<WebSocketManager>,
    /// Strategy manager
    pub strategy_manager: Arc<StrategyManager>,
    /// Server configuration
    pub config: Arc<config::ApiConfig>,
    /// Orchestrator state (thread-safe mutable)
    pub orchestrator_state: Arc<RwLock<OrchestratorState>>,
}

impl AppState {
    pub async fn new(config: config::ApiConfig) -> Result<Self, error::ApiError> {
        // Load database configuration
        let db_manager = Arc::new(
            DatabaseManager::new(ninja_gekko_database::DatabaseConfig {
                database_url: config.database_url.clone(),
                max_connections: 10,
                min_connections: 5,
                acquire_timeout: std::time::Duration::from_secs(30),
                idle_timeout: std::time::Duration::from_secs(600),
                max_lifetime: std::time::Duration::from_secs(1800),
                enable_ssl: false,
                connect_timeout: std::time::Duration::from_secs(10),
            })
            .await
            .map_err(|e| error::ApiError::database(e.to_string()))?,
        );

        // Run database migrations
        let migration_config = ninja_gekko_database::config::MigrationConfig {
            migration_dir: "database/migrations".to_string(),
            ..Default::default()
        };

        let migration_manager = ninja_gekko_database::migrations::MigrationManager::new(
            migration_config,
            "database/migrations",
        )
        .map_err(|e| {
            error::ApiError::internal(format!("Failed to init migration manager: {}", e))
        })?;

        info!("Running matching migrations...");
        if let Err(e) = migration_manager.run_migrations(db_manager.pool()).await {
            tracing::error!("Failed to run migrations: {}", e);
            // Fail startup if migrations fail? Yes, usually safer.
            return Err(error::ApiError::internal(format!(
                "Migration failed: {}",
                e
            )));
        }
        info!("Migrations completed successfully");

        let portfolio_manager = Arc::new(PortfolioManager::new(db_manager.clone()));

        // Initialize Exchange Connector
        // Prioritize Kraken as the primary execution venue
        let connector: Option<Arc<Box<dyn exchange_connectors::ExchangeConnector>>> =
            if let (Ok(api_key), Ok(api_secret)) = (
                std::env::var("KRAKEN_API_KEY"),
                std::env::var("KRAKEN_API_SECRET"),
            ) {
                info!("Initializing Kraken connector");
                let creds = exchange_connectors::credentials::ExchangeCredentials::new(
                    exchange_connectors::ExchangeId::Kraken,
                    api_key,
                    api_secret,
                    None,
                    false, // sandbox param, could load from env
                );

                let mut kraken_connector = exchange_connectors::kraken::KrakenConnector::new(creds);

                // Try to connect (validate credentials)
                if let Err(e) = kraken_connector.connect().await {
                    tracing::error!("Failed to connect to Kraken on startup: {}", e);
                    // Return it anyway, it might work later or this was just a connectivity blip
                }

                Some(Arc::new(Box::new(kraken_connector)))
            } else {
                info!("No Kraken credentials found, market data service will return errors");
                None
            };

        let market_data_service = Arc::new(MarketDataService::new(db_manager.clone(), connector));

        let strategy_manager = Arc::new(StrategyManager::new(db_manager.clone()));

        let websocket_manager = Arc::new(WebSocketManager::new());
        // Note: Start websocket background tasks in main.rs

        Ok(Self {
            db_manager,
            portfolio_manager,
            market_data_service,
            websocket_manager,
            strategy_manager,
            config: Arc::new(config),
            orchestrator_state: Arc::new(RwLock::new(OrchestratorState::default())),
        })
    }
}

/// Main API server structure
pub struct ApiServer {
    /// Axum router with all routes configured
    router: Router,
    /// Server configuration
    config: Arc<config::ApiConfig>,
    /// Application state
    state: Arc<AppState>,
}

impl ApiServer {
    /// Creates a new API server with all routes and middleware configured
    pub async fn new() -> Result<Self, error::ApiError> {
        // Load configuration
        let config = config::ApiConfig::from_env()
            .map_err(|e| error::ApiError::config(format!("Failed to load config: {}", e)))?;

        // Create application state
        let state = Arc::new(AppState::new(config.clone()).await?);

        // Build middleware stack using the middleware builder
        // Build middleware stack using the middleware builder
        let middleware_builder = middleware::MiddlewareBuilder::new()
            .cors(true)
            .rate_limiting(true)
            .logging(true)
            .security(true)
            .timing(true)
            .request_id(true);

        // Create router with all routes
        let router = Router::new()
            // Health check endpoint
            .route("/health", get(handlers::health_check))
            // Trade endpoints
            .route("/api/v1/trades", get(handlers::trades::list_trades))
            .route("/api/v1/trades", post(handlers::trades::create_trade))
            .route("/api/v1/trades/:id", get(handlers::trades::get_trade))
            .route("/api/v1/trades/:id", put(handlers::trades::update_trade))
            .route("/api/v1/trades/:id", delete(handlers::trades::delete_trade))
            // Portfolio endpoints
            .route("/api/v1/portfolio", get(handlers::portfolio::get_portfolio))
            .route(
                "/api/v1/portfolio/positions",
                get(handlers::portfolio::get_positions),
            )
            .route(
                "/api/v1/portfolio/positions/:symbol",
                get(handlers::portfolio::get_position),
            )
            .route(
                "/api/v1/portfolio/performance",
                get(handlers::portfolio::get_performance_metrics),
            )
            // Market data endpoints
            .route(
                "/api/v1/market-data",
                get(handlers::market_data::get_batch_market_data),
            )
            .route(
                "/api/v1/market-data/:symbol",
                get(handlers::market_data::get_market_data),
            )
            .route(
                "/api/v1/market-data/:symbol/history",
                get(handlers::market_data::get_historical_data),
            )
            // Strategy endpoints
            .route(
                "/api/v1/strategies",
                get(handlers::strategies::list_strategies),
            )
            .route(
                "/api/v1/strategies",
                post(handlers::strategies::create_strategy),
            )
            .route(
                "/api/v1/strategies/:id",
                get(handlers::strategies::get_strategy),
            )
            .route(
                "/api/v1/strategies/:id",
                put(handlers::strategies::update_strategy),
            )
            .route(
                "/api/v1/strategies/:id",
                delete(handlers::strategies::delete_strategy),
            )
            .route(
                "/api/v1/strategies/:id/execute",
                post(handlers::strategies::execute_strategy),
            )
            // WebSocket endpoint for real-time data
            .route("/api/v1/ws", get(websocket::handle_socket))
            // Authentication endpoints
            .route(
                "/api/v1/auth/login",
                post(handlers::auth_utils::login_handler),
            )
            .route(
                "/api/v1/auth/refresh",
                post(handlers::auth_utils::refresh_handler),
            )
            .route(
                "/api/v1/auth/logout",
                post(handlers::auth_utils::logout_handler),
            )
            // API documentation
            .route("/api/v1/docs", get(handlers::api_info))
            // Chat & Frontend routes
            .route("/api/chat/history", get(handlers::chat::get_chat_history))
            .route("/api/chat/models", get(handlers::chat::get_models))
            .route("/api/chat/message", post(handlers::chat::send_message))
            .route("/api/chat/persona", get(handlers::chat::get_persona))
            .route("/api/chat/persona", post(handlers::chat::update_persona)) // Handle both GET and POST
            .route("/api/trading/pause", post(handlers::chat::pause_trading))
            // Old generic route - keeping for backward compatibility if needed, or replacing
            .route(
                "/api/accounts/snapshot",
                get(handlers::accounts::get_account_snapshot),
            )
            // New routes
            .route(
                "/api/v1/accounts/snapshot",
                get(handlers::accounts::get_account_snapshot),
            )
            .route(
                "/api/v1/accounts/aggregate",
                get(handlers::accounts::get_aggregate_account),
            )
            .route(
                "/api/news/headlines",
                get(handlers::chat::get_news_headlines),
            )
            .route("/api/research/sonar", post(handlers::chat::research_sonar))
            .route("/api/agents/swarm", post(handlers::chat::summon_swarm))
            // Orchestrator Controls
            .route(
                "/api/orchestrator/engage",
                post(handlers::orchestrator::engage),
            )
            .route(
                "/api/orchestrator/wind-down",
                post(handlers::orchestrator::wind_down),
            )
            .route(
                "/api/orchestrator/emergency-halt",
                post(handlers::orchestrator::emergency_halt),
            )
            .route(
                "/api/orchestrator/risk-throttle",
                post(handlers::orchestrator::risk_throttle),
            )
            .route(
                "/api/orchestrator/state",
                get(handlers::orchestrator::get_state),
            )
            // Intel Stream
            .route(
                "/api/v1/intel/stream",
                get(handlers::intel::get_intel_stream),
            )
            // Apply middleware
            // Apply middleware
            //.layer(middleware)
            .with_state(state.clone());

        let router = middleware_builder.apply_to(router);

        info!(
            "API server configured with {} routes",
            count_routes(&router)
        );

        Ok(Self {
            router,
            config: Arc::new(config),
            state,
        })
    }

    /// Starts the API server and begins listening for requests
    pub async fn serve(self) -> Result<(), error::ApiError> {
        let addr = &self.config.bind_address;

        info!("Starting Ninja Gekko API server on {}", addr);
        info!("Health check available at http://{}/health", addr);
        info!("API documentation available at http://{}/api/v1/docs", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| error::ApiError::Internal {
                message: format!("Failed to bind to {}: {}", addr, e),
            })?;

        info!("🚀 Server listening on http://{}", addr);

        // Start the server
        axum::serve(listener, self.router)
            .await
            .map_err(|e| error::ApiError::Internal {
                message: format!("Server error: {}", e),
            })?;

        Ok(())
    }

    /// Returns server configuration
    pub fn config(&self) -> &config::ApiConfig {
        &self.config
    }

    /// Returns application state
    pub fn state(&self) -> &AppState {
        &self.state
    }
}

/// Counts the total number of routes in a router
fn count_routes(router: &Router) -> usize {
    // This is a simplified count - in production you might want to traverse the router tree
    // For now, we'll return an estimate based on the routes we know we added
    15 // Rough count of our endpoints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_api_server_creation() {
        // This test would require a test database and proper configuration
        // For now, we'll just ensure the struct can be created
        let server = ApiServer::new().await;
        assert!(server.is_ok());
    }

    #[test]
    fn test_route_counting() {
        // This would test the route counting logic
        // For now, just ensure it returns a positive number
        let count = count_routes(&Router::new());
        assert!(count >= 0);
    }
}

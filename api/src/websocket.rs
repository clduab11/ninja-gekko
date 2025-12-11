//! WebSocket support for real-time market data updates
//!
//! This module provides WebSocket functionality for real-time streaming of market data,
//! trade updates, portfolio changes, and strategy execution events. It supports
//! dynamic subscription management and efficient broadcasting to multiple clients.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{broadcast, RwLock},
    time::interval,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    error::ApiResult,
    models::{TradeResponse, PortfolioResponse, StrategyExecutionResponse, MarketDataResponse},
    AppState,
};

/// WebSocket connection manager
#[derive(Debug, Clone)]
pub struct WebSocketManager {
    /// Broadcast sender for market data updates
    market_data_tx: broadcast::Sender<MarketDataMessage>,
    /// Broadcast sender for trade updates
    trade_updates_tx: broadcast::Sender<TradeUpdateMessage>,
    /// Broadcast sender for portfolio updates
    portfolio_updates_tx: broadcast::Sender<PortfolioUpdateMessage>,
    /// Broadcast sender for strategy execution updates
    strategy_updates_tx: broadcast::Sender<StrategyUpdateMessage>,
    /// Broadcast sender for intel stream updates
    intel_updates_tx: broadcast::Sender<IntelUpdateMessage>,
    /// Active connections with their subscriptions
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    /// Market data stream for periodic updates
    market_data_stream: Arc<RwLock<MarketDataStream>>,
}

/// Connection information for each WebSocket client
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Unique connection ID
    pub connection_id: String,
    /// Client address
    pub client_addr: String,
    /// Connected timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Active subscriptions
    pub subscriptions: Vec<SubscriptionType>,
}

/// Types of subscriptions a client can have
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriptionType {
    /// Market data for specific symbols
    MarketData(Vec<String>),
    /// Trade updates for specific accounts
    TradeUpdates(Vec<String>),
    /// Portfolio updates for specific accounts
    PortfolioUpdates(Vec<String>),
    /// Strategy execution updates for specific strategies
    StrategyUpdates(Vec<String>),
    /// All market data updates
    AllMarketData,
    /// All trade updates
    AllTrades,
    /// All portfolio updates
    AllPortfolios,
    /// All strategy updates
    AllStrategies,
    /// Intel stream updates
    IntelStream,
}

/// WebSocket messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    /// Market data update
    MarketData {
        data: MarketDataResponse,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Trade update
    TradeUpdate {
        trade: TradeResponse,
        action: String, // "created", "updated", "deleted"
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Portfolio update
    PortfolioUpdate {
        portfolio: PortfolioResponse,
        account_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Strategy execution update
    StrategyUpdate {
        execution: StrategyExecutionResponse,
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Intel stream update
    IntelUpdate {
        item: crate::handlers::intel::IntelItem,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Subscription confirmation
    SubscriptionConfirmed {
        subscription_type: SubscriptionType,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Error message
    Error {
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Heartbeat/Ping
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Client messages sent to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to updates
    Subscribe {
        subscriptions: Vec<SubscriptionType>,
    },
    /// Unsubscribe from updates
    Unsubscribe {
        subscriptions: Vec<SubscriptionType>,
    },
    /// Ping/Pong for connection health
    Ping,
    /// Request current state snapshot
    GetSnapshot {
        data_types: Vec<String>, // "market_data", "portfolio", "positions"
    },
}

/// Internal message types for broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataMessage {
    pub data: MarketDataResponse,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeUpdateMessage {
    pub trade: TradeResponse,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioUpdateMessage {
    pub portfolio: PortfolioResponse,
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyUpdateMessage {
    pub execution: StrategyExecutionResponse,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelUpdateMessage {
    pub item: crate::handlers::intel::IntelItem,
}

/// Market data stream for periodic updates
#[derive(Debug)]
pub struct MarketDataStream {
    symbols: Vec<String>,
    update_interval: Duration,
    is_running: bool,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let (market_data_tx, _) = broadcast::channel(1000);
        let (trade_updates_tx, _) = broadcast::channel(1000);
        let (portfolio_updates_tx, _) = broadcast::channel(1000);
        let (strategy_updates_tx, _) = broadcast::channel(1000);
        let (intel_updates_tx, _) = broadcast::channel(1000);

        let market_data_stream = Arc::new(RwLock::new(MarketDataStream {
            symbols: Vec::new(),
            update_interval: Duration::from_secs(1),
            is_running: false,
        }));

        Self {
            market_data_tx,
            trade_updates_tx,
            portfolio_updates_tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
            market_data_stream,
            strategy_updates_tx,
            intel_updates_tx,
        }
    }

    /// Start the WebSocket manager with background tasks
    pub async fn start(&self, app_state: Arc<AppState>) -> ApiResult<()> {
        info!("Starting WebSocket manager");

        // Start market data streaming task
        self.start_market_data_streaming(app_state.clone()).await;

        // Start periodic cleanup task
        self.start_connection_cleanup().await;

        Ok(())
    }

    /// Get the number of active connections
    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        let mut stats = ConnectionStats::default();

        for conn_info in connections.values() {
            stats.total_connections += 1;
            stats.total_subscriptions += conn_info.subscriptions.len();

            // Count subscription types
            for sub in &conn_info.subscriptions {
                match sub {
                    SubscriptionType::MarketData(symbols) => {
                        stats.market_data_subscriptions += symbols.len();
                    }
                    SubscriptionType::TradeUpdates(accounts) => {
                        stats.trade_update_subscriptions += accounts.len();
                    }
                    SubscriptionType::PortfolioUpdates(accounts) => {
                        stats.portfolio_update_subscriptions += accounts.len();
                    }
                    SubscriptionType::StrategyUpdates(strategies) => {
                        stats.strategy_update_subscriptions += strategies.len();
                    }
                    _ => stats.all_type_subscriptions += 1,
                }
            }
        }

        if stats.total_connections > 0 {
            stats.average_subscriptions_per_connection =
                stats.total_subscriptions as f64 / stats.total_connections as f64;
        }

        stats
    }

    /// Broadcast market data to all subscribed clients
    pub async fn broadcast_market_data(&self, data: MarketDataResponse) -> ApiResult<()> {
        let message = MarketDataMessage {
            data: data.clone(),
            symbol: data.symbol.clone(),
        };

        if let Err(e) = self.market_data_tx.send(message) {
            warn!("Failed to broadcast market data: {}", e);
        }

        Ok(())
    }

    /// Broadcast trade update to all subscribed clients
    pub async fn broadcast_trade_update(&self, trade: TradeResponse, action: &str) -> ApiResult<()> {
        let message = TradeUpdateMessage {
            trade,
            action: action.to_string(),
        };

        if let Err(e) = self.trade_updates_tx.send(message) {
            warn!("Failed to broadcast trade update: {}", e);
        }

        Ok(())
    }

    /// Broadcast portfolio update to all subscribed clients
    pub async fn broadcast_portfolio_update(&self, portfolio: PortfolioResponse, account_id: &str) -> ApiResult<()> {
        let message = PortfolioUpdateMessage {
            portfolio,
            account_id: account_id.to_string(),
        };

        if let Err(e) = self.portfolio_updates_tx.send(message) {
            warn!("Failed to broadcast portfolio update: {}", e);
        }

        Ok(())
    }

    /// Broadcast strategy update to all subscribed clients
    pub async fn broadcast_strategy_update(&self, execution: StrategyExecutionResponse, status: &str) -> ApiResult<()> {
        let message = StrategyUpdateMessage {
            execution,
            status: status.to_string(),
        };

        if let Err(e) = self.strategy_updates_tx.send(message) {
            warn!("Failed to broadcast strategy update: {}", e);
        }

        Ok(())
    }

    /// Broadcast intel stream update to all subscribed clients
    pub async fn broadcast_intel_update(&self, item: crate::handlers::intel::IntelItem) -> ApiResult<()> {
        let message = IntelUpdateMessage {
            item,
        };

        if let Err(e) = self.intel_updates_tx.send(message) {
            warn!("Failed to broadcast intel update: {}", e);
        }

        Ok(())
    }

    /// Start background market data streaming
    async fn start_market_data_streaming(&self, app_state: Arc<AppState>) {
        let market_data_tx = self.market_data_tx.clone();
        let symbols = Arc::new(RwLock::new(Vec::<String>::new()));

        // Start periodic update task
        let symbols_read = symbols.clone();
        tokio::spawn(async move {
            let symbols_inner = symbols_read;
            let mut interval = interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                // Get current symbols to stream
                let current_symbols = symbols_inner.read().await.clone();

                if current_symbols.is_empty() {
                    continue;
                }

                // Fetch latest market data for each symbol
                for symbol in &current_symbols {
                    match app_state.market_data_service.get_latest_data(symbol).await {
                        Ok(data) => {
                            let message = MarketDataMessage {
                                data: data.clone(),
                                symbol: symbol.clone(),
                            };

                            if let Err(e) = market_data_tx.send(message) {
                                warn!("Failed to send market data for {}: {}", symbol, e);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to fetch market data for {}: {}", symbol, e);
                        }
                    }
                }
            }
        });

        // Update symbols list periodically
        let symbols_clone = symbols.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                // match app_state.market_data_service.get_available_symbols().await {
                //     Ok(available_symbols) => {
                //         let mut symbols = symbols_clone.write().await;
                //         *symbols = available_symbols;
                //     }
                //     Err(e) => {
                //         warn!("Failed to fetch available symbols: {}", e);
                //     }
                // }
                // Mock implementation until method is available
                let mut symbols = symbols_clone.write().await;
                if symbols.is_empty() {
                    *symbols = vec!["BTC-USD".to_string(), "ETH-USD".to_string()];
                }
            }
        });
    }

    /// Start connection cleanup task
    async fn start_connection_cleanup(&self) {
        let connections = self.connections.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5 * 60));

            loop {
                interval.tick().await;

                let mut connections = connections.write().await;
                let now = chrono::Utc::now();
                let mut to_remove = Vec::new();

                for (connection_id, conn_info) in connections.iter() {
                    // Check if connection has been inactive for too long
                    // In a real implementation, you'd track last activity
                    let connection_age = now - conn_info.connected_at;

                    if connection_age > chrono::Duration::hours(24) {
                        to_remove.push(connection_id.clone());
                    }
                }

                for connection_id in to_remove {
                    debug!("Removing stale connection: {}", connection_id);
                    connections.remove(&connection_id);
                }

                info!("Connection cleanup completed. Active connections: {}", connections.len());
            }
        });
    }

    /// Handle WebSocket upgrade request
    pub async fn handle_websocket(
        ws: WebSocketUpgrade,
        State(state): State<Arc<AppState>>,
        State(ws_manager): State<Arc<WebSocketManager>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(|socket| process_socket(socket, state, ws_manager))
    }

    /// Get WebSocket test page
    pub async fn websocket_test_page() -> Html<&'static str> {
        Html(include_str!("../static/websocket_test.html"))
    }
}

/// WebSocket handler endpoint
pub async fn handle_socket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let ws_manager = state.websocket_manager.clone();
    ws.on_upgrade(move |socket| process_socket(socket, state, ws_manager))
}

/// Process WebSocket connection
/// Process WebSocket connection
async fn process_socket(
    socket: WebSocket,
    app_state: Arc<AppState>,
    ws_manager: Arc<WebSocketManager>,
) {
    let connection_id = Uuid::new_v4().to_string();
    let client_addr = "unknown".to_string(); // In real impl, get from request

    info!("New WebSocket connection: {}", connection_id);

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    let mut subscriptions = Vec::new();

    // Subscribe to relevant broadcast channels
    let mut market_data_rx = ws_manager.market_data_tx.subscribe();
    let mut trade_updates_rx = ws_manager.trade_updates_tx.subscribe();
    let mut portfolio_updates_rx = ws_manager.portfolio_updates_tx.subscribe();
    let mut strategy_updates_rx = ws_manager.strategy_updates_tx.subscribe();
    let mut intel_updates_rx = ws_manager.intel_updates_tx.subscribe();

    // Store connection info
    let connection_info = ConnectionInfo {
        connection_id: connection_id.clone(),
        client_addr: client_addr.clone(),
        connected_at: chrono::Utc::now(),
        subscriptions: subscriptions.clone(),
    };

    ws_manager.connections.write().await.insert(connection_id.clone(), connection_info);

    // Send welcome message
    let welcome_message = WebSocketMessage::Ping {
        timestamp: chrono::Utc::now(),
    };

    if let Ok(message_text) = serde_json::to_string(&welcome_message) {
        if let Err(e) = sender.send(Message::Text(message_text)).await {
            error!("Failed to send welcome message: {}", e);
            return;
        }
    }

    // Handle incoming messages and broadcasts
    loop {
        tokio::select! {
            // Handle client messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_client_message(
                            &text,
                            &mut subscriptions,
                            &mut sender,
                            &ws_manager,
                        ).await {
                            error!("Error handling client message: {}", e);
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        debug!("WebSocket connection closed: {}", connection_id);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // Handle market data broadcasts
            msg = market_data_rx.recv() => {
                match msg {
                    Ok(market_data_msg) => {
                        if should_send_to_client(&SubscriptionType::MarketData(vec![market_data_msg.symbol.clone()]), &subscriptions) {
                            let ws_message = WebSocketMessage::MarketData {
                                data: market_data_msg.data,
                                timestamp: chrono::Utc::now(),
                            };

                            if let Ok(message_text) = serde_json::to_string(&ws_message) {
                                if let Err(e) = sender.send(Message::Text(message_text)).await {
                                    debug!("Failed to send market data, client disconnected");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Market data broadcast error: {}", e);
                    }
                }
            }

            // Handle trade update broadcasts
            msg = trade_updates_rx.recv() => {
                match msg {
                    Ok(trade_msg) => {
                        if should_send_to_client(&SubscriptionType::AllTrades, &subscriptions) {
                            let ws_message = WebSocketMessage::TradeUpdate {
                                trade: trade_msg.trade,
                                action: trade_msg.action,
                                timestamp: chrono::Utc::now(),
                            };

                            if let Ok(message_text) = serde_json::to_string(&ws_message) {
                                if let Err(e) = sender.send(Message::Text(message_text)).await {
                                    debug!("Failed to send trade update, client disconnected");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Trade update broadcast error: {}", e);
                    }
                }
            }

            // Handle portfolio update broadcasts
            msg = portfolio_updates_rx.recv() => {
                match msg {
                    Ok(portfolio_msg) => {
                        if should_send_to_client(&SubscriptionType::AllPortfolios, &subscriptions) {
                            let ws_message = WebSocketMessage::PortfolioUpdate {
                                portfolio: portfolio_msg.portfolio,
                                account_id: portfolio_msg.account_id,
                                timestamp: chrono::Utc::now(),
                            };

                            if let Ok(message_text) = serde_json::to_string(&ws_message) {
                                if let Err(e) = sender.send(Message::Text(message_text)).await {
                                    debug!("Failed to send portfolio update, client disconnected");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Portfolio update broadcast error: {}", e);
                    }
                }
            }

            // Handle strategy update broadcasts
            msg = strategy_updates_rx.recv() => {
                match msg {
                    Ok(strategy_msg) => {
                        if should_send_to_client(&SubscriptionType::AllStrategies, &subscriptions) {
                            let ws_message = WebSocketMessage::StrategyUpdate {
                                execution: strategy_msg.execution,
                                status: strategy_msg.status,
                                timestamp: chrono::Utc::now(),
                            };

                            if let Ok(message_text) = serde_json::to_string(&ws_message) {
                                if let Err(e) = sender.send(Message::Text(message_text)).await {
                                    debug!("Failed to send strategy update, client disconnected");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Strategy update broadcast error: {}", e);
                    }
                }
            }

            // Handle intel update broadcasts
            msg = intel_updates_rx.recv() => {
                match msg {
                    Ok(intel_msg) => {
                        if should_send_to_client(&SubscriptionType::IntelStream, &subscriptions) {
                            let ws_message = WebSocketMessage::IntelUpdate {
                                item: intel_msg.item,
                                timestamp: chrono::Utc::now(),
                            };

                            if let Ok(message_text) = serde_json::to_string(&ws_message) {
                                if let Err(e) = sender.send(Message::Text(message_text)).await {
                                    debug!("Failed to send intel update, client disconnected");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Intel update broadcast error: {}", e);
                    }
                }
            }
        }
    }

    // Clean up connection
    ws_manager.connections.write().await.remove(&connection_id);
    info!("WebSocket connection closed: {}", connection_id);
}

/// Handle client message
async fn handle_client_message(
    text: &str,
    subscriptions: &mut Vec<SubscriptionType>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ws_manager: &WebSocketManager,
) -> ApiResult<()> {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::Subscribe { subscriptions: new_subscriptions }) => {
            // Add new subscriptions
            for sub in new_subscriptions {
                if !subscriptions.contains(&sub) {
                    subscriptions.push(sub.clone());

                    // Send confirmation
                    let confirmation = WebSocketMessage::SubscriptionConfirmed {
                        subscription_type: sub,
                        timestamp: chrono::Utc::now(),
                    };

                    let message_text = serde_json::to_string(&confirmation)?;
                    sender.send(Message::Text(message_text)).await?;
                }
            }
        }

        Ok(ClientMessage::Unsubscribe { subscriptions: subscriptions_to_remove }) => {
            // Remove subscriptions
            subscriptions.retain(|sub| !subscriptions_to_remove.contains(sub));
        }

        Ok(ClientMessage::Ping) => {
            let pong = WebSocketMessage::Ping {
                timestamp: chrono::Utc::now(),
            };

            let message_text = serde_json::to_string(&pong)?;
            sender.send(Message::Text(message_text)).await?;
        }

        Ok(ClientMessage::GetSnapshot { data_types }) => {
            // Handle snapshot requests - simplified for this example
            for data_type in data_types {
                match data_type.as_str() {
                    "market_data" => {
                        // Send current market data snapshot
                        debug!("Sending market data snapshot");
                    }
                    "portfolio" => {
                        // Send current portfolio snapshot
                        debug!("Sending portfolio snapshot");
                    }
                    "positions" => {
                        // Send current positions snapshot
                        debug!("Sending positions snapshot");
                    }
                    _ => {
                        let error_msg = WebSocketMessage::Error {
                            message: format!("Unknown data type: {}", data_type),
                            timestamp: chrono::Utc::now(),
                        };

                        let message_text = serde_json::to_string(&error_msg)?;
                        sender.send(Message::Text(message_text)).await?;
                    }
                }
            }
        }

        Err(e) => {
            let error_msg = WebSocketMessage::Error {
                message: format!("Invalid message format: {}", e),
                timestamp: chrono::Utc::now(),
            };

            let message_text = serde_json::to_string(&error_msg)?;
            sender.send(Message::Text(message_text)).await?;
        }
    }

    Ok(())
}

/// Check if message should be sent to client based on subscriptions
fn should_send_to_client(subscription_type: &SubscriptionType, client_subscriptions: &[SubscriptionType]) -> bool {
    for client_sub in client_subscriptions {
        match (client_sub, subscription_type) {
            (SubscriptionType::AllMarketData, SubscriptionType::MarketData(_)) => return true,
            (SubscriptionType::AllTrades, SubscriptionType::TradeUpdates(_)) => return true,
            (SubscriptionType::AllPortfolios, SubscriptionType::PortfolioUpdates(_)) => return true,
            (SubscriptionType::AllStrategies, SubscriptionType::StrategyUpdates(_)) => return true,
            (SubscriptionType::IntelStream, SubscriptionType::IntelStream) => return true,
            (SubscriptionType::MarketData(client_symbols), SubscriptionType::MarketData(msg_symbols)) => {
                // Check if there's any symbol overlap
                for msg_symbol in msg_symbols {
                    if client_symbols.contains(&msg_symbol) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Connection statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub total_subscriptions: usize,
    pub average_subscriptions_per_connection: f64,
    pub market_data_subscriptions: usize,
    pub trade_update_subscriptions: usize,
    pub portfolio_update_subscriptions: usize,
    pub strategy_update_subscriptions: usize,
    pub all_type_subscriptions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        assert_eq!(manager.get_connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_subscription_matching() {
        let client_subs = vec![
            SubscriptionType::MarketData(vec!["AAPL".to_string(), "GOOGL".to_string()]),
            SubscriptionType::AllTrades,
        ];

        assert!(should_send_to_client(
            &SubscriptionType::MarketData(vec!["AAPL".to_string()]),
            &client_subs
        ));

        assert!(should_send_to_client(
            &SubscriptionType::TradeUpdates(vec!["account1".to_string()]),
            &client_subs
        ));

        assert!(!should_send_to_client(
            &SubscriptionType::MarketData(vec!["MSFT".to_string()]),
            &client_subs
        ));
    }
}
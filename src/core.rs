//! Core Ninja Gekko system implementation

use crate::neural::NeuralBackend;
use event_bus::EventBus;
use mcp_client::{McpClient, McpConfig};
use exchange_connectors::ExchangeConnector;
use std::fmt;

/// Main Ninja Gekko bot struct
#[derive(Clone)]
pub struct NinjaGekko {
    /// Operation mode
    pub mode: OperationMode,
    /// Neural network backend
    pub neural_backend: NeuralBackend,
    /// MCP client for protocol integrations
    pub mcp_client: McpClient,
    /// Event bus for inter-component communication
    pub event_bus: Option<EventBus>,
    /// Dry run flag
    pub dry_run: bool,
}

/// Operation modes for the trading bot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    /// Stealth mode - minimal market impact
    Stealth,
    /// Precision mode - microsecond timing
    Precision,
    /// Swarm mode - distributed intelligence
    Swarm,
}

impl fmt::Display for OperationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationMode::Stealth => write!(f, "Stealth"),
            OperationMode::Precision => write!(f, "Precision"),
            OperationMode::Swarm => write!(f, "Swarm"),
        }
    }
}

/// Builder for NinjaGekko
pub struct NinjaGekkoBuilder {
    mode: OperationMode,
    neural_backend: NeuralBackend,
    mcp_servers: Vec<String>,
    dry_run: bool,
    event_bus: Option<EventBus>,
}

impl NinjaGekko {
    /// Create a new builder
    pub fn builder() -> NinjaGekkoBuilder {
        NinjaGekkoBuilder {
            mode: OperationMode::Precision,
            neural_backend: NeuralBackend::RuvFann,
            mcp_servers: vec![],
            dry_run: false,
            event_bus: None,
        }
    }

    /// Start the bot
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🥷 Starting Ninja Gekko in {:?} mode", self.mode);

        // Start MCP client
        if let Err(e) = self.mcp_client.start().await {
            tracing::error!("Failed to start MCP client: {}", e);
        }

        // Initialize components based on mode
        match self.mode {
            OperationMode::Stealth => self.start_stealth_mode().await?,
            OperationMode::Precision => self.start_precision_mode().await?,
            OperationMode::Swarm => self.start_swarm_mode().await?,
        }

        Ok(())
    }

    async fn start_stealth_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🌙 Initializing stealth operations...");
        // TODO: Implement stealth mode logic
        Ok(())
    }

    async fn start_precision_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("⚡ Initializing precision operations...");
        
        // Initialize Coinbase Connector
        let api_key_name = std::env::var("COINBASE_API_KEY_NAME")
            .or_else(|_| std::env::var("COINBASE_API_KEY"))
            .unwrap_or_default();
        let private_key = std::env::var("COINBASE_PRIVATE_KEY")
            .or_else(|_| std::env::var("COINBASE_API_SECRET"))
            .unwrap_or_default();
        
        if !api_key_name.is_empty() && !private_key.is_empty() {
            tracing::info!("Found Coinbase credentials, attempting connection...");
             let config = exchange_connectors::coinbase::CoinbaseConfig {
                api_key_name,
                private_key: private_key.replace("\\n", "\n"),
                sandbox: false, 
                use_advanced_trade: true,
            };
            
            let mut connector = exchange_connectors::coinbase::CoinbaseConnector::new(config);
            match connector.connect().await {
                Ok(_) => tracing::info!("Successfully connected to Coinbase"),
                Err(e) => tracing::error!("Failed to connect to Coinbase: {}", e),
            }
        } else {
            tracing::warn!("Coinbase credentials not found in environment");
        }

        // TODO: Implement precision mode logic
        Ok(())
    }

    async fn start_swarm_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🤖 Initializing swarm operations...");
        // TODO: Implement swarm mode logic
        Ok(())
    }

    /// Get a reference to the MCP client
    pub fn mcp_client(&self) -> &McpClient {
        &self.mcp_client
    }
}

impl NinjaGekkoBuilder {
    /// Set operation mode
    pub fn mode(mut self, mode: OperationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set neural backend
    pub fn neural_backend(mut self, backend: NeuralBackend) -> Self {
        self.neural_backend = backend;
        self
    }

    /// Set MCP servers
    pub fn mcp_servers(mut self, servers: Vec<String>) -> Self {
        self.mcp_servers = servers;
        self
    }

    /// Set dry run mode
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set event bus
    pub fn event_bus(mut self, event_bus: EventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Build the NinjaGekko instance
    pub async fn build(self) -> anyhow::Result<NinjaGekko> {
        let mcp_config = McpConfig {
            servers: self.mcp_servers,
        };
        let mut mcp_client = McpClient::new(mcp_config);

        // Clone event bus for mcp_client integration if available
        if let Some(ref bus) = self.event_bus {
            mcp_client = mcp_client.with_event_bus(bus.clone());
        }

        Ok(NinjaGekko {
            mode: self.mode,
            neural_backend: self.neural_backend,
            mcp_client,
            event_bus: self.event_bus,
            dry_run: self.dry_run,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_mode_display() {
        assert_eq!(OperationMode::Stealth.to_string(), "Stealth");
        assert_eq!(OperationMode::Precision.to_string(), "Precision");
        assert_eq!(OperationMode::Swarm.to_string(), "Swarm");
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let result = NinjaGekko::builder()
            .mode(OperationMode::Stealth)
            .neural_backend(NeuralBackend::RuvFann)
            .mcp_servers(vec!["playwright".to_string()])
            .dry_run(true)
            .build()
            .await;

        assert!(result.is_ok());
        let bot = result.unwrap();
        assert_eq!(bot.mode, OperationMode::Stealth);
        assert!(bot.dry_run);
    }
}

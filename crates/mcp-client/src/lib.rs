//! MCP Client crate for Ninja Gekko
//!
//! This crate provides the client implementation for the Model Context Protocol (MCP),
//! allowing the trading bot to interact with external tools and resources.

pub mod discord_webhook;
pub mod perplexity_browser;
pub mod perplexity_client;
pub mod rate_limiter;

use event_bus::EventBus;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

pub use perplexity_browser::{FallbackReason, PerplexityBrowser, PerplexityFinanceData, PlaywrightConfig, requires_visual_data};
pub use perplexity_client::{classify_query, SonarClient, SonarConfig, SonarModel, SonarResponse};
pub use rate_limiter::SonarRateLimiter;

/// MCP Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: Vec<String>,
}

use crate::discord_webhook::{DiscordConfig, DiscordNotificationService};

/// MCP Client instance
#[derive(Clone)]
pub struct McpClient {
    config: McpConfig,
    event_bus: Option<EventBus>,
    pub discord_service: Option<DiscordNotificationService>,
    sonar_client: Option<SonarClient>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(config: McpConfig) -> Self {
        // Try to initialize Sonar client from environment
        let sonar_client = match SonarClient::from_env() {
            Ok(client) => {
                info!("🔍 Sonar client initialized successfully");
                Some(client)
            }
            Err(e) => {
                warn!("⚠️ Sonar client not initialized: {}", e);
                None
            }
        };

        Self {
            config,
            event_bus: None,
            discord_service: None,
            sonar_client,
        }
    }

    /// Set the event bus for the client
    pub fn with_event_bus(mut self, event_bus: EventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Enable private Discord notifications
    pub fn with_discord_config(mut self, config: Option<DiscordConfig>) -> Self {
        if let Some(cfg) = config {
            self.discord_service = Some(DiscordNotificationService::new(cfg));
        }
        self
    }

    /// Override Sonar client with custom configuration
    pub fn with_sonar_client(mut self, client: SonarClient) -> Self {
        self.sonar_client = Some(client);
        self
    }

    /// Check if Sonar client is available
    pub fn has_sonar(&self) -> bool {
        self.sonar_client.is_some()
    }

    /// Start the MCP client
    pub async fn start(&self) -> anyhow::Result<()> {
        info!(
            "🎭 Starting MCP Client with servers: {:?}",
            self.config.servers
        );

        // Placeholder: Simulate connection to servers
        for server in &self.config.servers {
            info!("🔗 Connecting to MCP server: {}", server);
        }

        // Placeholder: Start event loop if event bus is available
        if let Some(bus) = &self.event_bus {
            let bus_clone = bus.clone();
            tokio::spawn(async move {
                Self::run_event_loop(bus_clone).await;
            });
        }

        Ok(())
    }

    async fn run_event_loop(_event_bus: EventBus) {
        info!("🔄 MCP Client event loop started");
        // Subscribe to relevant events (e.g., requests for MCP tools)
        // For now, we just log
    }

    /// Get account snapshot from connected exchanges via MCP
    pub async fn get_account_snapshot(&self) -> anyhow::Result<serde_json::Value> {
        // Placeholder: In a real implementation, this would query the 'exchange-integration' MCP server
        Ok(serde_json::json!({
            "generated_at": chrono::Utc::now(),
            "total_equity": 2_540_000.23,
            "net_exposure": 0.34,
            "brokers": [
                {
                    "broker": "OANDA",
                    "balance": 1_240_000.0,
                    "open_positions": 12,
                    "risk_score": 0.42
                },
                {
                    "broker": "Coinbase Pro",
                    "balance": 780_000.0,
                    "open_positions": 5,
                    "risk_score": 0.28
                },
                {
                    "broker": "Binance.us",
                    "balance": 520_000.23,
                    "open_positions": 9,
                    "risk_score": 0.51
                }
            ]
        }))
    }

    /// Get latest news headlines via MCP
    pub async fn get_latest_news(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        // Placeholder: Query 'search' or 'news' MCP server
        Ok(vec![
            serde_json::json!({
                "id": uuid::Uuid::new_v4(),
                "title": "Fed minutes flag cautious optimism for Q4",
                "source": "Perplexity Finance",
                "published_at": chrono::Utc::now(),
                "url": "https://perplexity.ai/finance/fed-minutes"
            }),
            serde_json::json!({
                "id": uuid::Uuid::new_v4(),
                "title": "Sonar identifies energy sector leadership rotation",
                "source": "Sonar Deep Research",
                "published_at": chrono::Utc::now(),
                "url": "https://sonar.perplexity.ai/reports/energy-rotation"
            }),
        ])
    }

    /// Perform deep research via Sonar API
    /// 
    /// This method uses the Perplexity Sonar API for real-time financial research.
    /// Query classification automatically selects the appropriate model:
    /// - "sonar": Quick lookups (prices, basic info)
    /// - "sonar-pro": General financial queries
    /// - "sonar-reasoning": Analytical queries requiring explanation
    /// - "sonar-deep-research": Complex multi-step analysis
    pub async fn perform_research(&self, query: String) -> anyhow::Result<serde_json::Value> {
        // Use Sonar API if available
        if let Some(sonar) = &self.sonar_client {
            // Classify query to select appropriate model
            let model = classify_query(&query);
            info!("🔍 Research query classified as {:?}", model);

            match sonar.research(&query, Some(model)).await {
                Ok(response) => {
                    // Convert SonarResponse to JSON for API compatibility
                    let content = response
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default();

                    let citations: Vec<serde_json::Value> = response
                        .citations
                        .into_iter()
                        .map(|c| {
                            serde_json::json!({
                                "type": "external",
                                "title": c.title.unwrap_or_else(|| "Source".to_string()),
                                "url": c.url
                            })
                        })
                        .collect();

                    return Ok(serde_json::json!({
                        "task_id": response.id,
                        "query": query,
                        "model": response.model,
                        "summary": content,
                        "citations": citations,
                        "usage": response.usage.map(|u| serde_json::json!({
                            "prompt_tokens": u.prompt_tokens,
                            "completion_tokens": u.completion_tokens,
                            "total_tokens": u.total_tokens
                        }))
                    }));
                }
                Err(e) => {
                    warn!("⚠️ Sonar API error, falling back to stub: {}", e);
                    // Fall through to stub response
                }
            }
        }

        // Fallback: Return stub response when Sonar is unavailable
        info!("📝 Using stub research response (Sonar unavailable)");
        Ok(serde_json::json!({
            "task_id": uuid::Uuid::new_v4(),
            "query": query,
            "model": "stub",
            "summary": "Sonar API not configured. Set PERPLEXITY_API_KEY in .env to enable real-time research.",
            "citations": [
                {
                    "type": "external",
                    "title": "Configuration Guide",
                    "url": "https://docs.perplexity.ai/"
                }
            ]
        }))
    }

    /// Perform deep research (convenience method for complex queries)
    pub async fn deep_research(&self, query: String) -> anyhow::Result<serde_json::Value> {
        if let Some(sonar) = &self.sonar_client {
            match sonar.deep_research(&query).await {
                Ok(response) => {
                    let content = response
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default();

                    let citations: Vec<serde_json::Value> = response
                        .citations
                        .into_iter()
                        .map(|c| {
                            serde_json::json!({
                                "type": "external",
                                "title": c.title.unwrap_or_else(|| "Source".to_string()),
                                "url": c.url
                            })
                        })
                        .collect();

                    return Ok(serde_json::json!({
                        "task_id": response.id,
                        "query": query,
                        "model": "sonar-deep-research",
                        "summary": content,
                        "citations": citations
                    }));
                }
                Err(e) => {
                    warn!("⚠️ Deep research failed: {}", e);
                }
            }
        }

        // Fallback to regular research
        self.perform_research(query).await
    }

    /// Summon a swarm of agents via MCP
    pub async fn summon_swarm(&self, task: String) -> anyhow::Result<serde_json::Value> {
        // Placeholder: Trigger 'swarm' MCP server
        Ok(serde_json::json!({
            "swarm_id": uuid::Uuid::new_v4(),
            "task": task,
            "status": "initiated",
            "eta_seconds": 42
        }))
    }

    /// Get remaining Sonar API requests for a model
    pub fn sonar_remaining(&self, model: SonarModel) -> Option<u32> {
        self.sonar_client.as_ref().map(|c| c.remaining_requests(model))
    }
}

pub fn hello() {
    println!("MCP Client initialized");
}


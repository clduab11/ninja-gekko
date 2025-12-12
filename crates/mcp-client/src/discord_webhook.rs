use anyhow::{Context, Result};
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, instrument};

/// Configuration for Discord Notification Service
#[derive(Clone)]
pub struct DiscordConfig {
    pub webhook_url: Secret<String>,
    pub authorized_server_id: Option<String>,
    pub bot_name: String,
}

/// Message payload for Discord Webhook
#[derive(Debug, Serialize, Clone)]
struct WebhookPayload {
    content: String,
    username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
}

/// Service for sending private notifications to Gordon's Discord
#[derive(Clone)]
pub struct DiscordNotificationService {
    client: Client,
    config: DiscordConfig,
    history: Arc<RwLock<VecDeque<String>>>, // Recursion history
}

impl DiscordNotificationService {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            history: Arc::new(RwLock::new(VecDeque::with_capacity(50))),
        }
    }

    /// Send a message to the private Discord channel
    #[instrument(skip(self, content), fields(length = content.len()))]
    pub async fn send_message(&self, content: &str) -> Result<()> {
        let payload = WebhookPayload {
            content: content.to_string(),
            username: self.config.bot_name.clone(),
            avatar_url: None,
        };

        // Recursive Analysis: Store before sending
        self.record_message(content).await;

        let url = self.config.webhook_url.expose_secret();

        let response = self
            .client
            .post(url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send Discord webhook request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Discord Webhook failed: {} - {}", status, text);
            return Err(anyhow::anyhow!("Discord Webhook failed: {}", status));
        }

        info!("📨 Sent Discord notification: {:.30}...", content);
        Ok(())
    }

    /// Recursive Analysis: Review recent communication patterns
    /// Gordon uses this to "think" about its own outputs
    pub async fn analyze_recursive_feedback(&self) -> Vec<String> {
        let history = self.history.read().await;
        history.iter().cloned().collect()
    }

    async fn record_message(&self, content: &str) {
        let mut history = self.history.write().await;
        if history.len() >= 50 {
            history.pop_front();
        }
        history.push_back(content.to_string());
    }
}

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::time::Duration;
use tracing::{debug, error, info, warn};

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Clone)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "anthropic/claude-3.5-sonnet".to_string(), // Default logic model
            timeout_secs: 60,
        }
    }
}

#[derive(Clone)]
pub struct OpenRouterClient {
    client: Client,
    config: OpenRouterConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<ChatChoice>,
    pub model: String,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

impl OpenRouterClient {
    pub fn new(config: OpenRouterConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build reqwest client");

        Self { client, config }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENROUTER_API_KEY not set"))?;
        
        // Optional model override
        let model = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "anthropic/claude-3.5-sonnet".to_string());

        let config = OpenRouterConfig {
            api_key,
            model,
            ..Default::default()
        };

        Ok(Self::new(config))
    }

    pub async fn chat_completion(&self, messages: Vec<ChatMessage>, model_override: Option<String>) -> anyhow::Result<ChatCompletionResponse> {
        let model = model_override.as_deref().unwrap_or(&self.config.model);
        
        let payload = json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7,
            "stream": false
        });

        debug!("Sending OpenRouter request (model: {})", model);

        let response = self.client
            .post(OPENROUTER_API_URL)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("HTTP-Referer", "https://ninja-gekko.ai") // Requested by OpenRouter
            .header("X-Title", "Ninja Gekko")
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenRouter API error ({}): {}", status, error_text);
            return Err(anyhow::anyhow!("OpenRouter API error: {} - {}", status, error_text));
        }

        let chat_response: ChatCompletionResponse = response.json().await?;
        Ok(chat_response)
    }
}

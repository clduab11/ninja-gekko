//! Perplexity Sonar API client for Gordon's research intelligence
//!
//! Provides integration with Perplexity's Sonar models for real-time
//! financial research with citation-backed responses.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::rate_limiter::SonarRateLimiter;

/// Sonar API client configuration
#[derive(Debug, Clone)]
pub struct SonarConfig {
    pub api_key: String,
    pub base_url: String,
    pub default_model: SonarModel,
    pub timeout_ms: u64,
}

impl SonarConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            api_key: std::env::var("PERPLEXITY_API_KEY")?,
            base_url: std::env::var("PERPLEXITY_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.perplexity.ai".to_string()),
            default_model: std::env::var("SONAR_DEFAULT_MODEL")
                .map(|s| SonarModel::from_str(&s))
                .unwrap_or(SonarModel::SonarPro),
            timeout_ms: std::env::var("SONAR_REQUEST_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30000),
        })
    }
}

/// Available Sonar models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SonarModel {
    Sonar,
    SonarPro,
    SonarReasoning,
    SonarReasoningPro,
    SonarDeepResearch,
}

impl SonarModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sonar => "sonar",
            Self::SonarPro => "sonar-pro",
            Self::SonarReasoning => "sonar-reasoning",
            Self::SonarReasoningPro => "sonar-reasoning-pro",
            Self::SonarDeepResearch => "sonar-deep-research",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sonar" => Self::Sonar,
            "sonar-pro" => Self::SonarPro,
            "sonar-reasoning" => Self::SonarReasoning,
            "sonar-reasoning-pro" => Self::SonarReasoningPro,
            "sonar-deep-research" => Self::SonarDeepResearch,
            _ => Self::SonarPro, // Default fallback
        }
    }

    /// Get rate limit for this model (requests per minute)
    pub fn rate_limit(&self) -> u32 {
        match self {
            Self::SonarDeepResearch => 5,
            _ => 50,
        }
    }
}

/// Chat message for Sonar API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SonarMessage {
    pub role: String,
    pub content: String,
}

/// Request payload for Sonar API
#[derive(Debug, Serialize)]
pub struct SonarRequest {
    pub model: String,
    pub messages: Vec<SonarMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_recency_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_citations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Citation from Sonar response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SonarCitation {
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
}

/// Choice from Sonar response
#[derive(Debug, Deserialize)]
pub struct SonarChoice {
    pub message: SonarMessage,
    pub finish_reason: Option<String>,
}

/// Usage statistics from response
#[derive(Debug, Deserialize)]
pub struct SonarUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Response from Sonar API
#[derive(Debug, Deserialize)]
pub struct SonarResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<SonarChoice>,
    #[serde(default)]
    pub citations: Vec<SonarCitation>,
    #[serde(default)]
    pub usage: Option<SonarUsage>,
}

/// Error types for Sonar API
#[derive(Debug, thiserror::Error)]
pub enum SonarError {
    #[error("Rate limit exceeded for model {model}")]
    RateLimited { model: String },
    #[error("API request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API key")]
    Unauthorized,
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Perplexity Sonar API client
#[derive(Clone)]
pub struct SonarClient {
    config: SonarConfig,
    client: Client,
    rate_limiter: SonarRateLimiter,
}

impl SonarClient {
    /// Create a new Sonar client from environment configuration
    pub fn from_env() -> Result<Self, SonarError> {
        let config = SonarConfig::from_env()
            .map_err(|e| SonarError::ConfigError(format!("Missing env var: {}", e)))?;
        Ok(Self::new(config))
    }

    /// Create a new Sonar client with explicit configuration
    pub fn new(config: SonarConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        let rate_limiter = SonarRateLimiter::new();
        
        Self {
            config,
            client,
            rate_limiter,
        }
    }

    /// Perform a research query using the appropriate Sonar model
    pub async fn research(
        &self,
        query: &str,
        model: Option<SonarModel>,
    ) -> Result<SonarResponse, SonarError> {
        let model = model.unwrap_or(self.config.default_model);
        let model_str = model.as_str();

        // Check rate limit
        if !self.rate_limiter.try_acquire(model_str) {
            warn!("Rate limited on model '{}', remaining: {}", model_str, self.rate_limiter.remaining(model_str));
            return Err(SonarError::RateLimited {
                model: model_str.to_string(),
            });
        }

        info!("🔍 Sonar research query using model '{}'", model_str);
        debug!("Query: {}", query);

        let request = SonarRequest {
            model: model_str.to_string(),
            messages: vec![
                SonarMessage {
                    role: "system".to_string(),
                    content: "You are Gordon Gekko, an expert financial analyst. Provide concise, actionable insights with supporting data.".to_string(),
                },
                SonarMessage {
                    role: "user".to_string(),
                    content: query.to_string(),
                },
            ],
            search_recency_filter: Some("day".to_string()),
            return_citations: Some(true),
            stream: Some(false),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            let sonar_response: SonarResponse = response.json().await?;
            info!(
                "✅ Sonar response received: {} tokens used, {} citations",
                sonar_response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
                sonar_response.citations.len()
            );
            Ok(sonar_response)
        } else if status.as_u16() == 401 {
            error!("❌ Perplexity API key is invalid or expired");
            Err(SonarError::Unauthorized)
        } else if status.as_u16() == 429 {
            warn!("⚠️ Rate limited by Perplexity API (429)");
            Err(SonarError::RateLimited {
                model: model_str.to_string(),
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Sonar API error: {} - {}", status, error_text);
            Err(SonarError::ApiError {
                status: status.as_u16(),
                message: error_text,
            })
        }
    }

    /// Deep research for complex financial analysis
    pub async fn deep_research(&self, query: &str) -> Result<SonarResponse, SonarError> {
        self.research(query, Some(SonarModel::SonarDeepResearch)).await
    }

    /// Quick lookup for simple queries (prices, basic info)
    pub async fn quick_lookup(&self, query: &str) -> Result<SonarResponse, SonarError> {
        self.research(query, Some(SonarModel::Sonar)).await
    }

    /// Get remaining API requests for a model
    pub fn remaining_requests(&self, model: SonarModel) -> u32 {
        self.rate_limiter.remaining(model.as_str())
    }

    /// Check if a model is currently rate limited
    pub fn is_rate_limited(&self, model: SonarModel) -> bool {
        self.rate_limiter.remaining(model.as_str()) == 0
    }
}

/// Classify a query to determine the appropriate Sonar model
pub fn classify_query(query: &str) -> SonarModel {
    let query_lower = query.to_lowercase();

    // Deep research triggers
    let deep_research_keywords = [
        "analyze", "deep dive", "research", "investigate",
        "compare", "historical", "trend analysis", "catalyst",
        "earnings", "sec filing", "management commentary",
        "sector rotation", "macro outlook", "risk assessment",
        "comprehensive", "detailed", "in-depth",
    ];

    // Quick lookup triggers
    let quick_lookup_keywords = [
        "price", "current", "quote", "what is", "how much",
        "today", "now", "latest",
    ];

    // Reasoning triggers
    let reasoning_keywords = [
        "why", "explain", "reasoning", "implications",
        "predict", "forecast", "likely",
    ];

    if deep_research_keywords.iter().any(|kw| query_lower.contains(kw)) {
        SonarModel::SonarDeepResearch
    } else if quick_lookup_keywords.iter().any(|kw| query_lower.contains(kw)) {
        SonarModel::Sonar
    } else if reasoning_keywords.iter().any(|kw| query_lower.contains(kw)) {
        SonarModel::SonarReasoning
    } else {
        SonarModel::SonarPro // Default for general queries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_as_str() {
        assert_eq!(SonarModel::Sonar.as_str(), "sonar");
        assert_eq!(SonarModel::SonarDeepResearch.as_str(), "sonar-deep-research");
    }

    #[test]
    fn test_model_rate_limits() {
        assert_eq!(SonarModel::Sonar.rate_limit(), 50);
        assert_eq!(SonarModel::SonarDeepResearch.rate_limit(), 5);
    }

    #[test]
    fn test_query_classification() {
        assert_eq!(classify_query("What is the current BTC price?"), SonarModel::Sonar);
        assert_eq!(classify_query("Analyze NVDA Q3 earnings"), SonarModel::SonarDeepResearch);
        assert_eq!(classify_query("Why did the market drop?"), SonarModel::SonarReasoning);
        assert_eq!(classify_query("Tell me about Apple"), SonarModel::SonarPro);
    }
}

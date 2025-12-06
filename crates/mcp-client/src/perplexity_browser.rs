//! Playwright browser automation fallback for Perplexity.ai/finance
//!
//! This module provides browser-based data extraction when:
//! - Sonar API rate limits are exceeded
//! - Visual data (charts, tables) is required
//! - Complex interactive features need exploration

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Configuration for Playwright browser automation
#[derive(Debug, Clone)]
pub struct PlaywrightConfig {
    /// Whether browser fallback is enabled
    pub enabled: bool,
    /// Page load timeout in milliseconds
    pub page_load_timeout_ms: u64,
    /// Perplexity Finance base URL
    pub finance_url: String,
}

impl PlaywrightConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("PERPLEXITY_BROWSER_FALLBACK_ENABLED")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            page_load_timeout_ms: std::env::var("PLAYWRIGHT_PAGE_LOAD_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000),
            finance_url: std::env::var("PERPLEXITY_FINANCE_URL")
                .unwrap_or_else(|_| "https://www.perplexity.ai/finance".to_string()),
        }
    }
}

impl Default for PlaywrightConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Data extracted from Perplexity Finance page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerplexityFinanceData {
    /// Stock/asset symbol
    pub symbol: Option<String>,
    /// Current price
    pub price: Option<String>,
    /// Price change (e.g., "+2.5%")
    pub change: Option<String>,
    /// Additional financial metrics
    pub metrics: Vec<FinancialMetric>,
    /// Raw HTML content (if needed for further parsing)
    pub raw_content: Option<String>,
    /// Any errors encountered during extraction
    pub errors: Vec<String>,
}

/// A single financial metric extracted from the page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetric {
    pub label: String,
    pub value: String,
}

/// Playwright MCP command builder for Perplexity navigation
#[derive(Debug)]
pub struct PerplexityBrowser {
    config: PlaywrightConfig,
}

impl PerplexityBrowser {
    /// Create a new browser automation instance
    pub fn new(config: PlaywrightConfig) -> Self {
        Self { config }
    }

    /// Create from environment configuration
    pub fn from_env() -> Self {
        Self::new(PlaywrightConfig::from_env())
    }

    /// Check if browser fallback is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Build navigation command for MCP Playwright server
    pub fn build_navigate_command(&self, query: &str) -> serde_json::Value {
        let encoded_query = urlencoding::encode(query);
        let url = format!("{}?q={}", self.config.finance_url, encoded_query);
        
        info!("🌐 Building Playwright navigation to: {}", url);
        
        serde_json::json!({
            "command": "navigate",
            "params": {
                "url": url,
                "timeout": self.config.page_load_timeout_ms,
                "waitUntil": "networkidle"
            }
        })
    }

    /// Build scraping command for financial data extraction
    pub fn build_scrape_command(&self) -> serde_json::Value {
        debug!("🔍 Building Playwright scrape command for Perplexity Finance");
        
        serde_json::json!({
            "command": "scrape",
            "params": {
                "selectors": {
                    "price": "[data-testid='stock-price'], .stock-price, .current-price",
                    "change": "[data-testid='price-change'], .price-change, .change-percent",
                    "symbol": "[data-testid='stock-symbol'], .stock-symbol, h1.symbol",
                    "metrics": "[data-testid='financial-metrics'] > div, .metric-row",
                    "summary": "[data-testid='ai-summary'], .research-summary, .answer-text"
                },
                "timeout": 5000
            }
        })
    }

    /// Build screenshot command for visual data capture
    pub fn build_screenshot_command(&self, path: &str) -> serde_json::Value {
        serde_json::json!({
            "command": "screenshot",
            "params": {
                "path": path,
                "fullPage": false,
                "type": "png"
            }
        })
    }

    /// Build a complete research workflow command sequence
    pub fn build_research_workflow(&self, query: &str) -> Vec<serde_json::Value> {
        vec![
            self.build_navigate_command(query),
            serde_json::json!({
                "command": "wait",
                "params": {
                    "duration_ms": 2000
                }
            }),
            self.build_scrape_command(),
        ]
    }

    /// Parse raw scrape results into structured PerplexityFinanceData
    pub fn parse_scrape_result(&self, raw: serde_json::Value) -> PerplexityFinanceData {
        let mut data = PerplexityFinanceData {
            symbol: None,
            price: None,
            change: None,
            metrics: Vec::new(),
            raw_content: None,
            errors: Vec::new(),
        };

        // Extract individual fields from scrape result
        if let Some(obj) = raw.as_object() {
            if let Some(status) = obj.get("status") {
                if status != "success" {
                    data.errors.push(format!("Scrape failed: {:?}", obj.get("error")));
                    return data;
                }
            }

            if let Some(result_data) = obj.get("data").and_then(|d| d.as_object()) {
                data.symbol = result_data.get("symbol")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                data.price = result_data.get("price")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                data.change = result_data.get("change")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                // Parse metrics array
                if let Some(metrics) = result_data.get("metrics").and_then(|m| m.as_array()) {
                    for metric in metrics {
                        if let Some(metric_obj) = metric.as_object() {
                            if let (Some(label), Some(value)) = (
                                metric_obj.get("label").and_then(|l| l.as_str()),
                                metric_obj.get("value").and_then(|v| v.as_str()),
                            ) {
                                data.metrics.push(FinancialMetric {
                                    label: label.to_string(),
                                    value: value.to_string(),
                                });
                            }
                        }
                    }
                }

                data.raw_content = result_data.get("summary")
                    .and_then(|v| v.as_str())
                    .map(String::from);
            }
        }

        data
    }
}

/// Determines when to use Playwright fallback vs API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackReason {
    /// API rate limit exceeded (429 response)
    RateLimited,
    /// Visual data required (charts, screenshots)
    VisualDataRequired,
    /// API returned insufficient data
    InsufficientApiData,
    /// Explicit user request for browser-based research
    UserRequested,
    /// API temporarily unavailable
    ApiUnavailable,
}

impl std::fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited => write!(f, "API rate limit exceeded"),
            Self::VisualDataRequired => write!(f, "Visual data extraction required"),
            Self::InsufficientApiData => write!(f, "API returned insufficient data"),
            Self::UserRequested => write!(f, "User requested browser-based research"),
            Self::ApiUnavailable => write!(f, "API temporarily unavailable"),
        }
    }
}

/// Check if a query requires visual data extraction
pub fn requires_visual_data(query: &str) -> bool {
    let visual_keywords = [
        "chart", "graph", "screenshot", "visual",
        "price history", "historical chart", "technical analysis",
        "candlestick", "trend line", "support resistance",
    ];
    
    let query_lower = query.to_lowercase();
    visual_keywords.iter().any(|kw| query_lower.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playwright_config_default() {
        let config = PlaywrightConfig::default();
        assert!(config.enabled);
        assert_eq!(config.page_load_timeout_ms, 10000);
    }

    #[test]
    fn test_navigate_command() {
        let browser = PerplexityBrowser::from_env();
        let cmd = browser.build_navigate_command("AAPL stock price");
        
        assert_eq!(cmd["command"], "navigate");
        assert!(cmd["params"]["url"].as_str().unwrap().contains("AAPL"));
    }

    #[test]
    fn test_requires_visual_data() {
        assert!(requires_visual_data("Show me the AAPL chart"));
        assert!(requires_visual_data("Technical analysis of BTC"));
        assert!(!requires_visual_data("What is the current price of ETH?"));
    }

    #[test]
    fn test_fallback_reason_display() {
        assert_eq!(
            FallbackReason::RateLimited.to_string(),
            "API rate limit exceeded"
        );
    }
}

//! RSS Feed Integration for IntelManager
//! 
//! Fetches and parses RSS feeds from financial news sources,
//! converting them to IntelItem structs for the Intel Stream.

use chrono::{DateTime, Utc};
use reqwest::Client;
use crate::handlers::intel::IntelItem;
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// RSS Feed sources for financial intelligence
pub const RSS_SOURCES: &[(&str, &str)] = &[
    ("CoinDesk", "https://www.coindesk.com/arc/outboundfeeds/rss/"),
    ("Cointelegraph", "https://cointelegraph.com/rss"),
    ("CryptoPanic", "https://cryptopanic.com/news/rss/"),
];

/// Cached RSS items with last fetch timestamp
pub struct RssCache {
    pub items: Vec<IntelItem>,
    pub last_fetched: DateTime<Utc>,
}

impl Default for RssCache {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            last_fetched: DateTime::UNIX_EPOCH,
        }
    }
}

/// Fetches RSS feeds from all configured sources and returns IntelItems
pub async fn fetch_rss_feeds(http_client: &Client) -> Vec<IntelItem> {
    let mut all_items = Vec::new();
    
    for (source_name, url) in RSS_SOURCES {
        match fetch_single_feed(http_client, source_name, url).await {
            Ok(items) => {
                info!("Fetched {} items from {}", items.len(), source_name);
                all_items.extend(items);
            }
            Err(e) => {
                warn!("Failed to fetch RSS from {}: {}", source_name, e);
            }
        }
    }
    
    // Sort by published date, newest first
    all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    
    // Limit to most recent 50 items
    all_items.truncate(50);
    
    all_items
}

/// Fetches a single RSS feed and converts to IntelItems
async fn fetch_single_feed(
    http_client: &Client,
    source_name: &str,
    url: &str,
) -> Result<Vec<IntelItem>, Box<dyn std::error::Error + Send + Sync>> {
    let response = http_client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    let bytes = response.bytes().await?;
    let feed = feed_rs::parser::parse(&bytes[..])?;
    
    let items: Vec<IntelItem> = feed.entries
        .into_iter()
        .take(10) // Limit entries per feed
        .filter_map(|entry| {
            let title = entry.title.map(|t| t.content).unwrap_or_default();
            if title.is_empty() {
                return None;
            }
            
            let summary = entry.summary.map(|s| {
                // Strip HTML tags from summary
                let plain = s.content
                    .replace("<p>", "")
                    .replace("</p>", " ")
                    .replace("<br>", " ")
                    .replace("&nbsp;", " ");
                // Truncate to 200 chars
                if plain.len() > 200 {
                    format!("{}...", &plain[..200])
                } else {
                    plain
                }
            });
            
            let url = entry.links.first().map(|l| l.href.clone());
            
            // Extract published date or use current time
            let published_at = entry.published
                .or(entry.updated)
                .unwrap_or_else(Utc::now);
            
            // Simple sentiment placeholder - would integrate with LLM later
            let sentiment = determine_sentiment(&title);
            
            Some(IntelItem {
                id: uuid::Uuid::new_v4().to_string(),
                source: format!("📰 {}", source_name.to_uppercase()),
                title,
                summary,
                url,
                sentiment: Some(sentiment),
                published_at,
                relevance_score: 0.7, // RSS news gets mid-high relevance
            })
        })
        .collect();
    
    Ok(items)
}

/// Simple keyword-based sentiment analysis (placeholder for LLM integration)
fn determine_sentiment(title: &str) -> f32 {
    let title_lower = title.to_lowercase();
    
    let bullish_keywords = ["surge", "rally", "soar", "bullish", "gains", "rise", 
        "jump", "moon", "record", "high", "adoption", "approval"];
    let bearish_keywords = ["crash", "fall", "drop", "bearish", "loss", "plunge",
        "fear", "sell", "dump", "low", "reject", "ban"];
    
    let bullish_count = bullish_keywords.iter()
        .filter(|k| title_lower.contains(*k))
        .count();
    let bearish_count = bearish_keywords.iter()
        .filter(|k| title_lower.contains(*k))
        .count();
    
    if bullish_count > bearish_count {
        0.7 + (bullish_count as f32 * 0.1).min(0.3) // 0.7 to 1.0
    } else if bearish_count > bullish_count {
        0.3 - (bearish_count as f32 * 0.1).max(0.0) // 0.0 to 0.3
    } else {
        0.5 // Neutral
    }
}

/// Starts a background task that periodically fetches RSS feeds
/// and updates the provided cache
pub async fn start_rss_polling_task(
    cache: Arc<RwLock<RssCache>>,
    poll_interval_secs: u64,
) {
    let http_client = Client::builder()
        .user_agent("NinjaGekko/1.0 IntelManager")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    loop {
        info!("RSS Poller: Fetching feeds...");
        
        let items = fetch_rss_feeds(&http_client).await;
        
        {
            let mut cache_guard = cache.write().await;
            cache_guard.items = items;
            cache_guard.last_fetched = Utc::now();
        }
        
        info!("RSS Poller: Cache updated, sleeping for {}s", poll_interval_secs);
        tokio::time::sleep(std::time::Duration::from_secs(poll_interval_secs)).await;
    }
}

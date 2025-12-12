use crate::{error::ApiResult, models::ApiResponse, AppState};
use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelItem {
    pub id: String,
    pub source: String, // "PERPLEXITY FINANCE", "SONAR DEEP RESEARCH", etc.
    pub title: String,
    pub summary: Option<String>,
    pub url: Option<String>,
    pub sentiment: Option<f32>, // -1.0 to 1.0
    pub published_at: DateTime<Utc>,
    pub relevance_score: f32, // For Gordon's prioritization
}

#[derive(Deserialize)]
pub struct IntelStreamParams {
    pub limit: Option<usize>,
    pub min_relevance: Option<f32>,
}

/// Get intel stream
///
/// Returns real-time market intelligence from connected data sources:
/// - Live market data from Kraken
/// - Research from Perplexity Finance (when configured)
/// - Technical analysis signals
pub async fn get_intel_stream(
    State(state): State<Arc<AppState>>,
    Query(params): Query<IntelStreamParams>,
) -> ApiResult<Json<ApiResponse<Vec<IntelItem>>>> {
    let mut items = Vec::new();
    let limit = params.limit.unwrap_or(20);
    let min_relevance = params.min_relevance.unwrap_or(0.0);

    // ---------------------------------------------------------
    // Live Market Data from Kraken (High Priority)
    // ---------------------------------------------------------
    let symbols = vec!["XBT/USD", "ETH/USD", "SOL/USD"];

    for symbol in symbols {
        if let Ok(data) = state.market_data_service.get_latest_data(symbol).await {
            // Derive sentiment from price movement
            let sentiment = if data.change_24h > 2.0 {
                0.9
            } else if data.change_24h > 0.0 {
                0.6
            } else if data.change_24h > -2.0 {
                0.4
            } else {
                0.2
            };

            let direction = if data.change_24h >= 0.0 {
                "📈"
            } else {
                "📉"
            };
            let relevance = 0.95; // Live market data is always highly relevant

            if relevance >= min_relevance {
                items.push(IntelItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    source: "KRAKEN MARKET FEED".to_string(),
                    title: format!(
                        "{} {} ${:.2} ({:+.2}%)",
                        direction, symbol, data.price, data.change_24h
                    ),
                    summary: Some(format!(
                        "24h Volume: ${:.2}M",
                        data.volume_24h / 1_000_000.0
                    )),
                    url: None,
                    sentiment: Some(sentiment),
                    published_at: data.timestamp,
                    relevance_score: relevance,
                });
            }
        }
    }

    // If no live data available (e.g., startup), provide status message
    if items.is_empty() {
        items.push(IntelItem {
            id: uuid::Uuid::new_v4().to_string(),
            source: "SYSTEM".to_string(),
            title: "Market Data Stream Initializing...".to_string(),
            summary: Some("Connecting to Kraken for live market data...".to_string()),
            url: None,
            sentiment: Some(0.0),
            published_at: Utc::now(),
            relevance_score: 1.0,
        });
    }

    // Apply limit
    items.truncate(limit);

    Ok(Json(ApiResponse::success(items)))
}

/// Broadcast live intel update to WebSocket subscribers
///
/// Called by market data stream handlers when new data arrives.
/// Broadcasts real market intelligence to connected clients.
pub async fn broadcast_live_intel(state: Arc<AppState>, symbol: &str) {
    if let Ok(data) = state.market_data_service.get_latest_data(symbol).await {
        let sentiment = if data.change_24h > 0.0 { 0.6 } else { 0.4 };
        let direction = if data.change_24h >= 0.0 {
            "📈"
        } else {
            "📉"
        };

        let item = IntelItem {
            id: uuid::Uuid::new_v4().to_string(),
            source: "KRAKEN LIVE".to_string(),
            title: format!(
                "{} {} ${:.2} ({:+.2}%)",
                direction, symbol, data.price, data.change_24h
            ),
            summary: Some(format!(
                "Live tick @ {}",
                data.timestamp.format("%H:%M:%S UTC")
            )),
            url: None,
            sentiment: Some(sentiment),
            published_at: Utc::now(),
            relevance_score: 0.95,
        };

        if let Err(e) = state.websocket_manager.broadcast_intel_update(item).await {
            tracing::warn!("Failed to broadcast intel update: {}", e);
        }
    }
}

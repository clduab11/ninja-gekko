use axum::{
    extract::{State, Query},
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use crate::{
    AppState,
    error::ApiResult,
    models::ApiResponse,
    websocket::SubscriptionType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelItem {
    pub id: String,
    pub source: String,        // "PERPLEXITY FINANCE", "SONAR DEEP RESEARCH", etc.
    pub title: String,
    pub summary: Option<String>,
    pub url: Option<String>,
    pub sentiment: Option<f32>, // -1.0 to 1.0
    pub published_at: DateTime<Utc>,
    pub relevance_score: f32,   // For Gordon's prioritization
}

#[derive(Deserialize)]
pub struct IntelStreamParams {
    pub limit: Option<usize>,
    pub min_relevance: Option<f32>,
}

/// Get intel stream
pub async fn get_intel_stream(
    State(state): State<Arc<AppState>>,
    Query(params): Query<IntelStreamParams>,
) -> ApiResult<Json<ApiResponse<Vec<IntelItem>>>> {
    let mut items = Vec::new();

    // ---------------------------------------------------------
    // 1. Live Market Data (High Priority)
    // ---------------------------------------------------------
    let symbols = vec!["XBT/USD", "ETH/USD", "SOL/USD"];
    
    for symbol in symbols {
        if let Ok(data) = state.market_data_service.get_latest_data(symbol).await {
             // Simple sentiment logic
             let sentiment = if data.change_24h > 0.0 { 0.7 } else { 0.3 };
             let direction = if data.change_24h >= 0.0 { "📈" } else { "📉" };

             items.push(IntelItem {
                id: uuid::Uuid::new_v4().to_string(),
                source: "KRAKEN MARKET FEED".to_string(),
                title: format!("{} {} ${:.2} ({:+.2}%)", direction, symbol, data.price, data.change_24h),
                summary: Some(format!("24h Volume: ${:.2}M", data.volume_24h / 1_000_000.0)),
                url: None,
                sentiment: Some(sentiment),
                published_at: data.timestamp,
                relevance_score: 0.95,
            });
        }
    }

    // If no live data (e.g. startup), fallback to a system status message so list isn't empty
    if items.is_empty() {
        items.push(IntelItem {
            id: uuid::Uuid::new_v4().to_string(),
            source: "SYSTEM".to_string(),
            title: "Market Data Stream Initializing...".to_string(),
            summary: Some("Waiting for live ticks from Kraken...".to_string()),
            url: None,
            sentiment: Some(0.0),
            published_at: Utc::now(),
            relevance_score: 1.0,
        });
    }

    Ok(Json(ApiResponse::success(items)))
}

/// Start streaming intel updates (simulated for now)
/// This would be called by a background task or webhook handler
pub async fn broadcast_mock_intel(state: Arc<AppState>) {
    // Generate intelligent market update
    let symbol = "BTC-USD";
    if let Ok(data) = state.market_data_service.get_latest_data(symbol).await {
        let item = IntelItem {
            id: uuid::Uuid::new_v4().to_string(),
            source: "KRAKEN LIVE".to_string(),
            title: format!("{} Price Action: ${:.2}", symbol, data.price),
            summary: Some(format!("Live tick received. Timestamp: {}", data.timestamp)),
            url: None,
            sentiment: Some(0.5), // Neutral for now
            published_at: Utc::now(),
            relevance_score: 0.95,
        };

        if let Err(e) = state.websocket_manager.broadcast_intel_update(item).await {
            tracing::warn!("Failed to broadcast intel update: {}", e);
        }
    }
}

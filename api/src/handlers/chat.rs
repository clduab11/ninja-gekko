//! Chat and miscellaneous frontend handlers
//!
//! This module provides handlers for the chat UI and other frontend-specific features
//! that don't fit into the core REST API structure yet.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{
    error::{ApiError, ApiResult},
    models::ApiResponse,
    AppState,
};

// --- Models ---

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: ChatMessage,
    pub persona: PersonaSettings,
    pub actions: Vec<serde_json::Value>,
    pub diagnostics: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonaSettings {
    pub tone: String,
    pub style: String,
    pub mood: String,
}

#[derive(Debug, Deserialize)]
pub struct PauseTradingRequest {
    pub duration_hours: f64,
}

#[derive(Debug, Serialize)]
pub struct PauseTradingResponse {
    pub id: String,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AccountSnapshot {
    pub generated_at: String,
    pub total_equity: f64,
    pub net_exposure: f64,
    pub brokers: Vec<BrokerSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct BrokerSnapshot {
    pub broker: String,
    pub balance: f64,
    pub open_positions: i32,
    pub risk_score: f64,
}

#[derive(Debug, Serialize)]
pub struct NewsHeadline {
    pub id: String,
    pub title: String,
    pub source: String,
    pub published_at: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ResearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct ResearchResponse {
    pub task_id: String,
    pub query: String,
    pub summary: String,
    pub citations: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SwarmRequest {
    pub task: String,
}

#[derive(Debug, Serialize)]
pub struct SwarmResponse {
    pub swarm_id: String,
    pub task: String,
    pub status: String,
    pub eta_seconds: i32,
}

// --- Handlers ---

pub async fn get_chat_history() -> ApiResult<Json<Vec<ChatMessage>>> {
    Ok(Json(vec![
        ChatMessage {
            id: "1".to_string(),
            role: "assistant".to_string(),
            content: "Hello! I am Gordon. How can I help you dominate the market today?".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    ]))
}

pub async fn send_message(
    Json(request): Json<SendMessageRequest>,
) -> ApiResult<Json<ChatResponse>> {
    Ok(Json(ChatResponse {
        reply: ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: format!("I received your message: '{}'. Market analysis is running.", request.prompt),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        persona: PersonaSettings {
            tone: "dramatic".to_string(),
            style: "direct".to_string(),
            mood: "witty".to_string(),
        },
        actions: vec![],
        diagnostics: vec![],
    }))
}

pub async fn get_persona() -> ApiResult<Json<PersonaSettings>> {
    Ok(Json(PersonaSettings {
        tone: "dramatic".to_string(),
        style: "direct".to_string(),
        mood: "witty".to_string(),
    }))
}

pub async fn update_persona(
    Json(settings): Json<PersonaSettings>,
) -> ApiResult<Json<PersonaSettings>> {
    Ok(Json(settings))
}

pub async fn pause_trading(
    Json(request): Json<PauseTradingRequest>,
) -> ApiResult<Json<PauseTradingResponse>> {
    Ok(Json(PauseTradingResponse {
        id: uuid::Uuid::new_v4().to_string(),
        message: format!("Trading paused for {} hours.", request.duration_hours),
        status: "paused".to_string(),
    }))
}

pub async fn get_account_snapshot() -> ApiResult<Json<AccountSnapshot>> {
    Ok(Json(AccountSnapshot {
        generated_at: chrono::Utc::now().to_rfc3339(),
        total_equity: 1250000.0,
        net_exposure: 450000.0,
        brokers: vec![
            BrokerSnapshot {
                broker: "Kraken".to_string(),
                balance: 750000.0,
                open_positions: 5,
                risk_score: 0.2,
            },
            BrokerSnapshot {
                broker: "BinanceUS".to_string(),
                balance: 500000.0,
                open_positions: 3,
                risk_score: 0.3,
            },
        ],
    }))
}

pub async fn get_news_headlines() -> ApiResult<Json<Vec<NewsHeadline>>> {
    Ok(Json(vec![
        NewsHeadline {
            id: "1".to_string(),
            title: "Bitcoin breaks $100k barrier".to_string(),
            source: "Bloomberg".to_string(),
            published_at: chrono::Utc::now().to_rfc3339(),
            url: "https://bloomberg.com/crypto".to_string(),
        },
        NewsHeadline {
            id: "2".to_string(),
            title: "Fed announces surprise rate cut".to_string(),
            source: "Reuters".to_string(),
            published_at: chrono::Utc::now().to_rfc3339(),
            url: "https://reuters.com/finance".to_string(),
        },
    ]))
}

pub async fn research_sonar(
    Json(request): Json<ResearchRequest>,
) -> ApiResult<Json<ResearchResponse>> {
    Ok(Json(ResearchResponse {
        task_id: uuid::Uuid::new_v4().to_string(),
        query: request.query.clone(),
        summary: format!("Research completed for: {}", request.query),
        citations: vec![],
    }))
}

pub async fn summon_swarm(
    Json(request): Json<SwarmRequest>,
) -> ApiResult<Json<SwarmResponse>> {
    Ok(Json(SwarmResponse {
        swarm_id: uuid::Uuid::new_v4().to_string(),
        task: request.task,
        status: "active".to_string(),
        eta_seconds: 300,
    }))
}

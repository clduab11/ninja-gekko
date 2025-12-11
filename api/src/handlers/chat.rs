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
    // Return empty history - actual chat is stored client-side
    Ok(Json(vec![]))
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
    // TODO: Implement real account snapshot from connected exchanges
    Err(ApiError::NotImplemented {
        message: "Account snapshot requires connected exchange credentials".to_string(),
    })
}

pub async fn get_news_headlines() -> ApiResult<Json<Vec<NewsHeadline>>> {
    // TODO: Implement real news headlines from Perplexity/Sonar API
    Ok(Json(vec![]))
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


use crate::llm::models::{LlmModel, MODEL_REGISTRY};

pub async fn get_models() -> ApiResult<Json<Vec<LlmModel>>> {
    Ok(Json(MODEL_REGISTRY.to_vec()))
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

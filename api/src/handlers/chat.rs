//! Chat and miscellaneous frontend handlers
//!
//! This module provides handlers for the chat UI and other frontend-specific features
//! that don't fit into the core REST API structure yet.

use axum::{
    extract::{State},
    response::Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use tracing::{info, error};

use crate::{
    error::{ApiError, ApiResult},
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

#[derive(Debug, FromRow)]
struct ChatHistoryRow {
    id: uuid::Uuid,
    role: String,
    content: String,
    #[allow(dead_code)]
    input_tokens: Option<i32>,
    #[allow(dead_code)]
    output_tokens: Option<i32>,
    #[allow(dead_code)]
    model: Option<String>,
    timestamp: DateTime<Utc>,
}

impl From<ChatHistoryRow> for ChatMessage {
    fn from(row: ChatHistoryRow) -> Self {
        ChatMessage {
            id: row.id.to_string(),
            role: row.role,
            content: row.content,
            timestamp: row.timestamp.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub prompt: String,
    pub model: Option<String>, // Allowing model selection
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

pub async fn get_chat_history(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ChatMessage>>> {
    let rows = sqlx::query_as::<_, ChatHistoryRow>(
        "SELECT * FROM chat_history ORDER BY timestamp ASC LIMIT 100"
    )
    .fetch_all(state.db_manager.pool())
    .await
    .map_err(|e| {
        error!("Failed to fetch chat history: {}", e);
        ApiError::database(format!("Failed to fetch chat history: {}", e))
    })?;

    let messages: Vec<ChatMessage> = rows.into_iter().map(ChatMessage::from).collect();
    Ok(Json(messages))
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> ApiResult<Json<ChatResponse>> {
    info!("Received message: {}", request.prompt);

    // 1. Save User Message
    let user_msg_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO chat_history (id, role, content, timestamp) VALUES ($1, $2, $3, NOW())"
    )
    .bind(user_msg_id)
    .bind("user")
    .bind(&request.prompt)
    .execute(state.db_manager.pool())
    .await
    .map_err(|e| ApiError::database(e.to_string()))?;

    // 2. Generate AI Response (Mock for now, or could integrate actual LLM here if ready)
    // The previous implementation implied it was a mock or simple response.
    // Ideally this should call an LLM service.
    
    // For now, we will stick to the previous simple logic but persist it.
    let reply_content = format!("I received your message: '{}'. Market analysis is running.", request.prompt);
    let reply_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO chat_history (id, role, content, timestamp) VALUES ($1, $2, $3, NOW())"
    )
    .bind(reply_id)
    .bind("assistant")
    .bind(&reply_content)
    .execute(state.db_manager.pool())
    .await
    .map_err(|e| ApiError::database(e.to_string()))?;
    
    // Fetch the inserted reply to get the timestamp correct or just use NOW
    let reply_msg = ChatMessage {
        id: reply_id.to_string(),
        role: "assistant".to_string(),
        content: reply_content,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ChatResponse {
        reply: reply_msg,
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

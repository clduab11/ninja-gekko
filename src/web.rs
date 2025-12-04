//! Web API surface for the Ninja Gekko conversational control center.
//!
//! This lightweight Axum server exposes high-level orchestration endpoints used by the
//! forthcoming chat UI. The handlers are intentionally stubbed with in-memory state so that
//! the UI can be exercised end-to-end while the deeper trading, research, and automation
//! plumbing is implemented.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    http::Method,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};
use uuid::Uuid;

/// Composite application state shared across the HTTP handlers.
use event_bus::EventBus;
use mcp_client::McpClient;
use metrics_exporter_prometheus::PrometheusHandle;

#[derive(Clone)]
struct AppState {
    chat_history: Arc<RwLock<Vec<ChatMessage>>>,
    persona: Arc<RwLock<PersonaSettings>>,
    system_actions: Arc<RwLock<Vec<SystemAction>>>,
    event_bus: Option<EventBus>, // Optional for now to allow gradual migration
    mcp_client: Arc<McpClient>,
    prometheus_handle: PrometheusHandle,
}

impl AppState {
    fn new(
        event_bus: Option<EventBus>,
        mcp_client: McpClient,
        prometheus_handle: PrometheusHandle,
    ) -> Self {
        let mut system_actions = Vec::new();
        system_actions.push(SystemAction {
            id: Uuid::new_v4(),
            label: "Pause Trading".into(),
            description: "Immediately pause every automated execution pipeline".into(),
            action: ActionKind::PauseTrading,
        });
        system_actions.push(SystemAction {
            id: Uuid::new_v4(),
            label: "Account Snapshot".into(),
            description: "Request the most recent balance, exposure, and risk posture".into(),
            action: ActionKind::AccountSnapshot,
        });
        system_actions.push(SystemAction {
            id: Uuid::new_v4(),
            label: "Summon Swarm".into(),
            description: "Launch an agentic swarm for deep research or diagnostics".into(),
            action: ActionKind::SummonSwarm,
        });

        Self {
            chat_history: Arc::new(RwLock::new(Vec::new())),
            persona: Arc::new(RwLock::new(PersonaSettings::default())),
            system_actions: Arc::new(RwLock::new(system_actions)),
            event_bus,
            mcp_client: Arc::new(mcp_client),
            prometheus_handle,
        }
    }
}

/// Public entry-point for the web server.
pub fn spawn(
    addr: SocketAddr,
    event_bus: Option<EventBus>,
    mcp_client: McpClient,
    prometheus_handle: PrometheusHandle,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(err) = run_server(addr, event_bus, mcp_client, prometheus_handle).await {
            error!("Failed to launch chat orchestration server: {err:?}");
        }
    })
}

async fn run_server(
    addr: SocketAddr,
    event_bus: Option<EventBus>,
    mcp_client: McpClient,
    prometheus_handle: PrometheusHandle,
) -> anyhow::Result<()> {
    let state = AppState::new(event_bus, mcp_client, prometheus_handle);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics_handler))
        .route("/api/chat/history", get(chat_history))
        .route("/api/chat/message", post(post_message))
        .route("/api/chat/persona", get(get_persona).post(update_persona))
        .route("/api/actions", get(list_actions))
        .route("/api/trading/pause", post(pause_trading))
        .route("/api/accounts/snapshot", get(account_snapshot))
        .route("/api/news/headlines", get(latest_news))
        .route("/api/research/sonar", post(deep_research))
        .route("/api/agents/swarm", post(summon_swarm))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    info!("Launching chat orchestration server at {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.prometheus_handle.render();
    (
        axum::http::StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        metrics,
    )
        .into_response()
}

async fn chat_history(State(state): State<AppState>) -> Json<Vec<ChatMessage>> {
    Json(state.chat_history.read().clone())
}

async fn post_message(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let mut history = state.chat_history.write();

    let user_message = ChatMessage::new(
        ChatRole::User,
        payload.prompt.clone(),
        payload.citations.clone(),
    );
    history.push(user_message.clone());

    let persona = state.persona.read().clone();
    let reply = ChatMessage::new(
        ChatRole::Assistant,
        synthesize_response(&persona, &payload.prompt),
        Some(vec![Citation::Inline {
            source: "strategic-memory".into(),
            detail: "Synthesized from sandbox analytics".into(),
        }]),
    );
    history.push(reply.clone());

    Json(ChatResponse {
        reply,
        persona,
        actions: state.system_actions.read().clone(),
        diagnostics: vec![DiagnosticLog {
            id: Uuid::new_v4(),
            label: "Neural Forecast".into(),
            detail: "ruv-FANN ensemble suggests moderate bullish drift across ETH pairs".into(),
            severity: DiagnosticSeverity::Info,
        }],
    })
}

async fn get_persona(State(state): State<AppState>) -> Json<PersonaSettings> {
    Json(state.persona.read().clone())
}

async fn update_persona(
    State(state): State<AppState>,
    Json(payload): Json<PersonaSettings>,
) -> Json<PersonaSettings> {
    *state.persona.write() = payload.clone();
    Json(payload)
}

async fn list_actions(State(state): State<AppState>) -> Json<Vec<SystemAction>> {
    Json(state.system_actions.read().clone())
}

async fn pause_trading(Json(payload): Json<PauseTradingRequest>) -> Json<SystemAcknowledge> {
    Json(SystemAcknowledge {
        id: Uuid::new_v4(),
        message: format!(
            "Trading pipelines paused for {} hours across all venues (sandbox simulation)",
            payload.duration_hours
        ),
        status: "paused".into(),
    })
}

async fn account_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    match state.mcp_client.get_account_snapshot().await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch account snapshot: {}", e),
        )
            .into_response(),
    }
}

async fn latest_news(State(state): State<AppState>) -> impl IntoResponse {
    match state.mcp_client.get_latest_news().await {
        Ok(news) => Json(news).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch news: {}", e),
        )
            .into_response(),
    }
}

async fn deep_research(
    State(state): State<AppState>,
    Json(payload): Json<ResearchRequest>,
) -> impl IntoResponse {
    match state.mcp_client.perform_research(payload.query).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to perform research: {}", e),
        )
            .into_response(),
    }
}

async fn summon_swarm(
    State(state): State<AppState>,
    Json(payload): Json<SwarmRequest>,
) -> impl IntoResponse {
    match state.mcp_client.summon_swarm(payload.task).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to summon swarm: {}", e),
        )
            .into_response(),
    }
}

fn synthesize_response(persona: &PersonaSettings, prompt: &str) -> String {
    format!(
        "{} Gordon here — {} While the deep integrations are linking up, here's a simulated read on your ask: '{}'",
        match persona.mood.as_str() {
            "witty" => "🥂",
            "direct" => "📊",
            _ => "🧠",
        },
        match persona.tone.as_str() {
            "dramatic" => "let's keep the theatrics tight but the risk tighter.",
            "concise" => "signal extracted without the noise.",
            _ => "deploying a balanced narrative with actionable edges.",
        },
        prompt
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatRequest {
    prompt: String,
    #[serde(default)]
    citations: Option<Vec<Citation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatResponse {
    reply: ChatMessage,
    persona: PersonaSettings,
    actions: Vec<SystemAction>,
    diagnostics: Vec<DiagnosticLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiagnosticLog {
    id: Uuid,
    label: String,
    detail: String,
    severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DiagnosticSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    id: Uuid,
    role: ChatRole,
    content: String,
    timestamp: DateTime<Utc>,
    #[serde(default)]
    citations: Option<Vec<Citation>>,
}

impl ChatMessage {
    fn new(role: ChatRole, content: String, citations: Option<Vec<Citation>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role,
            content,
            timestamp: Utc::now(),
            citations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Citation {
    Inline { source: String, detail: String },
    External { title: String, url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonaSettings {
    #[serde(default = "PersonaSettings::default_tone")]
    tone: String,
    #[serde(default = "PersonaSettings::default_style")]
    style: String,
    #[serde(default = "PersonaSettings::default_mood")]
    mood: String,
}

impl Default for PersonaSettings {
    fn default() -> Self {
        Self {
            tone: Self::default_tone(),
            style: Self::default_style(),
            mood: Self::default_mood(),
        }
    }
}

impl PersonaSettings {
    fn default_tone() -> String {
        "balanced".into()
    }

    fn default_style() -> String {
        "analytical".into()
    }

    fn default_mood() -> String {
        "direct".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemAction {
    id: Uuid,
    label: String,
    description: String,
    action: ActionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ActionKind {
    PauseTrading,
    AccountSnapshot,
    SummonSwarm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PauseTradingRequest {
    duration_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemAcknowledge {
    id: Uuid,
    message: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountSnapshot {
    generated_at: DateTime<Utc>,
    total_equity: f64,
    net_exposure: f64,
    brokers: Vec<BrokerSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrokerSnapshot {
    broker: String,
    balance: f64,
    open_positions: u64,
    risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewsHeadline {
    id: Uuid,
    title: String,
    source: String,
    published_at: DateTime<Utc>,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResearchRequest {
    query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResearchResponse {
    task_id: Uuid,
    query: String,
    summary: String,
    citations: Vec<Citation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SwarmRequest {
    task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SwarmResponse {
    swarm_id: Uuid,
    task: String,
    status: String,
    eta_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_defaults_are_predictable() {
        let persona = PersonaSettings::default();
        assert_eq!(persona.tone, "balanced");
        assert_eq!(persona.style, "analytical");
        assert_eq!(persona.mood, "direct");
    }

    #[test]
    fn synthesize_response_respects_prompt() {
        let persona = PersonaSettings::default();
        let response = synthesize_response(&persona, "status report");
        assert!(response.contains("status report"));
    }
}

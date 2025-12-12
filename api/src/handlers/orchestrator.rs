use crate::models::ApiResponse;
use crate::AppState;
use axum::{extract::State, response::Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum OrchestratorCommand {
    Engage,
    WindDown { duration_seconds: u64 },
    EmergencyHalt { reason: String },
    SetRiskThrottle { value: f64 }, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorState {
    pub is_live: bool,
    pub is_winding_down: bool,
    pub wind_down_started_at: Option<DateTime<Utc>>,
    pub emergency_halt_active: bool,
    pub emergency_halt_reason: Option<String>,
    pub risk_throttle: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            is_live: false,
            is_winding_down: false,
            wind_down_started_at: None,
            emergency_halt_active: false,
            emergency_halt_reason: None,
            risk_throttle: 1.0, // 100% by default
            last_updated: Utc::now(),
        }
    }
}

/// Engage the trading system
pub async fn engage(State(state): State<Arc<AppState>>) -> Json<ApiResponse<OrchestratorState>> {
    info!("Orchestrator: ENGAGE command received");

    let mut orchestrator = state.orchestrator_state.write().await;

    // Clear emergency halt state on engage (allows recovery)
    if orchestrator.emergency_halt_active {
        info!("Orchestrator: Clearing emergency halt state on ENGAGE");
        orchestrator.emergency_halt_active = false;
        orchestrator.emergency_halt_reason = None;
    }

    orchestrator.is_live = true;
    orchestrator.is_winding_down = false;
    orchestrator.wind_down_started_at = None;
    orchestrator.risk_throttle = 1.0; // Reset throttle to 100%
    orchestrator.last_updated = Utc::now();

    info!("Orchestrator: System ENGAGED - Trading is now LIVE");

    Json(ApiResponse::success(orchestrator.clone()))
}

/// Wind down trading gracefully over a specified duration
pub async fn wind_down(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrchestratorCommand>,
) -> Json<ApiResponse<OrchestratorState>> {
    let duration = match payload {
        OrchestratorCommand::WindDown { duration_seconds } => duration_seconds,
        _ => 3600, // Default 1 hour
    };

    warn!("Orchestrator: WIND DOWN command received ({}s)", duration);

    let mut orchestrator = state.orchestrator_state.write().await;

    orchestrator.is_winding_down = true;
    orchestrator.wind_down_started_at = Some(Utc::now());
    orchestrator.last_updated = Utc::now();

    info!("Orchestrator: Winding down over {} seconds", duration);

    Json(ApiResponse::success(orchestrator.clone()))
}

/// Emergency halt - immediately stop all trading
pub async fn emergency_halt(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrchestratorCommand>,
) -> Json<ApiResponse<OrchestratorState>> {
    let reason = match payload {
        OrchestratorCommand::EmergencyHalt { reason } => reason,
        _ => "Unknown".to_string(),
    };

    error!("Orchestrator: EMERGENCY HALT triggered: {}", reason);

    let mut orchestrator = state.orchestrator_state.write().await;

    // Full system halt
    orchestrator.is_live = false;
    orchestrator.is_winding_down = false;
    orchestrator.wind_down_started_at = None;
    orchestrator.emergency_halt_active = true;
    orchestrator.emergency_halt_reason = Some(reason.clone());
    orchestrator.risk_throttle = 0.0;
    orchestrator.last_updated = Utc::now();

    // TODO: Disconnect all exchange connections via websocket_manager
    // state.websocket_manager.disconnect_all_exchanges().await;

    error!("Orchestrator: EMERGENCY HALT ACTIVE - All trading STOPPED");

    Json(ApiResponse::success(orchestrator.clone()))
}

/// Set risk throttle (0.0 to 1.0)
pub async fn risk_throttle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrchestratorCommand>,
) -> Json<ApiResponse<OrchestratorState>> {
    let value = match payload {
        OrchestratorCommand::SetRiskThrottle { value } => value.clamp(0.0, 1.0),
        _ => 1.0,
    };

    info!("Orchestrator: Risk throttle set to {:.1}%", value * 100.0);

    let mut orchestrator = state.orchestrator_state.write().await;
    orchestrator.risk_throttle = value;
    orchestrator.last_updated = Utc::now();

    Json(ApiResponse::success(orchestrator.clone()))
}

/// Get current orchestrator state
pub async fn get_state(State(state): State<Arc<AppState>>) -> Json<ApiResponse<OrchestratorState>> {
    let orchestrator = state.orchestrator_state.read().await;
    Json(ApiResponse::success(orchestrator.clone()))
}

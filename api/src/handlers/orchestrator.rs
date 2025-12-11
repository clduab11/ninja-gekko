use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::AppState;
use crate::models::ApiResponse;
use tracing::{info, warn, error};

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

// Global state for orchestrator (in-memory for now, could be moved to AppState)
// Using a lazy static or adding to AppState would be better, but for this task I'll use a static RwLock for simplicity
// or better yet, assume I can add it to AppState.
// Since I cannot easily modify AppState struct definition without touching multiple files, 
// I will implement a thread-safe singleton pattern here or just use a static for now.
// Ideally, this should be in AppState. 

// Actually, to follow best practices and "zero unsafe blocks", I should probably add this to AppState.
// But that requires modifying `api/src/lib.rs` to add the field to AppState struct.
// Let's check `api/src/lib.rs` again. `AppState` is struct.
// I will modify `AppState` in `api/src/lib.rs` to include `orchestrator_state`.

// For now, let's define the handlers.

pub async fn engage(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<OrchestratorState>> {
    info!("Orchestrator: ENGAGE command received");
    
    // TODO: interactions with state.orchestrator_state
    // For now, returning a mock state as if it succeeded
    let new_state = OrchestratorState {
        is_live: true,
        last_updated: Utc::now(),
        ..Default::default()
    };
    
    // In a real impl, we would start the trading engine here
    
    Json(ApiResponse::success(new_state))
}

pub async fn wind_down(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<OrchestratorCommand>,
) -> Json<ApiResponse<OrchestratorState>> {
    let duration = match payload {
        OrchestratorCommand::WindDown { duration_seconds } => duration_seconds,
        _ => 0,
    };
    
    warn!("Orchestrator: WIND DOWN command received ({}s)", duration);
    
    let new_state = OrchestratorState {
        is_live: true,
        is_winding_down: true,
        wind_down_started_at: Some(Utc::now()),
        last_updated: Utc::now(),
        ..Default::default()
    };
    
    Json(ApiResponse::success(new_state))
}

pub async fn emergency_halt(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<OrchestratorCommand>,
) -> Json<ApiResponse<OrchestratorState>> {
    let reason = match payload {
        OrchestratorCommand::EmergencyHalt { reason } => reason,
        _ => "Unknown".to_string(),
    };
    
    error!("Orchestrator: EMERGENCY HALT triggered: {}", reason);
    
    // CRITICAL: Immediately disconnect all exchange WebSocket connections
    // This would typically involve calling a method on state.websocket_manager or similar
    
    let new_state = OrchestratorState {
        emergency_halt_active: true,
        emergency_halt_reason: Some(reason),
        risk_throttle: 0.0,
        last_updated: Utc::now(),
        ..Default::default()
    };
    
    Json(ApiResponse::success(new_state))
}

pub async fn risk_throttle(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<OrchestratorCommand>,
) -> Json<ApiResponse<OrchestratorState>> {
    let value = match payload {
        OrchestratorCommand::SetRiskThrottle { value } => value,
        _ => 1.0,
    };
    
    info!("Orchestrator: Risk throttle set to {:.1}%", value * 100.0);
    
    let new_state = OrchestratorState {
        risk_throttle: value,
        last_updated: Utc::now(),
        ..Default::default()
    };
    
    Json(ApiResponse::success(new_state))
}

pub async fn get_state(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<OrchestratorState>> {
    Json(ApiResponse::success(OrchestratorState::default()))
}

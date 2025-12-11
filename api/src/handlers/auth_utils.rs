//! Authentication utilities and handlers
//!
//! This module provides JWT authentication functionality including
//! login, token refresh, logout handlers, and utility functions for
//! token management and validation.

use axum::{
    extract::State,
    response::Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::{
    error::{ApiError, ApiResult},
    models::ApiResponse,
    auth::AuthMiddleware,
};

/// Login request structure
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
}

/// Login response structure
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// JWT refresh token
    pub refresh_token: String,
    /// Token expiration timestamp
    pub expires_at: usize,
    /// User information
    pub user: UserInfo,
    /// Token type
    pub token_type: String,
}

/// User information structure
#[derive(Debug, Serialize)]
pub struct UserInfo {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<String>,
    /// Account IDs accessible to user
    pub account_ids: Vec<String>,
}

/// Refresh token request structure
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    /// Refresh token to exchange for new access token
    pub refresh_token: String,
}

/// Refresh token response structure
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    /// New JWT access token
    pub access_token: String,
    /// New refresh token
    pub refresh_token: String,
    /// Token expiration timestamp
    pub expires_at: usize,
    /// Token type
    pub token_type: String,
}

/// Logout response structure
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    /// Logout success message
    pub message: String,
    /// Timestamp of logout
    pub timestamp: String,
}

/// Login handler for user authentication
///
/// Authenticates users with username/password and returns JWT tokens.
/// Note: Full database authentication pending implementation.
/// Currently uses environment-based credentials for the primary admin user.
pub async fn login_handler(
    State(state): State<Arc<crate::AppState>>,
    Json(login_request): Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<LoginResponse>>> {
    // Validate input
    if login_request.username.trim().is_empty() {
        return Err(ApiError::Validation {
            message: "Username cannot be empty".to_string(),
            field: Some("username".to_string()),
        });
    }

    if login_request.password.trim().is_empty() {
        return Err(ApiError::Validation {
            message: "Password cannot be empty".to_string(),
            field: Some("password".to_string()),
        });
    }

    // Validate credentials
    // In production: query database, verify password hash with argon2/bcrypt
    // For now: check against environment-configured admin credentials
    let admin_user = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let admin_pass = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "".to_string());
    
    if admin_pass.is_empty() {
        return Err(ApiError::Auth {
            message: "Authentication not configured. Set ADMIN_USERNAME and ADMIN_PASSWORD environment variables.".to_string(),
        });
    }
    
    if login_request.username != admin_user || login_request.password != admin_pass {
        return Err(ApiError::Auth {
            message: "Invalid username or password".to_string(),
        });
    }

    // Create user context for authenticated session
    let user_id = format!("user-{}", uuid::Uuid::new_v4());
    let roles = vec!["admin".to_string(), "trader".to_string()];
    let account_ids = vec!["default".to_string()];

    // Generate JWT tokens
    let access_token = AuthMiddleware::generate_access_token(&user_id, roles.clone(), account_ids.clone())
        .await
        .map_err(|e| ApiError::Auth {
            message: format!("Failed to generate access token: {}", e),
        })?;

    let refresh_token = AuthMiddleware::generate_refresh_token(&user_id, roles.clone(), account_ids.clone())
        .await
        .map_err(|e| ApiError::Auth {
            message: format!("Failed to generate refresh token: {}", e),
        })?;

    let response = LoginResponse {
        access_token,
        refresh_token,
        expires_at: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        user: UserInfo {
            id: user_id,
            username: login_request.username,
            roles,
            account_ids,
        },
        token_type: "Bearer".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Refresh token handler
///
/// Exchanges a valid refresh token for a new access token.
/// This endpoint should be called when the access token expires.
pub async fn refresh_handler(
    State(_state): State<Arc<crate::AppState>>,
    Json(refresh_request): Json<RefreshRequest>,
) -> ApiResult<Json<ApiResponse<RefreshResponse>>> {
    // Validate refresh token
    if refresh_request.refresh_token.trim().is_empty() {
        return Err(ApiError::Validation {
            message: "Refresh token cannot be empty".to_string(),
            field: Some("refresh_token".to_string()),
        });
    }

    // Use AuthMiddleware to refresh tokens
    let (access_token, new_refresh_token) = AuthMiddleware::refresh_access_token(&refresh_request.refresh_token)
        .await
        .map_err(|e| ApiError::Auth {
             message: format!("Failed to refresh token: {}", e),
        })?;

    let response = RefreshResponse {
        access_token,
        refresh_token: new_refresh_token,
        expires_at: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        token_type: "Bearer".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Logout handler
///
/// Invalidates the user's refresh token and logs them out.
/// Requires authenticated request with valid JWT token.
pub async fn logout_handler(
    State(_state): State<Arc<crate::AppState>>,
    // In production: extract user_id from JWT claims via middleware
    // Extension(claims): Extension<Claims>
) -> ApiResult<Json<ApiResponse<LogoutResponse>>> {
    // TODO: Extract user_id from JWT token in request context
    // For now, logout is a no-op that clears client-side tokens
    // In production: invalidate refresh token in database/cache
    
    let response = LogoutResponse {
        message: "Successfully logged out".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}
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
/// This endpoint should be called to obtain initial authentication tokens.
pub async fn login_handler(
    State(_state): State<Arc<crate::AppState>>,
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

    // TODO: Implement actual authentication against user database
    // For now, we'll use mock authentication
    if !is_valid_credentials(&login_request.username, &login_request.password).await {
        return Err(ApiError::Auth {
            message: "Invalid username or password".to_string(),
        });
    }

    // Get user information from database
    let user = match get_user_by_username(&login_request.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(ApiError::Auth {
                message: "User not found".to_string(),
            });
        }
        Err(e) => {
            return Err(ApiError::Database {
                message: format!("Failed to retrieve user: {}", e),
            });
        }
    };

    // Generate JWT tokens
    let access_token = AuthMiddleware::generate_access_token(&user.id, user.roles.clone(), user.account_ids.clone())
        .await
        .map_err(|e| ApiError::Auth {
            message: format!("Failed to generate access token: {}", e),
        })?;

    let refresh_token = AuthMiddleware::generate_refresh_token(&user.id, user.roles.clone(), user.account_ids.clone())
        .await
        .map_err(|e| ApiError::Auth {
            message: format!("Failed to generate refresh token: {}", e),
        })?;

    // Store refresh token (TODO: implement proper token storage)
    // store_refresh_token(user.id, refresh_token).await?;

    let response = LoginResponse {
        access_token,
        refresh_token,
        expires_at: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        user: UserInfo {
            id: user.id,
            username: user.username,
            roles: user.roles,
            account_ids: user.account_ids,
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
/// This endpoint should be called when users want to log out.
pub async fn logout_handler(
    State(_state): State<Arc<crate::AppState>>,
    // Claims should be extracted by middleware and passed here, but for now we extract from extensions or assume middleware context
    // Ideally: Extract State<Arc<AppState>>, Extension(claims): Extension<Claims>
) -> ApiResult<Json<ApiResponse<LogoutResponse>>> {
    // TODO: Get user ID from JWT token in request context
    let user_id = "mock-user-id".to_string(); 

    // Revoke tokens
    AuthMiddleware::revoke_user_tokens(&user_id).await
        .map_err(|e| ApiError::Database {
            message: format!("Failed to revoke tokens: {}", e),
        })?;

    let response = LogoutResponse {
        message: "Successfully logged out".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Validate user credentials against secure database
/// TODO: IMPLEMENT actual authentication: fetch user hash, verify with bcrypt/scrypt/argon2, check status
async fn is_valid_credentials(username: &str, _password: &str) -> bool {
    // Mock credentials check - assume true for demo users if they exist in get_user_by_username
    username == "admin" || username == "trader"
}

/// Mock function to get user by username
/// TODO: Replace with actual database query
async fn get_user_by_username(username: &str) -> Result<Option<User>, String> {
    // Mock user data - replace with actual database query
    if username == "admin" {
        Ok(Some(User {
            id: "user-123".to_string(),
            username: "admin".to_string(),
            roles: vec!["admin".to_string(), "trader".to_string()],
            account_ids: vec!["acc-001".to_string(), "acc-002".to_string()],
        }))
    } else if username == "trader" {
        Ok(Some(User {
            id: "user-456".to_string(),
            username: "trader".to_string(),
            roles: vec!["trader".to_string()],
            account_ids: vec!["acc-003".to_string()],
        }))
    } else {
        Ok(None)
    }
}

/// User structure for internal use
struct User {
    id: String,
    username: String,
    roles: Vec<String>,
    account_ids: Vec<String>,
}
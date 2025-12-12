//! JWT-based authentication middleware
//!
//! This module provides JWT token validation, generation, and middleware integration
//! for securing API endpoints with stateless authentication and authorization.

use axum::{extract::Request, http::header, middleware::Next, response::IntoResponse};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{ApiError, ApiResult};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at timestamp
    pub iat: usize,
    /// Expiration timestamp
    pub exp: usize,
    /// Issuer
    pub iss: Option<String>,
    /// Audience
    pub aud: Option<String>,
    /// User roles/permissions
    pub roles: Vec<String>,
    /// Account IDs the user has access to
    pub account_ids: Vec<String>,
    /// Token type
    pub token_type: TokenType,
}

/// Token types for different use cases
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType {
    /// Access token for API authentication
    Access,
    /// Refresh token for token renewal
    Refresh,
}

/// JWT Authentication middleware
pub struct AuthMiddleware;

impl AuthMiddleware {
    /// Create a new authentication middleware instance
    pub fn new() -> Self {
        Self
    }

    /// Authenticate request using JWT token from Authorization header or cookies
    pub async fn authenticate(
        cookie_jar: CookieJar,
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Extract token from Authorization header or cookies
        let token = Self::extract_token(&cookie_jar, &request);

        match token {
            Some(token) => {
                match Self::validate_token(&token).await {
                    Ok(claims) => {
                        info!("Authentication successful for user: {}", claims.sub);

                        // Add claims to request extensions for later use
                        let mut request = request;
                        request.extensions_mut().insert(claims);

                        // Continue to next middleware/handler
                        next.run(request).await
                    }
                    Err(e) => {
                        warn!("Token validation failed: {}", e);
                        ApiError::Auth {
                            message: format!("Invalid token: {}", e),
                        }
                        .into_response()
                    }
                }
            }
            None => {
                warn!("No authentication token provided");
                ApiError::Auth {
                    message: "Authentication required".to_string(),
                }
                .into_response()
            }
        }
    }

    /// Extract JWT token from request (Authorization header or cookies)
    fn extract_token(cookie_jar: &CookieJar, request: &Request) -> Option<String> {
        // Try Authorization header first
        if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    return Some(auth_str[7..].to_string());
                }
            }
        }

        // Try cookies as fallback
        if let Some(cookie) = cookie_jar.get("access_token") {
            return Some(cookie.value().to_string());
        }

        None
    }

    /// Validate JWT token and return claims
    async fn validate_token(token: &str) -> ApiResult<Claims> {
        let decoding_key = DecodingKey::from_secret(Self::get_jwt_secret().as_ref());
        let mut validation = Validation::default();
        validation.set_audience(&["trading-platform"]);

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                // Check if token is expired
                let now = Utc::now().timestamp() as usize;
                if token_data.claims.exp < now {
                    return Err(ApiError::Auth {
                        message: "Token expired".to_string(),
                    });
                }

                Ok(token_data.claims)
            }
            Err(e) => Err(ApiError::Auth {
                message: format!("Token validation failed: {}", e),
            }),
        }
    }

    /// Generate new access token
    pub async fn generate_access_token(
        user_id: &str,
        roles: Vec<String>,
        account_ids: Vec<String>,
    ) -> ApiResult<String> {
        let claims = Claims {
            sub: user_id.to_string(),
            iat: Utc::now().timestamp() as usize,
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize, // 1 hour expiry
            iss: Some("ninja-gekko-api".to_string()),
            aud: Some("trading-platform".to_string()),
            roles,
            account_ids,
            token_type: TokenType::Access,
        };

        let encoding_key = EncodingKey::from_secret(Self::get_jwt_secret().as_ref());
        let token = encode(&Header::default(), &claims, &encoding_key)?;

        Ok(token)
    }

    /// Generate new refresh token
    pub async fn generate_refresh_token(
        user_id: &str,
        roles: Vec<String>,
        account_ids: Vec<String>,
    ) -> ApiResult<String> {
        let claims = Claims {
            sub: user_id.to_string(),
            iat: Utc::now().timestamp() as usize,
            exp: (Utc::now() + Duration::days(30)).timestamp() as usize, // 30 days expiry
            iss: Some("ninja-gekko-api".to_string()),
            aud: Some("trading-platform".to_string()),
            roles,
            account_ids,
            token_type: TokenType::Refresh,
        };

        let encoding_key = EncodingKey::from_secret(Self::get_refresh_secret().as_ref());
        let token = encode(&Header::default(), &claims, &encoding_key)?;

        Ok(token)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_access_token(refresh_token: &str) -> ApiResult<(String, String)> {
        // Validate refresh token
        let refresh_claims = Self::validate_refresh_token(refresh_token).await?;

        // Generate new access token
        let access_token = Self::generate_access_token(
            &refresh_claims.sub,
            refresh_claims.roles.clone(),
            refresh_claims.account_ids.clone(),
        )
        .await?;

        // Generate new refresh token
        let new_refresh_token = Self::generate_refresh_token(
            &refresh_claims.sub,
            refresh_claims.roles,
            refresh_claims.account_ids,
        )
        .await?;

        Ok((access_token, new_refresh_token))
    }

    /// Validate refresh token
    async fn validate_refresh_token(refresh_token: &str) -> ApiResult<Claims> {
        let decoding_key = DecodingKey::from_secret(Self::get_refresh_secret().as_ref());
        let validation = Validation::default();

        match decode::<Claims>(refresh_token, &decoding_key, &validation) {
            Ok(token_data) => {
                // Check if token is expired
                let now = Utc::now().timestamp() as usize;
                if token_data.claims.exp < now {
                    return Err(ApiError::Auth {
                        message: "Refresh token expired".to_string(),
                    });
                }

                // Verify it's a refresh token
                if token_data.claims.token_type != TokenType::Refresh {
                    return Err(ApiError::Auth {
                        message: "Invalid token type".to_string(),
                    });
                }

                Ok(token_data.claims)
            }
            Err(e) => Err(ApiError::Auth {
                message: format!("Refresh token validation failed: {}", e),
            }),
        }
    }

    /// Get JWT secret from environment
    fn get_jwt_secret() -> String {
        std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            if cfg!(test) {
                "test-secret-at-least-32-characters-long-for-testing".to_string()
            } else {
                panic!("JWT_SECRET environment variable must be set")
            }
        })
    }

    /// Get refresh token secret from environment
    fn get_refresh_secret() -> String {
        std::env::var("JWT_REFRESH_SECRET").unwrap_or_else(|_| {
            if cfg!(test) {
                "test-refresh-secret-at-least-32-characters-long".to_string()
            } else {
                panic!("JWT_REFRESH_SECRET environment variable must be set")
            }
        })
    }

    /// Revoke a user's tokens (logout)
    pub async fn revoke_user_tokens(user_id: &str) -> ApiResult<()> {
        // In a production system, you would store revoked tokens in a cache/database
        // For this example, we'll just log the revocation
        info!("Token revocation requested for user: {}", user_id);

        // Here you would:
        // 1. Add the user's tokens to a revocation list
        // 2. Clear any cached user sessions
        // 3. Log the logout event

        Ok(())
    }

    /// Check if user has required role
    pub fn has_role(claims: &Claims, required_role: &str) -> bool {
        claims.roles.contains(&required_role.to_string())
    }

    /// Check if user has access to specific account
    pub fn has_account_access(claims: &Claims, account_id: &str) -> bool {
        claims.account_ids.contains(&account_id.to_string()) ||
        claims.account_ids.contains(&"*".to_string()) || // Wildcard access
        claims.roles.contains(&"admin".to_string()) // Admin override
    }

    /// Check if user has any of the required permissions
    pub fn has_any_permission(claims: &Claims, required_permissions: &[&str]) -> bool {
        required_permissions
            .iter()
            .any(|perm| claims.roles.contains(&perm.to_string()))
    }

    /// Check if user has all required permissions
    pub fn has_all_permissions(claims: &Claims, required_permissions: &[&str]) -> bool {
        required_permissions
            .iter()
            .all(|perm| claims.roles.contains(&perm.to_string()))
    }
}

/// Authorization middleware for role-based access control
pub struct AuthorizationMiddleware;

impl AuthorizationMiddleware {
    /// Require specific role for endpoint access
    pub async fn require_role(
        claims: Claims,
        request: Request,
        next: Next,
        required_role: &str,
    ) -> impl IntoResponse {
        if !AuthMiddleware::has_role(&claims, required_role) {
            warn!(
                "Access denied for user {} - missing role: {}",
                claims.sub, required_role
            );
            return ApiError::Auth {
                message: format!("Required role: {}", required_role),
            }
            .into_response();
        }

        let mut request = request;
        request.extensions_mut().insert(claims);
        next.run(request).await
    }

    /// Require access to specific account
    pub async fn require_account_access(
        claims: Claims,
        request: Request,
        next: Next,
        account_id: &str,
    ) -> impl IntoResponse {
        if !AuthMiddleware::has_account_access(&claims, account_id) {
            warn!(
                "Access denied for user {} - no access to account: {}",
                claims.sub, account_id
            );
            return ApiError::Auth {
                message: format!("Access denied to account: {}", account_id),
            }
            .into_response();
        }

        let mut request = request;
        request.extensions_mut().insert(claims);
        next.run(request).await
    }

    /// Require specific permissions
    pub async fn require_permissions(
        claims: Claims,
        request: Request,
        next: Next,
        required_permissions: &[&str],
    ) -> impl IntoResponse {
        if !AuthMiddleware::has_all_permissions(&claims, required_permissions) {
            warn!(
                "Access denied for user {} - missing permissions: {:?}",
                claims.sub, required_permissions
            );
            return ApiError::Auth {
                message: format!("Required permissions: {:?}", required_permissions),
            }
            .into_response();
        }

        let mut request = request;
        request.extensions_mut().insert(claims);
        next.run(request).await
    }
}

/// Utility functions for authentication
pub mod auth_utils {
    use super::*;
    use axum::response::Json;
    use serde_json::json;

    /// Login response structure
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LoginResponse {
        pub access_token: String,
        pub refresh_token: String,
        pub token_type: String,
        pub expires_in: i64,
        pub user_id: String,
        pub roles: Vec<String>,
        pub account_ids: Vec<String>,
    }

    /// Login request structure
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
        pub account_id: Option<String>,
    }

    /// Refresh token request structure
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RefreshRequest {
        pub refresh_token: String,
    }

    /// Mock authentication function (replace with real implementation)
    pub async fn authenticate_user(
        request: &LoginRequest,
    ) -> ApiResult<(String, Vec<String>, Vec<String>)> {
        // This is a mock implementation - replace with real authentication logic
        if request.username == "admin" && request.password == "password" {
            Ok((
                "admin-user-id".to_string(),
                vec!["admin".to_string(), "trader".to_string()],
                vec!["*".to_string()], // Wildcard access to all accounts
            ))
        } else if request.username == "trader" && request.password == "password" {
            Ok((
                "trader-user-id".to_string(),
                vec!["trader".to_string()],
                vec!["account-123".to_string(), "account-456".to_string()],
            ))
        } else {
            Err(ApiError::Auth {
                message: "Invalid credentials".to_string(),
            })
        }
    }

    /// Handle login endpoint
    pub async fn login_handler(
        Json(request): Json<LoginRequest>,
    ) -> ApiResult<Json<LoginResponse>> {
        // Authenticate user (mock implementation)
        let (user_id, roles, account_ids) = authenticate_user(&request).await?;

        // Generate tokens
        let access_token =
            AuthMiddleware::generate_access_token(&user_id, roles.clone(), account_ids.clone())
                .await?;
        let refresh_token =
            AuthMiddleware::generate_refresh_token(&user_id, roles.clone(), account_ids.clone())
                .await?;

        let response = LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600, // 1 hour
            user_id: user_id.clone(),
            roles,
            account_ids,
        };

        info!("User logged in successfully: {}", user_id);
        Ok(Json(response))
    }

    /// Handle refresh token endpoint
    pub async fn refresh_handler(
        Json(request): Json<RefreshRequest>,
    ) -> ApiResult<Json<LoginResponse>> {
        let (access_token, refresh_token) =
            AuthMiddleware::refresh_access_token(&request.refresh_token).await?;

        // Decode refresh token to get user info (simplified)
        let refresh_claims = AuthMiddleware::validate_refresh_token(&request.refresh_token).await?;

        let response = LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600, // 1 hour
            user_id: refresh_claims.sub,
            roles: refresh_claims.roles,
            account_ids: refresh_claims.account_ids,
        };

        info!("Token refreshed successfully");
        Ok(Json(response))
    }

    /// Handle logout endpoint
    pub async fn logout_handler(claims: Claims) -> ApiResult<Json<serde_json::Value>> {
        // Revoke user's tokens
        AuthMiddleware::revoke_user_tokens(&claims.sub).await?;

        let response = json!({
            "message": "Logged out successfully",
            "user_id": claims.sub
        });

        info!("User logged out: {}", claims.sub);
        Ok(Json(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_generation() {
        let token = AuthMiddleware::generate_access_token(
            "test-user",
            vec!["trader".to_string()],
            vec!["account-123".to_string()],
        )
        .await
        .unwrap();

        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_token_validation() {
        let token = AuthMiddleware::generate_access_token(
            "test-user",
            vec!["trader".to_string()],
            vec!["account-123".to_string()],
        )
        .await
        .unwrap();

        let claims = AuthMiddleware::validate_token(&token).await.unwrap();

        assert_eq!(claims.sub, "test-user");
        assert!(claims.roles.contains(&"trader".to_string()));
    }

    #[tokio::test]
    async fn test_role_checking() {
        let claims = Claims {
            sub: "test-user".to_string(),
            iat: Utc::now().timestamp() as usize,
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
            iss: Some("test".to_string()),
            aud: Some("test".to_string()),
            roles: vec!["trader".to_string(), "user".to_string()],
            account_ids: vec!["account-123".to_string()],
            token_type: TokenType::Access,
        };

        assert!(AuthMiddleware::has_role(&claims, "trader"));
        assert!(!AuthMiddleware::has_role(&claims, "admin"));
        assert!(AuthMiddleware::has_account_access(&claims, "account-123"));
        assert!(!AuthMiddleware::has_account_access(&claims, "account-456"));
    }
}

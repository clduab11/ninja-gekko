//! Centralized authentication and authorization validation
//!
//! This module provides comprehensive JWT validation, authorization checks,
//! and secure authentication middleware for the Ninja Gekko API.

use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Algorithm, Validation, TokenData};
use chrono::{DateTime, Utc, Duration};
use std::fmt;
use crate::validation::{SecurityValidator, SanitizationLevel};

/// JWT Claims structure for authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID
    pub sub: String,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<String>,
    /// User permissions
    pub permissions: Vec<String>,
    /// Account IDs the user has access to
    pub account_ids: Vec<String>,
    /// Issued at timestamp
    pub iat: usize,
    /// Expiration timestamp
    pub exp: usize,
    /// Token type
    pub token_type: String,
    /// Session ID for tracking
    pub session_id: String,
}

/// JWT configuration for token management
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    pub secret: String,
    /// Token expiration time in seconds
    pub expiration: i64,
    /// Refresh token expiration in seconds
    pub refresh_expiration: i64,
    /// Issuer name
    pub issuer: String,
    /// Audience
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                eprintln!("WARNING: Using default JWT secret - set JWT_SECRET environment variable!");
                "default-secret-change-in-production".to_string()
            }),
            expiration: 3600, // 1 hour
            refresh_expiration: 86400 * 7, // 7 days
            issuer: "ninja-gekko-api".to_string(),
            audience: "ninja-gekko-client".to_string(),
        }
    }
}

/// Authentication error types
#[derive(Debug, Clone)]
pub enum AuthError {
    InvalidToken(String),
    ExpiredToken(String),
    InsufficientPermissions(String),
    InvalidCredentials(String),
    AccountAccessDenied(String),
    MalformedToken(String),
    InvalidSignature(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::ExpiredToken(msg) => write!(f, "Expired token: {}", msg),
            AuthError::InsufficientPermissions(msg) => write!(f, "Insufficient permissions: {}", msg),
            AuthError::InvalidCredentials(msg) => write!(f, "Invalid credentials: {}", msg),
            AuthError::AccountAccessDenied(msg) => write!(f, "Account access denied: {}", msg),
            AuthError::MalformedToken(msg) => write!(f, "Malformed token: {}", msg),
            AuthError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// Authentication context for request processing
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub account_ids: Vec<String>,
    pub session_id: String,
    pub token_type: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl AuthContext {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Check if user has access to an account
    pub fn has_account_access(&self, account_id: &str) -> bool {
        self.account_ids.contains(&account_id.to_string()) || self.account_ids.contains(&"*".to_string())
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin") || self.has_role("administrator")
    }

    /// Check if user is trader
    pub fn is_trader(&self) -> bool {
        self.has_role("trader") || self.has_role("user")
    }

    /// Get user's highest role level
    pub fn get_role_level(&self) -> i32 {
        if self.is_admin() { 100 }
        else if self.has_role("trader") { 50 }
        else if self.has_role("viewer") { 25 }
        else if self.has_role("user") { 10 }
        else { 0 }
    }
}

/// Authorization levels for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationLevel {
    /// Public access - no authentication required
    Public,
    /// User-level access - basic authentication
    User,
    /// Trader access - trading permissions required
    Trader,
    /// Admin access - administrative permissions required
    Admin,
}

impl AuthorizationLevel {
    /// Get the minimum role level required
    pub fn required_role_level(&self) -> i32 {
        match self {
            AuthorizationLevel::Public => 0,
            AuthorizationLevel::User => 10,
            AuthorizationLevel::Trader => 50,
            AuthorizationLevel::Admin => 100,
        }
    }

    /// Check if the user meets the authorization level
    pub fn check_access(&self, context: &AuthContext) -> Result<(), AuthError> {
        if context.get_role_level() >= self.required_role_level() {
            Ok(())
        } else {
            Err(AuthError::InsufficientPermissions(
                format!("Required level: {}, User level: {}",
                    self.required_role_level(),
                    context.get_role_level()
                )
            ))
        }
    }
}

/// Authentication validator for JWT and authorization
pub struct AuthValidator {
    config: JwtConfig,
    security_validator: SecurityValidator,
}

impl AuthValidator {
    /// Create new authentication validator
    pub fn new(config: JwtConfig) -> Self {
        Self {
            config,
            security_validator: SecurityValidator::new(),
        }
    }

    /// Create validator with default configuration
    pub fn default() -> Self {
        Self::new(JwtConfig::default())
    }

    /// Validate JWT token and extract claims
    pub fn validate_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        // Sanitize token input
        let clean_token = self.security_validator
            .validate_string(token, "token", SanitizationLevel::Strict)
            .map_err(|_| AuthError::MalformedToken("Token contains invalid characters".to_string()))?;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<Claims>(
            &clean_token,
            &DecodingKey::from_secret(self.config.secret.as_ref()),
            &validation,
        ).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AuthError::ExpiredToken("Token has expired".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                AuthError::InvalidSignature("Invalid token signature".to_string())
            }
            _ => AuthError::InvalidToken(format!("Token validation failed: {}", e)),
        })?;

        Ok(token_data)
    }

    /// Convert token data to authentication context
    pub fn token_to_context(&self, token_data: TokenData<Claims>) -> AuthContext {
        AuthContext {
            user_id: token_data.claims.sub,
            username: token_data.claims.username,
            roles: token_data.claims.roles,
            permissions: token_data.claims.permissions,
            account_ids: token_data.claims.account_ids,
            session_id: token_data.claims.session_id,
            token_type: token_data.claims.token_type,
            issued_at: DateTime::from_timestamp(token_data.claims.iat as i64, 0)
                .unwrap_or_else(|| Utc::now()),
            expires_at: DateTime::from_timestamp(token_data.claims.exp as i64, 0)
                .unwrap_or_else(|| Utc::now()),
        }
    }

    /// Generate access token for user
    pub fn generate_access_token(&self, username: &str, roles: Vec<String>, permissions: Vec<String>, account_ids: Vec<String>) -> Result<String, AuthError> {
        let now = Utc::now();
        let expire = now + Duration::seconds(self.config.expiration);

        let claims = Claims {
            sub: format!("user_{}", now.timestamp()),
            username: username.to_string(),
            roles,
            permissions,
            account_ids,
            iat: now.timestamp() as usize,
            exp: expire.timestamp() as usize,
            token_type: "access".to_string(),
            session_id: format!("session_{}", now.timestamp()),
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_ref()),
        ).map_err(|e| AuthError::InvalidCredentials(format!("Failed to generate token: {}", e)))
    }

    /// Generate refresh token for user
    pub fn generate_refresh_token(&self, username: &str, roles: Vec<String>, permissions: Vec<String>, account_ids: Vec<String>) -> Result<String, AuthError> {
        let now = Utc::now();
        let expire = now + Duration::seconds(self.config.refresh_expiration);

        let claims = Claims {
            sub: format!("user_{}", now.timestamp()),
            username: username.to_string(),
            roles,
            permissions,
            account_ids,
            iat: now.timestamp() as usize,
            exp: expire.timestamp() as usize,
            token_type: "refresh".to_string(),
            session_id: format!("session_{}", now.timestamp()),
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_ref()),
        ).map_err(|e| AuthError::InvalidCredentials(format!("Failed to generate refresh token: {}", e)))
    }

    /// Refresh access token using refresh token
    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String, AuthError> {
        let token_data = self.validate_token(refresh_token)?;

        if token_data.claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken("Invalid token type for refresh".to_string()));
        }

        self.generate_access_token(
            &token_data.claims.username,
            token_data.claims.roles,
            token_data.claims.permissions,
            token_data.claims.account_ids,
        )
    }

    /// Check if user has access to a specific account
    pub fn check_account_access(&self, context: &AuthContext, account_id: &str) -> Result<(), AuthError> {
        if context.has_account_access(account_id) {
            Ok(())
        } else {
            Err(AuthError::AccountAccessDenied(
                format!("User {} does not have access to account {}", context.username, account_id)
            ))
        }
    }

    /// Check if user has required authorization level
    pub fn check_authorization(&self, context: &AuthContext, required_level: AuthorizationLevel) -> Result<(), AuthError> {
        required_level.check_access(context)
    }

    /// Validate user credentials (placeholder for actual credential validation)
    pub fn validate_credentials(&self, username: &str, password: &str) -> Result<(Vec<String>, Vec<String>, Vec<String>), AuthError> {
        // Sanitize inputs
        let clean_username = self.security_validator
            .validate_string(username, "username", SanitizationLevel::Strict)
            .map_err(|_| AuthError::InvalidCredentials("Invalid username format".to_string()))?;

        let clean_password = self.security_validator
            .validate_string(password, "password", SanitizationLevel::Strict)
            .map_err(|_| AuthError::InvalidCredentials("Invalid password format".to_string()))?;

        // TODO: Implement actual credential validation against database
        // For now, return mock data based on username
        match clean_username.as_str() {
            "admin" => Ok((
                vec!["admin".to_string(), "trader".to_string()],
                vec!["read".to_string(), "write".to_string(), "admin".to_string()],
                vec!["*".to_string()], // Admin has access to all accounts
            )),
            "trader" => Ok((
                vec!["trader".to_string()],
                vec!["read".to_string(), "write".to_string()],
                vec!["acc_001".to_string(), "acc_002".to_string()],
            )),
            "user" => Ok((
                vec!["user".to_string()],
                vec!["read".to_string()],
                vec!["acc_001".to_string()],
            )),
            _ => Err(AuthError::InvalidCredentials("User not found".to_string())),
        }
    }
}

/// Authorization middleware for API endpoints
pub struct AuthMiddleware {
    auth_validator: AuthValidator,
    required_level: AuthorizationLevel,
    require_account_access: bool,
    account_id: Option<String>,
}

impl AuthMiddleware {
    /// Create new auth middleware with specified requirements
    pub fn new(required_level: AuthorizationLevel) -> Self {
        Self {
            auth_validator: AuthValidator::default(),
            required_level,
            require_account_access: false,
            account_id: None,
        }
    }

    /// Require specific account access
    pub fn with_account_access(mut self, account_id: String) -> Self {
        self.require_account_access = true;
        self.account_id = Some(account_id);
        self
    }

    /// Validate request with authentication and authorization
    pub fn validate_request(&self, token: Option<&str>) -> Result<AuthContext, AuthError> {
        let token = token.ok_or_else(|| {
            AuthError::InvalidToken("No authentication token provided".to_string())
        })?;

        let token_data = self.auth_validator.validate_token(token)?;
        let context = self.auth_validator.token_to_context(token_data);

        // Check authorization level
        self.auth_validator.check_authorization(&context, self.required_level)?;

        // Check account access if required
        if self.require_account_access {
            if let Some(ref account_id) = self.account_id {
                self.auth_validator.check_account_access(&context, account_id)?;
            }
        }

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_config_default() {
        let config = JwtConfig::default();
        assert_eq!(config.expiration, 3600);
        assert_eq!(config.refresh_expiration, 86400 * 7);
        assert_eq!(config.issuer, "ninja-gekko-api");
    }

    #[test]
    fn test_auth_context_roles() {
        let context = AuthContext {
            user_id: "user_123".to_string(),
            username: "testuser".to_string(),
            roles: vec!["admin".to_string(), "trader".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            account_ids: vec!["acc_001".to_string()],
            session_id: "session_123".to_string(),
            token_type: "access".to_string(),
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
        };

        assert!(context.has_role("admin"));
        assert!(context.has_role("trader"));
        assert!(!context.has_role("viewer"));
        assert!(context.has_permission("read"));
        assert!(context.has_account_access("acc_001"));
        assert!(!context.has_account_access("acc_002"));
        assert!(context.is_admin());
        assert!(context.get_role_level() == 100);
    }

    #[test]
    fn test_authorization_levels() {
        let admin_context = AuthContext {
            user_id: "user_123".to_string(),
            username: "admin".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            account_ids: vec![],
            session_id: "".to_string(),
            token_type: "".to_string(),
            issued_at: Utc::now(),
            expires_at: Utc::now(),
        };

        let user_context = AuthContext {
            user_id: "user_456".to_string(),
            username: "user".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec![],
            account_ids: vec![],
            session_id: "".to_string(),
            token_type: "".to_string(),
            issued_at: Utc::now(),
            expires_at: Utc::now(),
        };

        assert!(AuthorizationLevel::Admin.check_access(&admin_context).is_ok());
        assert!(AuthorizationLevel::User.check_access(&admin_context).is_ok());
        assert!(AuthorizationLevel::Admin.check_access(&user_context).is_err());
        assert!(AuthorizationLevel::User.check_access(&user_context).is_ok());
    }

    #[test]
    fn test_input_sanitization() {
        let validator = AuthValidator::default();

        // Valid inputs
        let result = validator.validate_credentials("user", "valid_pass");
        assert!(result.is_ok());

        // Invalid inputs should be sanitized
        let result = validator.validate_credentials("user<script>", "pass'--");
        assert!(result.is_err()); // Should be rejected due to invalid credentials
    }
}
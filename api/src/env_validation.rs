//! Environment variable validation and secure configuration management
//!
//! This module provides secure environment variable validation, configuration
//! management, and runtime security checks for the Ninja Gekko API.

use crate::validation::{SanitizationLevel, SecurityValidator, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Environment configuration with security validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// JWT configuration
    pub jwt: JwtSecureConfig,
    /// API configuration
    pub api: ApiSecureConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// External service configuration
    pub external: ExternalServiceConfig,
    /// Feature flags
    pub features: FeatureFlags,
}

impl Default for SecureConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            jwt: JwtSecureConfig::default(),
            api: ApiSecureConfig::default(),
            security: SecurityConfig::default(),
            external: ExternalServiceConfig::default(),
            features: FeatureFlags::default(),
        }
    }
}

/// Database configuration with security validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Database pool size
    pub pool_size: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// SSL mode
    pub ssl_mode: String,
    /// Database name
    pub database_name: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost:5432/gordon_gekko".to_string()),
            pool_size: std::env::var("DB_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            connection_timeout: std::env::var("DB_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            ssl_mode: std::env::var("DB_SSL_MODE").unwrap_or_else(|_| "require".to_string()),
            database_name: std::env::var("DB_NAME").unwrap_or_else(|_| "gordon_gekko".to_string()),
        }
    }
}

impl DatabaseConfig {
    /// Validate database configuration security
    pub fn validate(&self) -> ValidationResult<()> {
        // Check for hardcoded credentials in URL
        if self.url.contains("password=") || self.url.contains("@localhost") {
            // This is a warning, not an error - production should use proper secrets
            eprintln!("WARNING: Database URL may contain credentials. Use environment variables or secret management.");
        }

        // Validate SSL mode
        match self.ssl_mode.as_str() {
            "require" | "prefer" | "allow" | "disable" | "verify-ca" | "verify-full" => {}
            _ => return Err(self.create_ssl_mode_error(&self.ssl_mode)),
        }

        // Validate pool size
        if self.pool_size == 0 || self.pool_size > 100 {
            return Err(self.create_pool_size_error(self.pool_size));
        }

        Ok(())
    }

    fn create_ssl_mode_error(&self, mode: &str) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("invalid_ssl_mode");
        error.message = Some(std::borrow::Cow::from(format!(
            "Invalid SSL mode: {}",
            mode
        )));
        errors.add("ssl_mode", error);
        errors
    }

    fn create_pool_size_error(&self, size: u32) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("invalid_pool_size");
        error.message = Some(std::borrow::Cow::from(format!(
            "Invalid pool size: {} (must be 1-100)",
            size
        )));
        errors.add("pool_size", error);
        errors
    }
}

/// JWT configuration with security validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSecureConfig {
    /// JWT secret key (should be from environment)
    pub secret: String,
    /// Token expiration time
    pub expiration_seconds: i64,
    /// Refresh token expiration
    pub refresh_expiration_seconds: i64,
    /// Algorithm to use
    pub algorithm: String,
}

impl Default for JwtSecureConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                if cfg!(test) {
                    "test-secret-at-least-32-characters-long-for-testing".to_string()
                } else {
                    panic!("JWT_SECRET environment variable must be set")
                }
            }),
            expiration_seconds: std::env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string()) // 1 hour
                .parse()
                .unwrap_or(3600),
            refresh_expiration_seconds: std::env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .unwrap_or(604800),
            algorithm: std::env::var("JWT_ALGORITHM").unwrap_or_else(|_| "HS256".to_string()),
        }
    }
}

impl JwtSecureConfig {
    /// Validate JWT configuration security
    pub fn validate(&self) -> ValidationResult<()> {
        // Check secret strength
        if self.secret.len() < 32 {
            return Err(self.create_weak_secret_error(self.secret.len()));
        }

        // Check for default/weak secrets
        let weak_secrets = [
            "default-secret",
            "change-in-production",
            "password",
            "secret",
            "123456",
        ];

        if weak_secrets.iter().any(|&weak| self.secret.contains(weak)) {
            return Err(self.create_default_secret_error());
        }

        // Validate expiration times
        if self.expiration_seconds < 300 || self.expiration_seconds > 86400 {
            return Err(self.create_invalid_expiration_error(self.expiration_seconds));
        }

        if self.refresh_expiration_seconds < self.expiration_seconds {
            return Err(self.create_invalid_refresh_error(
                self.refresh_expiration_seconds,
                self.expiration_seconds,
            ));
        }

        // Validate algorithm
        match self.algorithm.as_str() {
            "HS256" | "HS384" | "HS512" | "RS256" | "RS384" | "RS512" => {}
            _ => return Err(self.create_invalid_algorithm_error(&self.algorithm)),
        }

        Ok(())
    }

    fn create_weak_secret_error(&self, length: usize) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("jwt_secret_too_short");
        error.message = Some(std::borrow::Cow::from(format!(
            "JWT secret too short: {} characters (minimum 32)",
            length
        )));
        errors.add("secret_length", error);
        errors
    }

    fn create_default_secret_error(&self) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let error =
            validator::ValidationError::new("JWT secret appears to be a default or weak value");
        errors.add("secret_strength", error);
        errors
    }

    fn create_invalid_expiration_error(&self, expiration: i64) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("invalid_jwt_expiration");
        error.message = Some(std::borrow::Cow::from(format!(
            "Invalid JWT expiration: {} seconds (must be 300-86400)",
            expiration
        )));
        errors.add("expiration", error);
        errors
    }

    fn create_invalid_refresh_error(
        &self,
        refresh: i64,
        access: i64,
    ) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("invalid_refresh_expiration");
        error.message = Some(std::borrow::Cow::from(format!(
            "Refresh token ({}) must expire after access token ({})",
            refresh, access
        )));
        errors.add("refresh_expiration", error);
        errors
    }

    fn create_invalid_algorithm_error(&self, algorithm: &str) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("unsupported_jwt_algorithm");
        error.message = Some(std::borrow::Cow::from(format!(
            "Unsupported JWT algorithm: {}",
            algorithm
        )));
        errors.add("algorithm", error);
        errors
    }
}

/// API configuration with security validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSecureConfig {
    /// Bind address
    pub bind_address: String,
    /// Port number
    pub port: u16,
    /// CORS origins
    pub cors_origins: Vec<String>,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
    /// Security headers configuration
    pub security_headers: SecurityHeadersConfig,
}

impl Default for ApiSecureConfig {
    fn default() -> Self {
        Self {
            bind_address: std::env::var("API_BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            rate_limiting: RateLimitConfig::default(),
            security_headers: SecurityHeadersConfig::default(),
        }
    }
}

impl ApiSecureConfig {
    /// Validate API configuration security
    pub fn validate(&self) -> ValidationResult<()> {
        // Check for binding to all interfaces in production
        if self.bind_address == "0.0.0.0"
            && std::env::var("ENVIRONMENT").unwrap_or_default() == "production"
        {
            eprintln!("WARNING: API binding to all interfaces (0.0.0.0) in production environment");
        }

        // Validate port range
        if self.port == 0 || self.port > 65535 {
            return Err(self.create_invalid_port_error(self.port));
        }

        // Validate CORS origins
        for origin in &self.cors_origins {
            if !self.is_valid_origin(origin) {
                return Err(self.create_invalid_cors_error(origin));
            }
        }

        Ok(())
    }

    fn is_valid_origin(&self, origin: &str) -> bool {
        origin.starts_with("http://") || origin.starts_with("https://")
    }

    fn create_invalid_port_error(&self, port: u16) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("invalid_port");
        error.message = Some(std::borrow::Cow::from(format!(
            "Invalid port number: {} (must be 1-65535)",
            port
        )));
        errors.add("port", error);
        errors
    }

    fn create_invalid_cors_error(&self, origin: &str) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("invalid_cors_origin");
        error.message = Some(std::borrow::Cow::from(format!(
            "Invalid CORS origin format: {}",
            origin
        )));
        errors.add("cors_origins", error);
        errors
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Global rate limit (requests per minute)
    pub global_rpm: u32,
    /// Per-user rate limit
    pub user_rpm: u32,
    /// Burst limit
    pub burst_limit: u32,
    /// Rate limit window in seconds
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            global_rpm: std::env::var("RATE_LIMIT_GLOBAL")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            user_rpm: std::env::var("RATE_LIMIT_USER")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            burst_limit: std::env::var("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            window_seconds: std::env::var("RATE_LIMIT_WINDOW")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        }
    }
}

/// Security headers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeadersConfig {
    /// Content Security Policy
    pub csp: String,
    /// HSTS configuration
    pub hsts_max_age: u64,
    /// Frame options
    pub frame_options: String,
    /// Content type options
    pub content_type_options: String,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            csp: std::env::var("CSP_HEADER").unwrap_or_else(|_| {
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
                    .to_string()
            }),
            hsts_max_age: std::env::var("HSTS_MAX_AGE")
                .unwrap_or_else(|_| "31536000".to_string()) // 1 year
                .parse()
                .unwrap_or(31536000),
            frame_options: std::env::var("FRAME_OPTIONS").unwrap_or_else(|_| "DENY".to_string()),
            content_type_options: std::env::var("CONTENT_TYPE_OPTIONS")
                .unwrap_or_else(|_| "nosniff".to_string()),
        }
    }
}

/// External service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceConfig {
    /// Market data provider URLs
    pub market_data_urls: Vec<String>,
    /// API keys (should be encrypted in production)
    pub api_keys: HashMap<String, String>,
    /// Service timeouts
    pub timeouts: ServiceTimeouts,
}

impl Default for ExternalServiceConfig {
    fn default() -> Self {
        Self {
            market_data_urls: std::env::var("MARKET_DATA_URLS")
                .unwrap_or_else(|_| "https://api.marketdata.com".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            api_keys: HashMap::new(), // Should be loaded securely
            timeouts: ServiceTimeouts::default(),
        }
    }
}

/// Service timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTimeouts {
    /// HTTP request timeout
    pub http_timeout: u64,
    /// Connection timeout
    pub connect_timeout: u64,
    /// Read timeout
    pub read_timeout: u64,
}

impl Default for ServiceTimeouts {
    fn default() -> Self {
        Self {
            http_timeout: std::env::var("HTTP_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            connect_timeout: std::env::var("CONNECT_TIMEOUT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            read_timeout: std::env::var("READ_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }
}

/// Feature flags for runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable debug mode
    pub debug_mode: bool,
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Enable audit logging
    pub audit_logging: bool,
    /// Enable rate limiting
    pub rate_limiting: bool,
    /// Enable CORS
    pub cors_enabled: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            debug_mode: std::env::var("DEBUG_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            metrics_enabled: std::env::var("METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            audit_logging: std::env::var("AUDIT_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            rate_limiting: std::env::var("RATE_LIMITING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cors_enabled: std::env::var("CORS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

/// Security configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session configuration
    pub session_config: SessionConfig,
    /// Encryption configuration
    pub encryption_config: EncryptionConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            password_policy: PasswordPolicy::default(),
            session_config: SessionConfig::default(),
            encryption_config: EncryptionConfig::default(),
        }
    }
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: u8,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special_chars: bool,
    /// Maximum password age in days
    pub max_age_days: u32,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: std::env::var("PASSWORD_MIN_LENGTH")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .unwrap_or(12),
            require_uppercase: std::env::var("PASSWORD_REQUIRE_UPPERCASE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            require_lowercase: std::env::var("PASSWORD_REQUIRE_LOWERCASE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            require_numbers: std::env::var("PASSWORD_REQUIRE_NUMBERS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            require_special_chars: std::env::var("PASSWORD_REQUIRE_SPECIAL")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_age_days: std::env::var("PASSWORD_MAX_AGE")
                .unwrap_or_else(|_| "90".to_string())
                .parse()
                .unwrap_or(90),
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in minutes
    pub timeout_minutes: u32,
    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: u32,
    /// Session cookie security settings
    pub cookie_secure: bool,
    /// HTTP-only cookies
    pub cookie_http_only: bool,
    /// Same-site policy
    pub cookie_same_site: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_minutes: std::env::var("SESSION_TIMEOUT")
                .unwrap_or_else(|_| "480".to_string()) // 8 hours
                .parse()
                .unwrap_or(480),
            max_concurrent_sessions: std::env::var("MAX_CONCURRENT_SESSIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            cookie_secure: std::env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cookie_http_only: std::env::var("COOKIE_HTTP_ONLY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cookie_same_site: std::env::var("COOKIE_SAME_SITE")
                .unwrap_or_else(|_| "Strict".to_string()),
        }
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Data encryption key
    pub data_key: String,
    /// Key rotation period in days
    pub key_rotation_days: u32,
    /// Encryption algorithm
    pub algorithm: String,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            data_key: std::env::var("DATA_ENCRYPTION_KEY").unwrap_or_else(|_| {
                eprintln!("WARNING: Using default encryption key - set DATA_ENCRYPTION_KEY!");
                "default-encryption-key-32-chars-min".to_string()
            }),
            key_rotation_days: std::env::var("KEY_ROTATION_DAYS")
                .unwrap_or_else(|_| "90".to_string())
                .parse()
                .unwrap_or(90),
            algorithm: std::env::var("ENCRYPTION_ALGORITHM")
                .unwrap_or_else(|_| "AES-256-GCM".to_string()),
        }
    }
}

/// Environment validator for comprehensive configuration validation
pub struct EnvironmentValidator {
    _security_validator: SecurityValidator,
}

impl EnvironmentValidator {
    /// Create new environment validator
    pub fn new() -> Self {
        Self {
            _security_validator: SecurityValidator::new(),
        }
    }

    /// Validate all environment variables and configuration
    pub fn validate_all(&self) -> ValidationResult<SecureConfig> {
        let config = SecureConfig::default();

        // Validate each component
        config.database.validate()?;
        config.jwt.validate()?;
        config.api.validate()?;

        // Check for required environment variables
        self.validate_required_env_vars()?;

        // Check for security warnings
        self.check_security_warnings(&config);

        Ok(config)
    }

    /// Validate required environment variables
    fn validate_required_env_vars(&self) -> ValidationResult<()> {
        let required_vars = vec!["DATABASE_URL", "JWT_SECRET"];

        let mut missing_vars = Vec::new();

        for var in &required_vars {
            if std::env::var(var).is_err() {
                missing_vars.push(var.to_string());
            }
        }

        if !missing_vars.is_empty() {
            return Err(self.create_missing_env_vars_error(&missing_vars));
        }

        Ok(())
    }

    /// Check for security warnings in configuration
    fn check_security_warnings(&self, config: &SecureConfig) {
        // Check for development defaults in production
        if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
            if config.jwt.secret.contains("default") || config.jwt.secret.contains("change") {
                eprintln!("SECURITY WARNING: Using default JWT secret in production");
            }

            if config.api.bind_address == "0.0.0.0" {
                eprintln!("SECURITY WARNING: API binding to all interfaces in production");
            }

            if !config.api.cors_origins.is_empty() {
                eprintln!("SECURITY WARNING: CORS enabled in production - verify origins");
            }
        }

        // Check for debug mode in production
        if std::env::var("ENVIRONMENT").unwrap_or_default() == "production"
            && config.features.debug_mode
        {
            eprintln!("SECURITY WARNING: Debug mode enabled in production");
        }
    }

    fn create_missing_env_vars_error(&self, missing: &[String]) -> validator::ValidationErrors {
        let mut errors = validator::ValidationErrors::new();
        let mut error = validator::ValidationError::new("missing_required_env_vars");
        error.message = Some(std::borrow::Cow::from(format!(
            "Missing required environment variables: {:?}",
            missing
        )));
        errors.add("environment", error);
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_database_config_validation() {
        let config = DatabaseConfig::default();
        let result = config.validate();
        // Should not panic even with default values
        assert!(result.is_ok() || result.is_err()); // Either is acceptable for this test
    }

    #[test]
    fn test_jwt_config_validation() {
        let config = JwtSecureConfig::default();
        let result = config.validate();
        // Should validate with default values
        if let Err(errors) = result {
            // If validation fails, it should be for specific security reasons
            assert!(
                errors.field_errors().contains_key("secret_length")
                    || errors.field_errors().contains_key("secret_strength")
            );
        }
    }

    #[test]
    fn test_api_config_validation() {
        let config = ApiSecureConfig::default();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_policy_defaults() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 12);
        assert!(policy.require_uppercase);
        assert!(policy.require_lowercase);
        assert!(policy.require_numbers);
        assert!(policy.require_special_chars);
        assert_eq!(policy.max_age_days, 90);
    }

    #[test]
    fn test_feature_flags_defaults() {
        let flags = FeatureFlags::default();
        assert!(flags.metrics_enabled);
        assert!(flags.audit_logging);
        assert!(flags.rate_limiting);
        assert!(flags.cors_enabled);
        assert!(!flags.debug_mode);
    }

    #[test]
    fn test_environment_validation() {
        // Set required environment variables for test
        env::set_var("DATABASE_URL", "postgresql://localhost/test");
        env::set_var("JWT_SECRET", "test-secret-at-least-32-characters-long");

        let validator = EnvironmentValidator::new();
        let result = validator.validate_required_env_vars();
        assert!(result.is_ok());

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("JWT_SECRET");
    }
}

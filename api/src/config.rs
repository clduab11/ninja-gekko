//! API Configuration Module
//!
//! Handles configuration loading and management for the Ninja Gekko API server.
//! Supports environment variables, configuration files, and runtime configuration.

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::{info, warn};

/// Server configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    /// Server bind address and port
    pub bind_address: SocketAddr,

    /// Database connection URL
    pub database_url: String,

    /// JWT secret for authentication
    pub jwt_secret: String,

    /// Server environment (development, staging, production)
    pub environment: String,

    /// CORS allowed origins
    pub cors_origins: Vec<String>,

    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Maximum payload size in bytes
    pub max_payload_size: usize,

    /// Enable WebSocket support
    pub enable_websocket: bool,

    /// WebSocket heartbeat interval in seconds
    pub websocket_heartbeat_interval: u64,

    /// Enable API documentation endpoint
    pub enable_docs: bool,

    /// Logging configuration
    pub logging: LoggingConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080),
            database_url: "postgresql://localhost/gordon_gekko".to_string(),
            jwt_secret: std::env::var("GG_API_JWT_SECRET").unwrap_or_else(|_| {
                if cfg!(test) {
                    "test-secret-value-for-unit-tests-do-not-use-in-prod".to_string()
                } else {
                    panic!("JWT secret must be set via GG_API_JWT_SECRET env variable")
                }
            }),
            environment: "development".to_string(),
            cors_origins: vec!["http://localhost:3000".to_string()],
            rate_limiting: RateLimitingConfig::default(),
            request_timeout_secs: 30,
            max_payload_size: 1024 * 1024, // 1MB
            enable_websocket: true,
            websocket_heartbeat_interval: 30,
            enable_docs: true,
            logging: LoggingConfig::default(),
        }
    }
}

impl ApiConfig {
    /// Loads configuration from environment variables and config files
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(File::with_name("api.toml").required(false))
            .add_source(File::with_name("config/api.toml").required(false))
            .add_source(
                Environment::with_prefix("GG_API")
                    .separator("_")
                    .try_parsing(true),
            )
            .set_override_option("database_url", std::env::var("GG_API_DATABASE_URL").ok())?
            .set_default("bind_address", "0.0.0.0:8080")?
            .set_default("database_url", "postgresql://localhost/gordon_gekko")?
            .set_default(
                "jwt_secret",
                "your-super-secret-jwt-key-change-this-in-production",
            )?
            .set_default("environment", "development")?
            .set_default("cors_origins", vec!["http://localhost:3000"])?
            .set_default("request_timeout_secs", 30)?
            .set_default("max_payload_size", 1024 * 1024)?
            .set_default("enable_websocket", true)?
            .set_default("websocket_heartbeat_interval", 30)?
            .set_default("enable_docs", true)?;

        // Parse CORS origins from environment variable
        if let Ok(cors_origins_str) = env::var("GG_API_CORS_ORIGINS") {
            let cors_origins: Vec<String> = cors_origins_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !cors_origins.is_empty() {
                builder = builder.set_override("cors_origins", cors_origins)?;
            }
        }

        let config = builder.build()?;
        let api_config: ApiConfig = config.try_deserialize()?;

        // Validate configuration
        api_config.validate()?;

        // Log configuration (without sensitive data)
        info!("API Configuration loaded:");
        info!("  Environment: {}", api_config.environment);
        info!("  Bind Address: {}", api_config.bind_address);
        // Do not log database connection secrets or sensitive info
        info!("  Database: [REDACTED]");
        info!("  CORS Origins: {:?}", api_config.cors_origins);
        info!(
            "  Rate Limiting: {} req/minute",
            api_config.rate_limiting.requests_per_minute
        );
        info!("  WebSocket Enabled: {}", api_config.enable_websocket);
        info!("  Request Timeout: {}s", api_config.request_timeout_secs);

        if api_config.jwt_secret == "your-super-secret-jwt-key-change-this-in-production" {
            warn!("⚠️  Using default JWT secret! Please set GG_API_JWT_SECRET environment variable in production!");
        }

        Ok(api_config)
    }

    /// Validates the configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.request_timeout_secs == 0 {
            return Err(ConfigError::Message(
                "Request timeout must be greater than 0".to_string(),
            ));
        }

        if self.max_payload_size == 0 {
            return Err(ConfigError::Message(
                "Max payload size must be greater than 0".to_string(),
            ));
        }

        if self.jwt_secret.is_empty() {
            return Err(ConfigError::Message(
                "JWT secret cannot be empty".to_string(),
            ));
        }

        if self.database_url.is_empty() {
            return Err(ConfigError::Message(
                "Database URL cannot be empty".to_string(),
            ));
        }

        if self.cors_origins.is_empty() {
            warn!("CORS origins list is empty - this may cause issues in production");
        }

        Ok(())
    }

    /// Returns true if running in development mode
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    /// Returns true if running in production mode
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Returns true if running in staging mode
    pub fn is_staging(&self) -> bool {
        self.environment == "staging"
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimitingConfig {
    /// Requests per second
    pub requests_per_second: u32,

    /// Requests per minute
    pub requests_per_minute: u32,

    /// Requests per hour
    pub requests_per_hour: u32,

    /// Burst limit (allowance for sudden spikes)
    pub burst_limit: u32,

    /// Enable rate limiting
    pub enabled: bool,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            requests_per_minute: 600,
            requests_per_hour: 36000,
            burst_limit: 20,
            enabled: true,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Enable structured logging (JSON format)
    pub structured: bool,

    /// Log to file in addition to stdout
    pub log_to_file: bool,

    /// Log file path (if logging to file)
    pub log_file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            structured: false,
            log_to_file: false,
            log_file_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ApiConfig::default();
        assert_eq!(config.bind_address.port(), 8080);
        assert_eq!(config.environment, "development");
        assert!(config.is_development());
        assert!(!config.is_production());
        assert!(!config.is_staging());
    }

    #[test]
    fn test_rate_limiting_config() {
        let config = RateLimitingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.requests_per_minute, 600);
        assert_eq!(config.burst_limit, 20);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ApiConfig::default();
        config.request_timeout_secs = 0;
        assert!(config.validate().is_err());

        config.request_timeout_secs = 30;
        config.jwt_secret = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cors_origins_parsing() {
        // This would test the environment variable parsing for CORS origins
        // In a real test, we would set environment variables
        let config = ApiConfig::default();
        assert!(!config.cors_origins.is_empty());
    }
}

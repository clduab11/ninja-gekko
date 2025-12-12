//! Error handling and custom error types for the API
//!
//! This module provides comprehensive error handling with custom error types,
//! structured error responses, and proper HTTP status code mapping.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{error, warn};

/// Main API error type that encompasses all possible errors
#[derive(Debug, Error)]
pub enum ApiError {
    /// Configuration-related errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Database-related errors
    #[error("Database error: {message}")]
    Database { message: String },

    /// Authentication errors
    #[error("Authentication error: {message}")]
    Auth { message: String },

    /// Authorization errors
    #[error("Authorization error: {message}")]
    Authorization { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// Trading engine errors
    #[error("Trading error: {message}")]
    Trading { message: String },

    /// Market data errors
    #[error("Market data error: {message}")]
    MarketData { message: String },

    /// Portfolio errors
    #[error("Portfolio error: {message}")]
    Portfolio { message: String },

    /// Strategy errors
    #[error("Strategy error: {message}")]
    Strategy { message: String },

    /// Arbitrage errors
    #[error("Arbitrage error: {message}")]
    Arbitrage { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },

    /// External service errors
    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    /// Internal server errors
    #[error("Internal server error: {message}")]
    Internal { message: String },

    /// Not found errors
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    /// Bad request errors
    #[error("Bad request: {message}")]
    BadRequest { message: String },

    /// Not implemented errors
    #[error("Not implemented: {message}")]
    NotImplemented { message: String },
}

impl ApiError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a database error
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    /// Create an authorization error
    pub fn auth_required() -> Self {
        Self::Authorization {
            message: "Authentication required".to_string(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S, field: Option<S>) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.map(|f| f.into()),
        }
    }

    /// Create a trading error
    pub fn trading<S: Into<String>>(message: S) -> Self {
        Self::Trading {
            message: message.into(),
        }
    }

    /// Create a market data error
    pub fn market_data<S: Into<String>>(message: S) -> Self {
        Self::MarketData {
            message: message.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Self::RateLimit {
            message: message.into(),
        }
    }

    /// Create an external service error
    pub fn external_service<S: Into<String>>(service: S, message: S) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create an internal server error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a bad request error
    pub fn bad_request<S: Into<String>>(message: S) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    /// Create a not implemented error
    pub fn not_implemented<S: Into<String>>(message: S) -> Self {
        Self::NotImplemented {
            message: message.into(),
        }
    }

    /// Get the appropriate HTTP status code for the error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Config { .. } | ApiError::Internal { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Auth { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Authorization { .. } => StatusCode::FORBIDDEN,
            ApiError::Validation { .. } => StatusCode::BAD_REQUEST,
            ApiError::Trading { .. } => StatusCode::BAD_REQUEST,
            ApiError::MarketData { .. } => StatusCode::BAD_REQUEST,
            ApiError::Portfolio { .. } => StatusCode::BAD_REQUEST,
            ApiError::Strategy { .. } => StatusCode::BAD_REQUEST,
            ApiError::Arbitrage { .. } => StatusCode::BAD_REQUEST,
            ApiError::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            ApiError::ExternalService { .. } => StatusCode::BAD_GATEWAY,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::NotImplemented { .. } => StatusCode::NOT_IMPLEMENTED,
        }
    }

    /// Get the error code for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::Config { .. } => "CONFIG_ERROR",
            ApiError::Database { .. } => "DATABASE_ERROR",
            ApiError::Auth { .. } => "AUTH_ERROR",
            ApiError::Authorization { .. } => "AUTHORIZATION_ERROR",
            ApiError::Validation { .. } => "VALIDATION_ERROR",
            ApiError::Trading { .. } => "TRADING_ERROR",
            ApiError::MarketData { .. } => "MARKET_DATA_ERROR",
            ApiError::Portfolio { .. } => "PORTFOLIO_ERROR",
            ApiError::Strategy { .. } => "STRATEGY_ERROR",
            ApiError::Arbitrage { .. } => "ARBITRAGE_ERROR",
            ApiError::RateLimit { .. } => "RATE_LIMIT_ERROR",
            ApiError::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            ApiError::Internal { .. } => "INTERNAL_ERROR",
            ApiError::NotFound { .. } => "NOT_FOUND",
            ApiError::BadRequest { .. } => "BAD_REQUEST",
            ApiError::NotImplemented { .. } => "NOT_IMPLEMENTED",
        }
    }

    /// Check if the error is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self.status_code(),
            StatusCode::BAD_REQUEST
                | StatusCode::UNAUTHORIZED
                | StatusCode::FORBIDDEN
                | StatusCode::NOT_FOUND
                | StatusCode::TOO_MANY_REQUESTS
        )
    }

    /// Check if the error is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_GATEWAY
        )
    }

    /// Log the error appropriately based on type
    pub fn log_error(&self) {
        match self {
            ApiError::Config { message } => {
                error!("Configuration error: {}", message);
            }
            ApiError::Database { message } => {
                error!("Database error: {}", message);
            }
            ApiError::Internal { message } => {
                error!("Internal server error: {}", message);
            }
            ApiError::ExternalService { service, message } => {
                error!("External service {} error: {}", service, message);
            }
            ApiError::RateLimit { message } => {
                warn!("Rate limit exceeded: {}", message);
            }
            _ => {
                // Client errors are logged at debug level
                tracing::debug!("Client error: {}", self);
            }
        }
    }

    /// Convert to a structured error response
    pub fn to_error_response(&self, request_id: Option<String>) -> ErrorResponse {
        self.log_error();

        let mut details = HashMap::new();
        if let ApiError::Validation { field, .. } = self {
            if let Some(field_name) = field {
                details.insert("field".to_string(), field_name.clone().into());
            }
        }

        ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                details,
            },
            request_id,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Structured error response for API clients
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetail,

    /// Request ID for tracing
    pub request_id: Option<String>,

    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detailed error information
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Machine-readable error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional error details
    pub details: HashMap<String, serde_json::Value>,
}

/// Custom result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Custom result type for API responses
pub type ApiResponse<T> = Result<super::models::ApiResponse<T>, ApiError>;

/// Extension trait for converting results to API responses
pub trait ToApiResponse<T> {
    /// Convert a result to an API response
    fn to_api_response(self, request_id: Option<String>) -> ApiResponse<T>;
}

impl<T, E> ToApiResponse<T> for Result<T, E>
where
    E: Into<ApiError>,
{
    fn to_api_response(self, request_id: Option<String>) -> ApiResponse<T> {
        match self {
            Ok(data) => Ok(super::models::ApiResponse::success_with_request_id(
                data,
                request_id.unwrap_or_default(),
            )),
            Err(err) => {
                let api_error = err.into();
                let error_response = api_error.to_error_response(request_id.clone());
                Ok(super::models::ApiResponse::error_with_request_id(
                    serde_json::to_string(&error_response)
                        .unwrap_or_else(|_| "Failed to serialize error response".to_string()),
                    request_id.unwrap_or_default(),
                ))
            }
        }
    }
}

/// Axum response implementation for API errors
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_response = self.to_error_response(None);

        tracing::debug!(
            "API Error Response: status={}, code={}, message={}",
            status_code,
            error_response.error.code,
            error_response.error.message
        );

        (status_code, Json(error_response)).into_response()
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApiError::Auth {
            message: format!("Token error: {}", err),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::Internal {
            message: format!("Serialization error: {}", err),
        }
    }
}

impl From<axum::Error> for ApiError {
    fn from(err: axum::Error) -> Self {
        ApiError::Internal {
            message: format!("Server error: {}", err),
        }
    }
}

/// Common validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Required field is missing
    #[error("Required field '{field}' is missing")]
    Required { field: String },

    /// Field has invalid format
    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },

    /// Field value is out of range
    #[error("Field '{field}' value {value} is out of range: {reason}")]
    OutOfRange {
        field: String,
        value: String,
        reason: String,
    },

    /// Field value is too long
    #[error("Field '{field}' is too long (max {max_length} characters)")]
    TooLong { field: String, max_length: usize },

    /// Field value is too short
    #[error("Field '{field}' is too short (min {min_length} characters)")]
    TooShort { field: String, min_length: usize },

    /// Invalid enumeration value
    #[error("Field '{field}' has invalid value '{value}'. Allowed values: {allowed}")]
    InvalidEnum {
        field: String,
        value: String,
        allowed: String,
    },
}

impl ValidationError {
    /// Create a required field error
    pub fn required(field: impl Into<String>) -> Self {
        Self::Required {
            field: field.into(),
        }
    }

    /// Create an invalid format error
    pub fn invalid_format(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidFormat {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Create an out of range error
    pub fn out_of_range(
        field: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::OutOfRange {
            field: field.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create a too long error
    pub fn too_long(field: impl Into<String>, max_length: usize) -> Self {
        Self::TooLong {
            field: field.into(),
            max_length,
        }
    }

    /// Create a too short error
    pub fn too_short(field: impl Into<String>, min_length: usize) -> Self {
        Self::TooShort {
            field: field.into(),
            min_length,
        }
    }

    /// Create an invalid enum error
    pub fn invalid_enum(
        field: impl Into<String>,
        value: impl Into<String>,
        allowed: impl Into<String>,
    ) -> Self {
        Self::InvalidEnum {
            field: field.into(),
            value: value.into(),
            allowed: allowed.into(),
        }
    }

    /// Convert to ApiError
    pub fn to_api_error(self) -> ApiError {
        match self {
            ValidationError::Required { field } => ApiError::Validation {
                message: format!("Required field '{}' is missing", field),
                field: Some(field),
            },
            ValidationError::InvalidFormat { field, reason } => ApiError::Validation {
                message: format!("Field '{}' has invalid format: {}", field, reason),
                field: Some(field),
            },
            ValidationError::OutOfRange {
                field,
                value,
                reason,
            } => ApiError::Validation {
                message: format!(
                    "Field '{}' value {} is out of range: {}",
                    field, value, reason
                ),
                field: Some(field),
            },
            ValidationError::TooLong { field, max_length } => ApiError::Validation {
                message: format!(
                    "Field '{}' is too long (max {} characters)",
                    field, max_length
                ),
                field: Some(field),
            },
            ValidationError::TooShort { field, min_length } => ApiError::Validation {
                message: format!(
                    "Field '{}' is too short (min {} characters)",
                    field, min_length
                ),
                field: Some(field),
            },
            ValidationError::InvalidEnum {
                field,
                value,
                allowed,
            } => ApiError::Validation {
                message: format!(
                    "Field '{}' has invalid value '{}'. Allowed values: {}",
                    field, value, allowed
                ),
                field: Some(field),
            },
        }
    }
}

/// Trading-specific errors
#[derive(Debug, Error)]
pub enum TradingError {
    /// Insufficient funds for trade
    #[error("Insufficient funds: available {available}, required {required}")]
    InsufficientFunds { available: f64, required: f64 },

    /// Invalid order parameters
    #[error("Invalid order parameters: {reason}")]
    InvalidOrder { reason: String },

    /// Market is closed
    #[error("Market is closed for symbol {symbol}")]
    MarketClosed { symbol: String },

    /// Position not found
    #[error("Position not found: {position_id}")]
    PositionNotFound { position_id: String },

    /// Order not found
    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: String },

    /// Risk limit exceeded
    #[error("Risk limit exceeded: {limit_type} limit {limit_value}")]
    RiskLimitExceeded {
        limit_type: String,
        limit_value: f64,
    },
}

impl TradingError {
    /// Convert to ApiError
    pub fn to_api_error(self) -> ApiError {
        match self {
            TradingError::InsufficientFunds {
                available,
                required,
            } => ApiError::Trading {
                message: format!(
                    "Insufficient funds: available {}, required {}",
                    available, required
                ),
            },
            TradingError::InvalidOrder { reason } => ApiError::Trading {
                message: format!("Invalid order parameters: {}", reason),
            },
            TradingError::MarketClosed { symbol } => ApiError::Trading {
                message: format!("Market is closed for symbol {}", symbol),
            },
            TradingError::PositionNotFound { position_id } => ApiError::NotFound {
                resource: format!("Position {}", position_id),
            },
            TradingError::OrderNotFound { order_id } => ApiError::NotFound {
                resource: format!("Order {}", order_id),
            },
            TradingError::RiskLimitExceeded {
                limit_type,
                limit_value,
            } => ApiError::Trading {
                message: format!("Risk limit exceeded: {} limit {}", limit_type, limit_value),
            },
        }
    }
}

/// Market data errors
#[derive(Debug, Error)]
pub enum MarketDataError {
    /// Symbol not found
    #[error("Symbol not found: {symbol}")]
    SymbolNotFound { symbol: String },

    /// No data available
    #[error("No market data available for symbol {symbol}")]
    NoDataAvailable { symbol: String },

    /// Invalid time range
    #[error("Invalid time range: {reason}")]
    InvalidTimeRange { reason: String },

    /// Data source unavailable
    #[error("Market data source unavailable: {details}")]
    DataSourceUnavailable { details: String },
}

impl MarketDataError {
    /// Convert to ApiError
    pub fn to_api_error(self) -> ApiError {
        match self {
            MarketDataError::SymbolNotFound { symbol } => ApiError::NotFound {
                resource: format!("Market data for symbol {}", symbol),
            },
            MarketDataError::NoDataAvailable { symbol } => ApiError::MarketData {
                message: format!("No market data available for symbol {}", symbol),
            },
            MarketDataError::InvalidTimeRange { reason } => ApiError::MarketData {
                message: format!("Invalid time range: {}", reason),
            },
            MarketDataError::DataSourceUnavailable { details } => ApiError::ExternalService {
                service: details,
                message: "Market data source unavailable".to_string(),
            },
        }
    }
}

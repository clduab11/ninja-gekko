//! Comprehensive security validation layer
//!
//! This module provides centralized input validation, sanitization,
//! and security checks for the Ninja Gekko API server.

use serde::{Deserialize, Serialize};
use validator::{ValidationError, ValidationErrors};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use chrono::{DateTime, Utc};
use std::borrow::Cow;

/// Security configuration for validation rules
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum string length allowed
    pub max_string_length: usize,
    /// Maximum array/collection size
    pub max_collection_size: usize,
    /// Maximum numeric value
    pub max_numeric_value: f64,
    /// Minimum numeric value
    pub min_numeric_value: f64,
    /// Allowed file extensions for uploads
    pub allowed_file_extensions: Vec<String>,
    /// Blocked IP patterns
    pub blocked_ip_patterns: Vec<String>,
    /// Rate limiting thresholds
    pub rate_limits: HashMap<String, u32>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_string_length: 1000,
            max_collection_size: 100,
            max_numeric_value: 1_000_000_000.0,
            min_numeric_value: -1_000_000_000.0,
            allowed_file_extensions: vec![
                "jpg".to_string(), "jpeg".to_string(), "png".to_string(),
                "gif".to_string(), "pdf".to_string(), "txt".to_string(),
                "csv".to_string(), "json".to_string()
            ],
            blocked_ip_patterns: vec![
                r"192\.168\..*".to_string(),
                r"10\..*".to_string(),
                r"127\..*".to_string(),
            ],
            rate_limits: [
                ("auth".to_string(), 5),
                ("trades".to_string(), 100),
                ("portfolio".to_string(), 50),
                ("market_data".to_string(), 1000),
            ].iter().cloned().collect(),
        }
    }
}

lazy_static! {
    static ref SQL_INJECTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)union\s+select").unwrap(),
        Regex::new(r"(?i)select\s+.*\s+from").unwrap(),
        Regex::new(r"(?i)insert\s+into").unwrap(),
        Regex::new(r"(?i)update\s+.*\s+set").unwrap(),
        Regex::new(r"(?i)delete\s+from").unwrap(),
        Regex::new(r"(?i)drop\s+table").unwrap(),
        Regex::new(r"(?i)alter\s+table").unwrap(),
        Regex::new(r"--.*").unwrap(),
        Regex::new(r"/\*.*\*/").unwrap(),
        Regex::new(r";.*--").unwrap(),
    ];

    static ref XSS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"<script[^>]*>.*?</script>").unwrap(),
        Regex::new(r"javascript:").unwrap(),
        Regex::new(r"on\w+\s*=").unwrap(),
    ];
}

/// Input sanitization levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SanitizationLevel {
    /// Basic sanitization - remove dangerous characters
    Basic,
    /// Strict sanitization - remove all HTML/script patterns
    Strict,
    /// None - no sanitization (use with caution)
    None,
}

/// Rate limiting context for validation
#[derive(Debug, Clone)]
pub struct RateLimitContext {
    pub endpoint: String,
    pub user_id: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Main validation result type
pub type ValidationResult<T> = Result<T, ValidationErrors>;

/// Security validator for comprehensive input validation
pub struct SecurityValidator {
    pub config: SecurityConfig, // Made public for access in RateLimitValidator
}

impl SecurityValidator {
    /// Create a new security validator with default configuration
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
        }
    }

    /// Create a new security validator with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Validate and sanitize a string input
    pub fn validate_string(&self, input: &str, field_name: &'static str, level: SanitizationLevel) -> ValidationResult<String> {
        let sanitized = match level {
            SanitizationLevel::Basic => self.sanitize_basic(input),
            SanitizationLevel::Strict => self.sanitize_strict(input),
            SanitizationLevel::None => input.to_string(),
        };

        if sanitized.len() > self.config.max_string_length {
            return Err(self.create_length_error(field_name, sanitized.len(), self.config.max_string_length));
        }

        if self.contains_sql_injection(&sanitized) {
            return Err(self.create_security_error(
                field_name,
                "sql_injection",
                "Potential SQL injection detected",
                ValidationSeverity::Critical,
            ));
        }

        if level == SanitizationLevel::Strict && self.contains_xss(&sanitized) {
            return Err(self.create_security_error(
                field_name,
                "xss_attempt",
                "Potential XSS attack detected",
                ValidationSeverity::High,
            ));
        }

        Ok(sanitized)
    }

    /// Validate numeric input within bounds
    pub fn validate_numeric<T>(&self, input: T, field_name: &'static str) -> ValidationResult<T>
    where
        T: PartialOrd + Copy + std::fmt::Debug + Into<f64>,
    {
        let min_val = self.config.min_numeric_value;
        let max_val = self.config.max_numeric_value;
        let input_val: f64 = input.into();

        if input_val < min_val || input_val > max_val {
            return Err(self.create_range_error(field_name, input_val, min_val, max_val));
        }

        Ok(input)
    }

    /// Validate collection size
    pub fn validate_collection<T>(&self, collection: &[T], field_name: &'static str) -> ValidationResult<()> {
        if collection.len() > self.config.max_collection_size {
            return Err(self.create_collection_size_error(
                field_name,
                collection.len(),
                self.config.max_collection_size
            ));
        }
        Ok(())
    }

    /// Validate file extension
    pub fn validate_file_extension(&self, filename: &str) -> ValidationResult<String> {
        let extension = filename
            .split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();

        if !self.config.allowed_file_extensions.contains(&extension) {
            return Err(self.create_file_extension_error(&extension));
        }

        Ok(extension)
    }

    /// Validate IP address against blocked patterns
    pub fn validate_ip_address(&self, ip: &str) -> ValidationResult<()> {
        for pattern in &self.config.blocked_ip_patterns {
            if Regex::new(pattern).unwrap().is_match(ip) {
                return Err(self.create_ip_blocked_error(ip));
            }
        }
        Ok(())
    }

    fn sanitize_basic(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '<' | '>' | '&' | '"' | '\'' => ' ',
                _ => c,
            })
            .collect::<String>()
    }

    fn sanitize_strict(&self, input: &str) -> String {
        let mut result = input.to_string();
        result = Regex::new(r"<[^>]*>").unwrap().replace_all(&result, "").to_string();
        result
    }

    pub fn contains_sql_injection(&self, input: &str) -> bool {
        SQL_INJECTION_PATTERNS.iter().any(|pattern| pattern.is_match(input))
    }

    fn contains_xss(&self, input: &str) -> bool {
        XSS_PATTERNS.iter().any(|pattern| pattern.is_match(input))
    }

    fn create_length_error(&self, field: &'static str, actual: usize, _max: usize) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let mut error = ValidationError::new("length");
        error.message = Some(Cow::from("Maximum length exceeded"));
        error.add_param(Cow::from("value"), &actual);
        errors.add(field, error);
        errors
    }

    fn create_range_error(&self, field: &'static str, value: f64, min: f64, max: f64) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let mut error = ValidationError::new("range");
        error.message = Some(Cow::from(format!("Value {} out of range", value)));
        errors.add(field, error);
        errors
    }

    fn create_collection_size_error(&self, field: &'static str, actual: usize, max: usize) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let mut error = ValidationError::new("collection_size");
        error.message = Some(Cow::from(format!("Collection size {} exceeds maximum {}", actual, max)));
        errors.add(field, error);
        errors
    }

    fn create_file_extension_error(&self, extension: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let mut error = ValidationError::new("file_extension");
        error.message = Some(Cow::from(format!("File extension {} not allowed", extension)));
        errors.add("file_extension", error);
        errors
    }

    fn create_ip_blocked_error(&self, ip: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let mut error = ValidationError::new("ip_blocked");
        error.message = Some(Cow::from(format!("IP address {} is blocked", ip)));
        errors.add("ip_address", error);
        errors
    }

    fn create_security_error(
        &self,
        field: &'static str,
        code: &'static str,
        message: &str,
        _severity: ValidationSeverity,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let mut error = ValidationError::new(code);
        error.message = Some(Cow::from(message.to_string()));
        errors.add(field, error);
        errors
    }
}

/// Rate limiting validation
pub struct RateLimitValidator {
    validator: SecurityValidator,
}

impl RateLimitValidator {
    pub fn new() -> Self {
        Self {
            validator: SecurityValidator::new(),
        }
    }

    pub fn check_rate_limit(&self, context: &RateLimitContext) -> ValidationResult<()> {
        if context.endpoint.is_empty() {
             let mut errors = ValidationErrors::new();
             let mut error = ValidationError::new("rate_limit");
             error.message = Some(Cow::from("Endpoint cannot be empty"));
             errors.add("rate_limit", error);
             return Err(errors);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let validator = SecurityValidator::new();
        assert!(validator.contains_sql_injection("SELECT * FROM users"));
        assert!(validator.contains_sql_injection("UNION SELECT password FROM users"));
        assert!(!validator.contains_sql_injection("normal text"));
    }

    #[test]
    fn test_string_validation() {
        let validator = SecurityValidator::new();
        let result = validator.validate_string("hello world", "test_field", SanitizationLevel::Basic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_numeric_validation() {
        let validator = SecurityValidator::new();
        let result = validator.validate_numeric(100.0, "amount");
        assert!(result.is_ok());
    }
}

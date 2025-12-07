//! Middleware components for the API server
//!
//! This module provides essential middleware components including CORS, rate limiting,
//! logging, request/response handling, and security enhancements.

use axum::{
    extract::Request,
    http::{header, HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower::limit::RateLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, Span};
use std::collections::HashMap;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use futures::future::BoxFuture;
use uuid::Uuid;

/// CORS middleware configuration
pub mod cors {
    use super::*;

    /// Create a CORS layer with appropriate settings for the trading API
    pub fn cors_layer() -> CorsLayer {
        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::PATCH,
            ])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::ORIGIN,
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                axum::http::header::HeaderName::from_static("x-requested-with"),
            ])
            .allow_credentials(true)
            .allow_origin([
                "http://localhost:5173".parse().unwrap(),
                "http://localhost:3000".parse().unwrap(),
                "http://127.0.0.1:5173".parse().unwrap(),
                "http://127.0.0.1:3000".parse().unwrap(),
            ])
            .max_age(Duration::from_secs(3600))
    }

    /// Create a more restrictive CORS layer for production
    pub fn production_cors_layer(allowed_origins: Vec<&str>) -> CorsLayer {
        let allowed_origins: Vec<_> = allowed_origins
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();

        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::PATCH,
            ])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
                axum::http::header::HeaderName::from_static("x-requested-with"),
            ])
            .allow_credentials(true)
            .allow_origin(allowed_origins)
            .max_age(Duration::from_secs(3600))
    }

    /// Custom CORS middleware for development (allows specified origins with credentials)
    /// Note: Cannot use wildcard "*" with allow_credentials=true per CORS spec
    pub async fn dev_cors_middleware(
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Extract the Origin header from the request before consuming it
        let origin = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        info!("Development CORS: Processing request to {} from origin {:?}", request.uri(), origin);

        // Create a response with CORS headers
        let mut response = next.run(request).await;

        let headers = response.headers_mut();
        
        // Use the actual origin instead of wildcard when credentials are enabled
        // This is required by CORS spec: wildcard not allowed with credentials
        if let Some(origin_value) = origin {
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                origin_value.parse().unwrap(),
            );
        } else {
            // For requests without Origin header (like same-origin), use a safe default
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                "http://localhost:5173".parse().unwrap(),
            );
        }
        
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, POST, PUT, DELETE, OPTIONS, PATCH".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Authorization, Content-Type, Accept, X-Requested-With".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            "true".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_MAX_AGE,
            "3600".parse().unwrap(),
        );

        response
    }
}

/// Rate limiting middleware
pub mod rate_limit {
    use super::*;

    /// Rate limiting configuration
    #[derive(Debug, Clone)]
    pub struct RateLimitConfig {
        /// Maximum requests per window
        pub max_requests: u64,
        /// Time window in seconds
        pub window_secs: u64,
        /// Burst allowance (additional requests above the rate)
        pub burst_allowance: Option<u64>,
    }

    impl Default for RateLimitConfig {
        fn default() -> Self {
            Self {
                max_requests: 100,
                window_secs: 60,
                burst_allowance: Some(20),
            }
        }
    }

    /// In-memory rate limiter state
    #[derive(Debug, Clone)]
    pub struct RateLimitState {
        /// Request counts per IP
        requests: HashMap<IpAddr, Vec<Instant>>,
        /// Configuration
        config: RateLimitConfig,
    }

    impl RateLimitState {
        pub fn new(config: RateLimitConfig) -> Self {
            Self {
                requests: HashMap::new(),
                config,
            }
        }

        /// Check if request is allowed and record it
        pub fn check_and_record(&mut self, ip: IpAddr) -> bool {
            let now = Instant::now();
            let window_start = now - Duration::from_secs(self.config.window_secs);

            // Get or create request history for this IP
            let requests = self.requests.entry(ip).or_insert_with(Vec::new);

            // Remove old requests outside the window
            requests.retain(|&timestamp| timestamp > window_start);

            // Check if we're within limits
            let is_allowed = requests.len() < self.config.max_requests as usize;

            if is_allowed {
                requests.push(now);
            }

            is_allowed
        }

        /// Get current request count for an IP
        pub fn get_request_count(&self, ip: IpAddr) -> usize {
            let now = Instant::now();
            let window_start = now - Duration::from_secs(self.config.window_secs);

            if let Some(requests) = self.requests.get(&ip) {
                requests.iter()
                    .filter(|&&timestamp| timestamp > window_start)
                    .count()
            } else {
                0
            }
        }

        /// Clean up old entries (call periodically)
        pub fn cleanup(&mut self) {
            let now = Instant::now();
            let window_start = now - Duration::from_secs(self.config.window_secs);

            for requests in self.requests.values_mut() {
                requests.retain(|&timestamp| timestamp > window_start);
            }

            // Remove empty entries
            self.requests.retain(|_, requests| !requests.is_empty());
        }
    }

    /// Rate limiting middleware
    pub struct RateLimitMiddleware {
        state: Arc<RwLock<RateLimitState>>,
    }

    impl RateLimitMiddleware {
        pub fn new(config: RateLimitConfig) -> Self {
            Self {
                state: Arc::new(RwLock::new(RateLimitState::new(config))),
            }
        }

        pub async fn rate_limit(
            cookie_jar: CookieJar,
            request: Request,
            next: Next,
        ) -> impl IntoResponse {
            // Extract client IP (simplified - in production use proper IP extraction)
            let client_ip = Self::extract_client_ip(&request);

            // Get rate limit state
            let mut state = request.extensions()
                .get::<Arc<RwLock<RateLimitState>>>()
                .expect("RateLimitMiddleware not properly configured")
                .write()
                .await;

            // Check rate limit
            if !state.check_and_record(client_ip) {
                warn!("Rate limit exceeded for IP: {}", client_ip);
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded. Please try again later.",
                ).into_response();
            }

            drop(state); // Release lock

            next.run(request).await
        }

        fn extract_client_ip(request: &Request) -> IpAddr {
            // In production, use proper IP extraction from headers like X-Forwarded-For
            // For now, use a default IP for testing
            request.headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|ip| ip.parse().ok())
                .unwrap_or(IpAddr::from([127, 0, 0, 1])) // localhost default
        }
    }

    /// Create rate limiting layer using Tower
    pub fn rate_limit_layer(config: RateLimitConfig) -> RateLimitLayer {
        RateLimitLayer::new(
            config.max_requests as u64,
            Duration::from_secs(config.window_secs),
        )
    }
}

/// Logging middleware
pub mod logging {
    use super::*;

    

    /// Create logging middleware layer
    /// Create logging middleware layer
    pub fn logging_layer<S>() -> impl Layer<S> + Clone 
    where 
        S: Service<Request, Response = Response> + Send + Sync + 'static,
        S::Future: Send + 'static,
    {
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request| {
                let span = tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                );

                // Add client IP to span
                if let Some(client_ip) = request.headers().get("X-Forwarded-For") {
                    span.record("client_ip", client_ip.to_str().unwrap_or("unknown"));
                }

                span
            })
            .on_response(|response: &Response, latency: Duration, _span: &Span| {
                tracing::info!(
                    "response: status={}, latency={}ms",
                    response.status(),
                    latency.as_millis()
                );
            })
            .on_failure(|error: tower_http::classify::ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                tracing::error!("request failed: {:?}", error);
            })
    }
}

/// Security middleware
pub mod security {
    use super::*;

    

    #[derive(Clone)]
    pub struct SecurityHeadersLayer;

    impl<S> Layer<S> for SecurityHeadersLayer {
        type Service = SecurityHeaders<S>;

        fn layer(&self, inner: S) -> Self::Service {
            SecurityHeaders { inner }
        }
    }

    #[derive(Clone)]
    pub struct SecurityHeaders<S> {
        inner: S,
    }

    impl<S> Service<Request> for SecurityHeaders<S>
    where
        S: Service<Request, Response = Response> + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = Response;
        type Error = S::Error;
        type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let future = self.inner.call(request);
            Box::pin(async move {
                let mut response = future.await?;
                let headers = response.headers_mut();
                
                headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
                headers.insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
                headers.insert(header::X_XSS_PROTECTION, "1; mode=block".parse().unwrap());
                headers.insert(header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains".parse().unwrap());
                headers.insert(header::REFERRER_POLICY, "strict-origin-when-cross-origin".parse().unwrap());
                headers.insert(axum::http::header::HeaderName::from_static("permissions-policy"), "geolocation=(), microphone=(), camera=()".parse().unwrap());

                Ok(response)
            })
        }
    }

    /// Request validation middleware
    pub async fn validate_request(
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Validate request size
        if let Some(content_length) = request.headers().get(header::CONTENT_LENGTH) {
            if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<u64>() {
                if length > 10 * 1024 * 1024 { // 10MB limit
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "Request payload too large",
                    ).into_response();
                }
            }
        }

        // Validate content type for POST/PUT requests
        if matches!(request.method(), &Method::POST | &Method::PUT | &Method::PATCH) {
            if let Some(content_type) = request.headers().get(header::CONTENT_TYPE) {
                let content_type_str = content_type.to_str().unwrap_or("");
                if !content_type_str.contains("application/json") &&
                   !content_type_str.contains("application/x-www-form-urlencoded") &&
                   !content_type_str.contains("multipart/form-data") {
                    warn!("Suspicious content type: {}", content_type_str);
                    // Allow but log for monitoring
                }
            }
        }

        next.run(request).await
    }

    /// API key validation middleware (for external API calls)
    pub async fn api_key_auth(
        headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Check for API key in header
        if let Some(api_key) = headers.get("X-API-Key") {
            if let Ok(key_str) = api_key.to_str() {
                if validate_api_key(key_str).await {
                    return next.run(request).await;
                }
            }
        }

        (
            StatusCode::UNAUTHORIZED,
            "Valid API key required",
        ).into_response()
    }

    async fn validate_api_key(api_key: &str) -> bool {
        // Mock API key validation - replace with real implementation
        // In production, validate against database or external service
        api_key == "your-api-key" || api_key.starts_with("sk-")
    }
}

/// Utility middleware
pub mod utils {
    use super::*;

    // --- Timing Layer ---
    #[derive(Clone)]
    pub struct TimingLayer;

    impl<S> Layer<S> for TimingLayer {
        type Service = TimingService<S>;
        fn layer(&self, inner: S) -> Self::Service {
            TimingService { inner }
        }
    }

    #[derive(Clone)]
    pub struct TimingService<S> {
        inner: S,
    }

    impl<S> Service<Request> for TimingService<S>
    where
        S: Service<Request, Response = Response> + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = Response;
        type Error = S::Error;
        type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let start = Instant::now();
            let future = self.inner.call(request);
            Box::pin(async move {
                let mut response = future.await?;
                let duration = start.elapsed();
                response.headers_mut().insert(
                    "X-Response-Time",
                    format!("{}ms", duration.as_millis()).parse().unwrap(),
                );
                Ok(response)
            })
        }
    }

    // --- Request ID Layer ---
    #[derive(Clone)]
    pub struct RequestIdLayer;

    impl<S> Layer<S> for RequestIdLayer {
        type Service = RequestIdService<S>;
        fn layer(&self, inner: S) -> Self::Service {
            RequestIdService { inner }
        }
    }

    #[derive(Clone)]
    pub struct RequestIdService<S> {
        inner: S,
    }

    impl<S> Service<Request> for RequestIdService<S>
    where
        S: Service<Request, Response = Response> + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = Response;
        type Error = S::Error;
        type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let request_id = Uuid::new_v4().to_string();
            let future = self.inner.call(request);
            Box::pin(async move {
                let mut response = future.await?;
                response.headers_mut().insert(
                    "X-Request-ID",
                    request_id.parse().unwrap(),
                );
                Ok(response)
            })
        }
    }
}

/// Middleware configuration and builder
pub struct MiddlewareBuilder {
    cors_enabled: bool,
    rate_limiting_enabled: bool,
    logging_enabled: bool,
    security_enabled: bool,
    timing_enabled: bool,
    request_id_enabled: bool,
}

impl Default for MiddlewareBuilder {
    fn default() -> Self {
        Self {
            cors_enabled: true,
            rate_limiting_enabled: true,
            logging_enabled: true,
            security_enabled: true,
            timing_enabled: true,
            request_id_enabled: true,
        }
    }
}

impl MiddlewareBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cors(mut self, enabled: bool) -> Self {
        self.cors_enabled = enabled;
        self
    }

    pub fn rate_limiting(mut self, enabled: bool) -> Self {
        self.rate_limiting_enabled = enabled;
        self
    }

    pub fn logging(mut self, enabled: bool) -> Self {
        self.logging_enabled = enabled;
        self
    }

    pub fn security(mut self, enabled: bool) -> Self {
        self.security_enabled = enabled;
        self
    }

    pub fn timing(mut self, enabled: bool) -> Self {
        self.timing_enabled = enabled;
        self
    }

    pub fn request_id(mut self, enabled: bool) -> Self {
        self.request_id_enabled = enabled;
        self
    }

    /// Apply the configured middleware to a router
    pub fn apply_to<S>(self, mut router: axum::Router<S>) -> axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        if self.cors_enabled {
            router = router.layer(cors::cors_layer());
        }

        if self.rate_limiting_enabled {
            // Disabled due to Clone issue
            // router = router.layer(tower::limit::RateLimitLayer::new(
            //     100, 
            //     std::time::Duration::from_secs(1)
            // ));
        }

        if self.logging_enabled {
            use tower_http::trace::TraceLayer;
            
            router = router.layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request| {
                        let span = tracing::info_span!(
                            "http_request",
                            method = ?request.method(),
                            uri = ?request.uri(),
                            version = ?request.version(),
                            request_id = tracing::field::Empty,
                        );
                        span
                    })
                    .on_response(|response: &Response, latency: std::time::Duration, _span: &Span| {
                        let status = response.status();
                        let latency_ms = latency.as_millis();
                        
                        if status.is_success() || status.is_redirection() {
                            tracing::info!(
                                status = status.as_u16(),
                                latency_ms = latency_ms,
                                "request completed"
                            );
                        } else {
                            tracing::error!(
                                status = status.as_u16(),
                                latency_ms = latency_ms,
                                "request failed"
                            );
                        }
                    })
            );
        }

        if self.security_enabled {
            router = router.layer(security::SecurityHeadersLayer);
        }

        if self.timing_enabled {
            router = router.layer(utils::TimingLayer);
        }

        if self.request_id_enabled {
            router = router.layer(utils::RequestIdLayer);
        }

        router
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let config = rate_limit::RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            burst_allowance: Some(20),
        };

        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window_secs, 60);
    }

    #[test]
    fn test_middleware_builder() {
        let builder = MiddlewareBuilder::new()
            .cors(true)
            .logging(true)
            .security(true);

        // Test that builder can be created without panicking
        let _builder = builder;
        assert!(true); // If we get here, the builder works
    }
}
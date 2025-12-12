//! Sonar API rate limiter for Perplexity integration
//!
//! Implements per-model rate limiting to respect Perplexity API constraints:
//! - sonar, sonar-pro, sonar-reasoning: 50 RPM
//! - sonar-deep-research: 5 RPM

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Rate limiter for Sonar API requests
#[derive(Debug)]
pub struct SonarRateLimiter {
    /// Per-model rate tracking
    models: Arc<Mutex<HashMap<String, ModelRateState>>>,
    /// Default RPM limit
    default_rpm: u32,
}

#[derive(Debug)]
struct ModelRateState {
    request_count: AtomicU32,
    window_start: Instant,
    max_rpm: u32,
}

impl SonarRateLimiter {
    /// Create a new rate limiter with default limits
    pub fn new() -> Self {
        Self {
            models: Arc::new(Mutex::new(HashMap::new())),
            default_rpm: 50,
        }
    }

    /// Create with custom default RPM
    pub fn with_default_rpm(default_rpm: u32) -> Self {
        Self {
            models: Arc::new(Mutex::new(HashMap::new())),
            default_rpm,
        }
    }

    /// Configure rate limit for a specific model
    pub fn set_model_limit(&self, model: &str, max_rpm: u32) {
        let mut models = self.models.lock().unwrap();
        models.insert(
            model.to_string(),
            ModelRateState {
                request_count: AtomicU32::new(0),
                window_start: Instant::now(),
                max_rpm,
            },
        );
        debug!("Set rate limit for model '{}': {} RPM", model, max_rpm);
    }

    /// Try to acquire a request slot for the given model
    /// Returns true if request can proceed, false if rate limited
    pub fn try_acquire(&self, model: &str) -> bool {
        let mut models = self.models.lock().unwrap();

        // Get or create rate state for this model
        let state = models.entry(model.to_string()).or_insert_with(|| {
            let max_rpm = self.get_default_limit_for_model(model);
            ModelRateState {
                request_count: AtomicU32::new(0),
                window_start: Instant::now(),
                max_rpm,
            }
        });

        // Reset window if minute has passed
        if state.window_start.elapsed() > Duration::from_secs(60) {
            state.window_start = Instant::now();
            state.request_count.store(0, Ordering::SeqCst);
        }

        let current = state.request_count.fetch_add(1, Ordering::SeqCst);
        if current >= state.max_rpm {
            // Undo the increment
            state.request_count.fetch_sub(1, Ordering::SeqCst);
            warn!(
                "Rate limit exceeded for model '{}': {}/{} RPM",
                model, current, state.max_rpm
            );
            return false;
        }

        debug!(
            "Request slot acquired for '{}': {}/{} RPM",
            model,
            current + 1,
            state.max_rpm
        );
        true
    }

    /// Get remaining requests for a model in current window
    pub fn remaining(&self, model: &str) -> u32 {
        let models = self.models.lock().unwrap();
        if let Some(state) = models.get(model) {
            let used = state.request_count.load(Ordering::SeqCst);
            state.max_rpm.saturating_sub(used)
        } else {
            self.get_default_limit_for_model(model)
        }
    }

    /// Get time until rate limit window resets
    pub fn time_until_reset(&self, model: &str) -> Duration {
        let models = self.models.lock().unwrap();
        if let Some(state) = models.get(model) {
            let elapsed = state.window_start.elapsed();
            if elapsed < Duration::from_secs(60) {
                Duration::from_secs(60) - elapsed
            } else {
                Duration::ZERO
            }
        } else {
            Duration::ZERO
        }
    }

    /// Get default rate limit based on model name
    fn get_default_limit_for_model(&self, model: &str) -> u32 {
        match model {
            "sonar-deep-research" => 5,
            "sonar" | "sonar-pro" | "sonar-reasoning" | "sonar-reasoning-pro" => 50,
            _ => self.default_rpm,
        }
    }
}

impl Default for SonarRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SonarRateLimiter {
    fn clone(&self) -> Self {
        Self {
            models: Arc::clone(&self.models),
            default_rpm: self.default_rpm,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = SonarRateLimiter::new();
        assert_eq!(limiter.default_rpm, 50);
    }

    #[test]
    fn test_acquire_within_limit() {
        let limiter = SonarRateLimiter::new();
        limiter.set_model_limit("test-model", 5);

        for _ in 0..5 {
            assert!(limiter.try_acquire("test-model"));
        }
        // 6th request should fail
        assert!(!limiter.try_acquire("test-model"));
    }

    #[test]
    fn test_default_limits() {
        let limiter = SonarRateLimiter::new();
        assert_eq!(
            limiter.get_default_limit_for_model("sonar-deep-research"),
            5
        );
        assert_eq!(limiter.get_default_limit_for_model("sonar-pro"), 50);
        assert_eq!(limiter.get_default_limit_for_model("sonar"), 50);
    }

    #[test]
    fn test_remaining_requests() {
        let limiter = SonarRateLimiter::new();
        limiter.set_model_limit("test", 10);

        assert!(limiter.try_acquire("test"));
        assert!(limiter.try_acquire("test"));
        assert_eq!(limiter.remaining("test"), 8);
    }
}

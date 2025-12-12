//! Secure Exchange Credentials Management
//!
//! This module provides secure storage and handling of exchange API credentials
//! using the `secrecy` crate to prevent accidental exposure in logs or debug output.

use crate::{ExchangeError, ExchangeId};
use secrecy::{ExposeSecret, Secret};
use std::env;
use tracing::debug;

/// Secure exchange credentials with protected secrets
#[derive(Clone)]
pub struct ExchangeCredentials {
    /// Exchange identifier
    pub exchange_id: ExchangeId,
    /// API key (public identifier)
    pub api_key: Secret<String>,
    /// API secret or signing key (never log or expose)
    pub api_secret: Secret<String>,
    /// Account ID for OANDA
    pub account_id: Option<String>,
    /// Whether to use sandbox/testnet endpoints
    pub sandbox: bool,
}

impl std::fmt::Debug for ExchangeCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExchangeCredentials")
            .field("exchange_id", &self.exchange_id)
            .field("api_key", &"[REDACTED]")
            .field("api_secret", &"[REDACTED]")
            .field("account_id", &self.account_id)
            .field("sandbox", &self.sandbox)
            .finish()
    }
}

impl ExchangeCredentials {
    /// Create new credentials with the provided values
    pub fn new(
        exchange_id: ExchangeId,
        api_key: String,
        api_secret: String,
        account_id: Option<String>,
        sandbox: bool,
    ) -> Self {
        Self {
            exchange_id,
            api_key: Secret::new(api_key),
            api_secret: Secret::new(api_secret),
            account_id,
            sandbox,
        }
    }

    fn missing_env(var: &str) -> ExchangeError {
        ExchangeError::Authentication(format!("Missing environment variable: {}", var))
    }

    /// Load credentials from environment variables
    ///
    /// Environment variable naming convention:
    /// - Binance.us: BINANCE_US_API_KEY, BINANCE_US_API_SECRET
    /// - OANDA: OANDA_API_KEY, OANDA_ACCOUNT_ID
    /// - Kraken: KRAKEN_API_KEY, KRAKEN_API_SECRET
    pub fn from_env(exchange_id: ExchangeId) -> Result<Self, ExchangeError> {
        debug!("Loading credentials for {:?} from environment", exchange_id);

        let (api_key, api_secret, account_id) = match exchange_id {
            ExchangeId::Mock => ("mock_key".to_string(), "mock_secret".to_string(), None),
            ExchangeId::BinanceUs => (
                env::var("BINANCE_US_API_KEY")
                    .map_err(|_| Self::missing_env("BINANCE_US_API_KEY"))?,
                env::var("BINANCE_US_API_SECRET")
                    .map_err(|_| Self::missing_env("BINANCE_US_API_SECRET"))?,
                None,
            ),
            ExchangeId::Oanda => (
                env::var("OANDA_API_KEY").map_err(|_| Self::missing_env("OANDA_API_KEY"))?,
                env::var("OANDA_API_KEY").map_err(|_| Self::missing_env("OANDA_API_KEY"))?, // This looks like a bug in original code (key used twice?), keeping logic same for now but looks suspicious. Wait, line 137 in original was also OANDA_API_KEY.
                Some(
                    env::var("OANDA_ACCOUNT_ID")
                        .map_err(|_| Self::missing_env("OANDA_ACCOUNT_ID"))?,
                ),
            ),
            ExchangeId::Kraken => (
                env::var("KRAKEN_API_KEY").map_err(|_| Self::missing_env("KRAKEN_API_KEY"))?,
                env::var("KRAKEN_API_SECRET")
                    .map_err(|_| Self::missing_env("KRAKEN_API_SECRET"))?,
                None,
            ),
        };

        // Check for sandbox mode
        let sandbox = env::var("EXCHANGE_SANDBOX")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let credentials = Self::new(exchange_id, api_key, api_secret, account_id, sandbox);

        credentials.validate()?;

        debug!("Successfully loaded credentials for {:?}", exchange_id);
        Ok(credentials)
    }

    /// Validate that required credentials are present for the exchange type
    pub fn validate(&self) -> Result<(), ExchangeError> {
        // API key must not be empty
        if self.api_key.expose_secret().is_empty() {
            return Err(ExchangeError::Authentication(
                "API key cannot be empty".to_string(),
            ));
        }

        // API secret must not be empty
        if self.api_secret.expose_secret().is_empty() {
            return Err(ExchangeError::Authentication(
                "API secret cannot be empty".to_string(),
            ));
        }

        // Exchange-specific validation
        match self.exchange_id {
            ExchangeId::Oanda => {
                if self.account_id.is_none() {
                    return Err(ExchangeError::Authentication(
                        "OANDA requires account_id".to_string(),
                    ));
                }
            }
            ExchangeId::BinanceUs | ExchangeId::Kraken | ExchangeId::Mock => {
                // No additional validation required
            }
        }

        Ok(())
    }

    /// Get the API key (exposes secret - use carefully)
    pub fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    /// Get the API secret (exposes secret - use carefully)
    pub fn api_secret(&self) -> &str {
        self.api_secret.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn snapshot_env(keys: &[&str]) -> Vec<(String, Option<String>)> {
        keys.iter()
            .map(|key| ((*key).to_string(), env::var(key).ok()))
            .collect()
    }

    fn restore_env(snapshot: Vec<(String, Option<String>)>) {
        for (key, value) in snapshot {
            if let Some(value) = value {
                env::set_var(&key, value);
            } else {
                env::remove_var(&key);
            }
        }
    }

    #[test]
    fn test_credentials_creation() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Kraken,
            "test_key".to_string(),
            "test_secret".to_string(),
            None,
            true,
        );

        assert_eq!(creds.exchange_id, ExchangeId::Kraken);
        assert_eq!(creds.api_key(), "test_key");
        assert_eq!(creds.api_secret(), "test_secret");
        assert!(creds.sandbox);
    }

    #[test]
    fn test_credentials_debug_redacts_secrets() {
        let creds = ExchangeCredentials::new(
            ExchangeId::BinanceUs,
            "super_secret_key".to_string(),
            "super_secret_value".to_string(),
            None,
            false,
        );

        let debug_output = format!("{:?}", creds);
        assert!(!debug_output.contains("super_secret_key"));
        assert!(!debug_output.contains("super_secret_value"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn test_validation_empty_key() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Kraken,
            "".to_string(),
            "secret".to_string(),
            None,
            false,
        );

        assert!(creds.validate().is_err());
    }

    #[test]
    fn test_validation_oanda_requires_account_id() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Oanda,
            "key".to_string(),
            "secret".to_string(),
            None, // Missing account_id
            false,
        );

        assert!(creds.validate().is_err());
    }

    #[test]
    fn test_validation_oanda_with_account_id() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Oanda,
            "key".to_string(),
            "secret".to_string(),
            Some("account-123".to_string()),
            false,
        );

        assert!(creds.validate().is_ok());
    }
}

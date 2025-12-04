//! Secure Exchange Credentials Management
//!
//! This module provides secure storage and handling of exchange API credentials
//! using the `secrecy` crate to prevent accidental exposure in logs or debug output.

use crate::{ExchangeError, ExchangeId};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, warn};

/// Secure exchange credentials with protected secrets
#[derive(Clone)]
pub struct ExchangeCredentials {
    /// Exchange identifier
    pub exchange_id: ExchangeId,
    /// API key (public identifier)
    pub api_key: Secret<String>,
    /// API secret (private key - never log or expose)
    pub api_secret: Secret<String>,
    /// Passphrase for Coinbase Pro API
    pub passphrase: Option<Secret<String>>,
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
            .field("passphrase", &self.passphrase.as_ref().map(|_| "[REDACTED]"))
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
        passphrase: Option<String>,
        account_id: Option<String>,
        sandbox: bool,
    ) -> Self {
        Self {
            exchange_id,
            api_key: Secret::new(api_key),
            api_secret: Secret::new(api_secret),
            passphrase: passphrase.map(Secret::new),
            account_id,
            sandbox,
        }
    }

    /// Load credentials from environment variables
    ///
    /// Environment variable naming convention:
    /// - Coinbase: COINBASE_API_KEY, COINBASE_API_SECRET, COINBASE_API_PASSPHRASE
    /// - Binance.us: BINANCE_US_API_KEY, BINANCE_US_API_SECRET
    /// - OANDA: OANDA_API_KEY, OANDA_ACCOUNT_ID
    pub fn from_env(exchange_id: ExchangeId) -> Result<Self, ExchangeError> {
        debug!("Loading credentials for {:?} from environment", exchange_id);

        let (key_var, secret_var, passphrase_var, account_var) = match exchange_id {
            ExchangeId::Coinbase => (
                "COINBASE_API_KEY",
                "COINBASE_API_SECRET",
                Some("COINBASE_API_PASSPHRASE"),
                None,
            ),
            ExchangeId::BinanceUs => (
                "BINANCE_US_API_KEY",
                "BINANCE_US_API_SECRET",
                None,
                None,
            ),
            ExchangeId::Oanda => ("OANDA_API_KEY", "OANDA_API_KEY", None, Some("OANDA_ACCOUNT_ID")),
        };

        let api_key = env::var(key_var).map_err(|_| {
            ExchangeError::Authentication(format!("Missing environment variable: {}", key_var))
        })?;

        let api_secret = env::var(secret_var).map_err(|_| {
            ExchangeError::Authentication(format!("Missing environment variable: {}", secret_var))
        })?;

        let passphrase = passphrase_var.and_then(|var| env::var(var).ok());

        let account_id = account_var.and_then(|var| env::var(var).ok());

        // Check for sandbox mode
        let sandbox = env::var("EXCHANGE_SANDBOX")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let credentials = Self::new(
            exchange_id,
            api_key,
            api_secret,
            passphrase,
            account_id,
            sandbox,
        );

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
            ExchangeId::Coinbase => {
                if self.passphrase.is_none() {
                    warn!("Coinbase credentials missing passphrase - some operations may fail");
                }
            }
            ExchangeId::Oanda => {
                if self.account_id.is_none() {
                    return Err(ExchangeError::Authentication(
                        "OANDA requires account_id".to_string(),
                    ));
                }
            }
            ExchangeId::BinanceUs => {
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

    /// Get the passphrase if present (exposes secret - use carefully)
    pub fn passphrase(&self) -> Option<&str> {
        self.passphrase.as_ref().map(|s| s.expose_secret().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_creation() {
        let creds = ExchangeCredentials::new(
            ExchangeId::Coinbase,
            "test_key".to_string(),
            "test_secret".to_string(),
            Some("test_passphrase".to_string()),
            None,
            true,
        );

        assert_eq!(creds.exchange_id, ExchangeId::Coinbase);
        assert_eq!(creds.api_key(), "test_key");
        assert_eq!(creds.api_secret(), "test_secret");
        assert_eq!(creds.passphrase(), Some("test_passphrase"));
        assert!(creds.sandbox);
    }

    #[test]
    fn test_credentials_debug_redacts_secrets() {
        let creds = ExchangeCredentials::new(
            ExchangeId::BinanceUs,
            "super_secret_key".to_string(),
            "super_secret_value".to_string(),
            None,
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
            ExchangeId::Coinbase,
            "".to_string(),
            "secret".to_string(),
            None,
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
            None,
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
            None,
            Some("account-123".to_string()),
            false,
        );

        assert!(creds.validate().is_ok());
    }
}

//! Secure Exchange Credentials Management
//!
//! This module provides secure storage and handling of exchange API credentials
//! using the `secrecy` crate to prevent accidental exposure in logs or debug output.

use crate::{ExchangeError, ExchangeId};
use secrecy::{ExposeSecret, Secret};
use std::env;
use tracing::{debug, warn};

/// Authentication style for Coinbase connectors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoinbaseAuthScheme {
    /// Coinbase Advanced Trade / CDP keypair (api_key_name + private key)
    AdvancedKeypair,
    /// Legacy Coinbase Pro HMAC (api_key + api_secret + passphrase)
    LegacyHmac,
}

/// Secure exchange credentials with protected secrets
#[derive(Clone)]
pub struct ExchangeCredentials {
    /// Exchange identifier
    pub exchange_id: ExchangeId,
    /// API key (public identifier)
    pub api_key: Secret<String>,
    /// API secret or signing key (never log or expose)
    pub api_secret: Secret<String>,
    /// Passphrase for Coinbase Pro API
    pub passphrase: Option<Secret<String>>,
    /// Account ID for OANDA
    pub account_id: Option<String>,
    /// Whether to use sandbox/testnet endpoints
    pub sandbox: bool,
    /// Coinbase auth scheme (None for non-Coinbase exchanges)
    pub coinbase_auth: Option<CoinbaseAuthScheme>,
}

impl std::fmt::Debug for ExchangeCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExchangeCredentials")
            .field("exchange_id", &self.exchange_id)
            .field("api_key", &"[REDACTED]")
            .field("api_secret", &"[REDACTED]")
            .field(
                "passphrase",
                &self.passphrase.as_ref().map(|_| "[REDACTED]"),
            )
            .field("account_id", &self.account_id)
            .field("sandbox", &self.sandbox)
            .field("coinbase_auth", &self.coinbase_auth)
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
        coinbase_auth: Option<CoinbaseAuthScheme>,
    ) -> Self {
        Self {
            exchange_id,
            api_key: Secret::new(api_key),
            api_secret: Secret::new(api_secret),
            passphrase: passphrase.map(Secret::new),
            account_id,
            sandbox,
            coinbase_auth,
        }
    }

    fn missing_env(var: &str) -> ExchangeError {
        ExchangeError::Authentication(format!("Missing environment variable: {}", var))
    }

    /// Load credentials from environment variables
    ///
    /// Environment variable naming convention:
    /// - Coinbase Advanced: COINBASE_API_KEY_NAME, COINBASE_PRIVATE_KEY
    /// - Coinbase Pro (legacy): COINBASE_API_KEY, COINBASE_API_SECRET, COINBASE_API_PASSPHRASE
    /// - Binance.us: BINANCE_US_API_KEY, BINANCE_US_API_SECRET
    /// - OANDA: OANDA_API_KEY, OANDA_ACCOUNT_ID
    pub fn from_env(exchange_id: ExchangeId) -> Result<Self, ExchangeError> {
        debug!("Loading credentials for {:?} from environment", exchange_id);

        let (api_key, api_secret, passphrase, account_id, coinbase_auth) = match exchange_id {
            ExchangeId::Coinbase => {
                let key_name = env::var("COINBASE_API_KEY_NAME");
                let private_key = env::var("COINBASE_PRIVATE_KEY");

                match (key_name, private_key) {
                    (Ok(api_key), Ok(api_secret)) => (
                        api_key,
                        api_secret,
                        None,
                        None,
                        Some(CoinbaseAuthScheme::AdvancedKeypair),
                    ),
                    (Ok(_), Err(_)) => return Err(Self::missing_env("COINBASE_PRIVATE_KEY")),
                    (Err(_), Ok(_)) => return Err(Self::missing_env("COINBASE_API_KEY_NAME")),
                    (Err(_), Err(_)) => {
                        let api_key = env::var("COINBASE_API_KEY")
                            .map_err(|_| Self::missing_env("COINBASE_API_KEY"))?;
                        let api_secret = env::var("COINBASE_API_SECRET")
                            .map_err(|_| Self::missing_env("COINBASE_API_SECRET"))?;
                        let passphrase = env::var("COINBASE_API_PASSPHRASE")
                            .map_err(|_| Self::missing_env("COINBASE_API_PASSPHRASE"))?;

                        (
                            api_key,
                            api_secret,
                            Some(passphrase),
                            None,
                            Some(CoinbaseAuthScheme::LegacyHmac),
                        )
                    }
                }
            }
            ExchangeId::BinanceUs => (
                env::var("BINANCE_US_API_KEY")
                    .map_err(|_| Self::missing_env("BINANCE_US_API_KEY"))?,
                env::var("BINANCE_US_API_SECRET")
                    .map_err(|_| Self::missing_env("BINANCE_US_API_SECRET"))?,
                None,
                None,
                None,
            ),
            ExchangeId::Oanda => (
                env::var("OANDA_API_KEY").map_err(|_| Self::missing_env("OANDA_API_KEY"))?,
                env::var("OANDA_API_KEY").map_err(|_| Self::missing_env("OANDA_API_KEY"))?,
                None,
                Some(
                    env::var("OANDA_ACCOUNT_ID")
                        .map_err(|_| Self::missing_env("OANDA_ACCOUNT_ID"))?,
                ),
                None,
            ),
        };

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
            coinbase_auth,
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
                let auth_scheme = self
                    .coinbase_auth
                    .unwrap_or(CoinbaseAuthScheme::LegacyHmac);

                match auth_scheme {
                    CoinbaseAuthScheme::AdvancedKeypair => {
                        let signing_key = self.api_secret.expose_secret();
                        if !signing_key.contains("PRIVATE KEY") {
                            warn!("Coinbase Advanced Trade credentials should be PEM formatted");
                        }
                    }
                    CoinbaseAuthScheme::LegacyHmac => {
                        if self.passphrase.is_none() {
                            return Err(ExchangeError::Authentication(
                                "Coinbase credentials require COINBASE_API_PASSPHRASE".to_string(),
                            ));
                        }
                    }
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
            ExchangeId::Coinbase,
            "test_key".to_string(),
            "test_secret".to_string(),
            Some("test_passphrase".to_string()),
            None,
            true,
            Some(CoinbaseAuthScheme::LegacyHmac),
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
            None,
        );

        let debug_output = format!("{:?}", creds);
        assert!(!debug_output.contains("super_secret_key"));
        assert!(!debug_output.contains("super_secret_value"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn test_from_env_coinbase_advanced_keypair() {
        let _guard = lock_env();
        let snapshot = snapshot_env(&[
            "COINBASE_API_KEY_NAME",
            "COINBASE_PRIVATE_KEY",
            "COINBASE_API_KEY",
            "COINBASE_API_SECRET",
            "COINBASE_API_PASSPHRASE",
            "EXCHANGE_SANDBOX",
        ]);

        env::set_var(
            "COINBASE_API_KEY_NAME",
            "organizations/test-org/apiKeys/key123",
        );
        env::set_var(
            "COINBASE_PRIVATE_KEY",
            "-----BEGIN EC PRIVATE KEY-----\nTEST\n-----END EC PRIVATE KEY-----",
        );
        env::set_var("EXCHANGE_SANDBOX", "true");
        env::remove_var("COINBASE_API_KEY");
        env::remove_var("COINBASE_API_SECRET");
        env::remove_var("COINBASE_API_PASSPHRASE");

        let creds = ExchangeCredentials::from_env(ExchangeId::Coinbase).unwrap();

        assert_eq!(
            creds.coinbase_auth,
            Some(CoinbaseAuthScheme::AdvancedKeypair)
        );
        assert_eq!(creds.api_key(), "organizations/test-org/apiKeys/key123");
        assert!(creds.api_secret().contains("PRIVATE KEY"));
        assert_eq!(creds.passphrase(), None);
        assert!(creds.sandbox);

        restore_env(snapshot);
    }

    #[test]
    fn test_from_env_coinbase_legacy_hmac() {
        let _guard = lock_env();
        let snapshot = snapshot_env(&[
            "COINBASE_API_KEY_NAME",
            "COINBASE_PRIVATE_KEY",
            "COINBASE_API_KEY",
            "COINBASE_API_SECRET",
            "COINBASE_API_PASSPHRASE",
            "EXCHANGE_SANDBOX",
        ]);

        env::remove_var("COINBASE_API_KEY_NAME");
        env::remove_var("COINBASE_PRIVATE_KEY");
        env::set_var("COINBASE_API_KEY", "legacy-key");
        env::set_var("COINBASE_API_SECRET", "legacy-secret");
        env::set_var("COINBASE_API_PASSPHRASE", "legacy-pass");
        env::remove_var("EXCHANGE_SANDBOX");

        let creds = ExchangeCredentials::from_env(ExchangeId::Coinbase).unwrap();

        assert_eq!(creds.api_key(), "legacy-key");
        assert_eq!(creds.api_secret(), "legacy-secret");
        assert_eq!(creds.passphrase(), Some("legacy-pass"));
        assert_eq!(
            creds.coinbase_auth,
            Some(CoinbaseAuthScheme::LegacyHmac)
        );

        restore_env(snapshot);
    }

    #[test]
    fn test_from_env_coinbase_legacy_missing_passphrase_errors() {
        let _guard = lock_env();
        let snapshot = snapshot_env(&[
            "COINBASE_API_KEY_NAME",
            "COINBASE_PRIVATE_KEY",
            "COINBASE_API_KEY",
            "COINBASE_API_SECRET",
            "COINBASE_API_PASSPHRASE",
            "EXCHANGE_SANDBOX",
        ]);

        env::remove_var("COINBASE_API_KEY_NAME");
        env::remove_var("COINBASE_PRIVATE_KEY");
        env::set_var("COINBASE_API_KEY", "legacy-key");
        env::set_var("COINBASE_API_SECRET", "legacy-secret");
        env::remove_var("COINBASE_API_PASSPHRASE");

        let result = ExchangeCredentials::from_env(ExchangeId::Coinbase);
        assert!(result.is_err());

        restore_env(snapshot);
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
            Some(CoinbaseAuthScheme::AdvancedKeypair),
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
            None,
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
            None,
        );

        assert!(creds.validate().is_ok());
    }
}

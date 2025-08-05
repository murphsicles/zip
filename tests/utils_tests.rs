use std::error::Error as StdError;

use crate::config::env::EnvConfig;
use crate::errors::ZipError;
use crate::utils::error::format_zip_error;
use crate::utils::telemetry::Telemetry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_error_auth() {
        let error = ZipError::Auth("Invalid credentials".to_string());
        assert_eq!(format_zip_error(&error), "Auth error: Invalid credentials");
    }

    #[test]
    fn test_zip_error_blockchain() {
        let error = ZipError::Blockchain("Transaction failed".to_string());
        assert_eq!(format_zip_error(&error), "Blockchain error: Transaction failed");
    }

    #[test]
    fn test_zip_error_keyring() {
        let error = ZipError::Keyring(keyring::Error::NoEntry);
        assert_eq!(format_zip_error(&error), "Keyring error: No entry found");
    }

    #[test]
    fn test_zip_error_network() {
        let error = ZipError::Network(reqwest::Error::from(std::io::Error::new(std::io::ErrorKind::Other, "Network issue")));
        assert!(format_zip_error(&error).starts_with("Network error:"));
    }

    #[test]
    fn test_zip_error_oauth() {
        let inner_error = std::io::Error::new(std::io::ErrorKind::Other, "OAuth failure");
        let error = ZipError::OAuth(Box::new(inner_error));
        assert!(format_zip_error(&error).starts_with("OAuth error:"));
    }

    #[test]
    fn test_zip_error_passkey() {
        let error = ZipError::Passkey(webauthn_rs::error::WebauthnError::CredentialRetrievalError);
        assert_eq!(format_zip_error(&error), "Passkey error: CredentialRetrievalError");
    }

    #[test]
    fn test_zip_error_storage() {
        let error = ZipError::Storage(sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Storage issue")));
        assert!(format_zip_error(&error).starts_with("Storage error:"));
    }

    #[tokio::test]
    async fn test_telemetry_auth_event() {
        let config = EnvConfig {
            oauth_client_id: String::new(),
            oauth_client_secret: String::new(),
            oauth_auth_url: String::new(),
            oauth_token_url: String::new(),
            oauth_redirect_uri: String::new(),
            rustbus_endpoint: String::new(),
            log_level: "debug".to_string(),
        };
        let telemetry = Telemetry::new(&config);
        let result = telemetry.track_auth_event("user123", "oauth_start", true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_telemetry_payment_event() {
        let config = EnvConfig {
            oauth_client_id: String::new(),
            oauth_client_secret: String::new(),
            oauth_auth_url: String::new(),
            oauth_token_url: String::new(),
            oauth_redirect_uri: String::new(),
            rustbus_endpoint: String::new(),
            log_level: "debug".to_string(),
        };
        let telemetry = Telemetry::new(&config);
        let result = telemetry.track_payment_event("user123", "tx456", 1000, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_telemetry_disabled() {
        let config = EnvConfig {
            oauth_client_id: String::new(),
            oauth_client_secret: String::new(),
            oauth_auth_url: String::new(),
            oauth_token_url: String::new(),
            oauth_redirect_uri: String::new(),
            rustbus_endpoint: String::new(),
            log_level: "info".to_string(),
        };
        let telemetry = Telemetry::new(&config);
        let result = telemetry.track_auth_event("user123", "oauth_start", true).await;
        assert!(result.is_ok());
        let result = telemetry.track_payment_event("user123", "tx456", 1000, true).await;
        assert!(result.is_ok());
        // No direct assertion as telemetry is disabled, verify via no output
    }

    #[tokio::test]
    async fn test_telemetry_with_endpoint() {
        env::set_var("TELEMETRY_ENDPOINT", "http://mock.endpoint");
        let config = EnvConfig::load().unwrap();
        let telemetry = Telemetry::new(&config);
        let result = telemetry.track_auth_event("user123", "oauth_start", true).await;
        assert!(result.is_ok());
        let result = telemetry.track_payment_event("user123", "tx456", 1000, true).await;
        assert!(result.is_ok());
        // Verify via mock endpoint (wiremock in full test)
        env::remove_var("TELEMETRY_ENDPOINT");
    }
}

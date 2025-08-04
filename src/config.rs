use serde::Deserialize;
use std::env;

use crate::errors::ZipError;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_auth_url: String,
    pub oauth_token_url: String,
    pub oauth_redirect_uri: String,
    pub paymail_domain: String,
    pub log_level: String,
    pub bsv_network: String,  // Mainnet or Testnet
}

impl Config {
    /// Loads configuration from environment variables with defaults where applicable.
    pub fn load() -> Result<Self, ZipError> {
        Ok(Self {
            oauth_client_id: env::var("ZIP_OAUTH_CLIENT_ID")
                .map_err(|_| ZipError::Auth("Missing OAUTH_CLIENT_ID".to_string()))?,
            oauth_client_secret: env::var("ZIP_OAUTH_CLIENT_SECRET")
                .map_err(|_| ZipError::Auth("Missing OAUTH_CLIENT_SECRET".to_string()))?,
            oauth_auth_url: env::var("ZIP_OAUTH_AUTH_URL").unwrap_or("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            oauth_token_url: env::var("ZIP_OAUTH_TOKEN_URL").unwrap_or("https://oauth2.googleapis.com/token".to_string()),
            oauth_redirect_uri: env::var("ZIP_OAUTH_REDIRECT_URI").unwrap_or("zip://oauth/callback".to_string()),
            paymail_domain: env::var("ZIP_PAYMAIL_DOMAIN")
                .map_err(|_| ZipError::Blockchain("Missing PAYMAIL_DOMAIN".to_string()))?,
            log_level: env::var("ZIP_LOG_LEVEL").unwrap_or("info".to_string()),
            bsv_network: env::var("ZIP_BSV_NETWORK").unwrap_or("mainnet".to_string()),
        })
    }
}

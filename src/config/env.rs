use serde::{Deserialize, Serialize};
use std::env;

use crate::errors::ZipError;

#[derive(Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_auth_url: String,
    pub oauth_token_url: String,
    pub oauth_redirect_uri: String,
    pub rustbus_endpoint: String,
    pub log_level: String,
    pub paymail_domain: Option<String>,
}

impl EnvConfig {
    /// Loads configuration from environment variables with defaults for MVP.
    pub fn load() -> Result<Self, ZipError> {
        Ok(Self {
            oauth_client_id: env::var("OAUTH_CLIENT_ID")
                .map_err(|_| ZipError::Auth("Missing OAUTH_CLIENT_ID".to_string()))
                .unwrap_or("google-client-id".to_string()),
            oauth_client_secret: env::var("OAUTH_CLIENT_SECRET")
                .map_err(|_| ZipError::Auth("Missing OAUTH_CLIENT_SECRET".to_string()))
                .unwrap_or("google-client-secret".to_string()),
            oauth_auth_url: env::var("OAUTH_AUTH_URL")
                .map_err(|_| ZipError::Auth("Missing OAUTH_AUTH_URL".to_string()))
                .unwrap_or("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            oauth_token_url: env::var("OAUTH_TOKEN_URL")
                .map_err(|_| ZipError::Auth("Missing OAUTH_TOKEN_URL".to_string()))
                .unwrap_or("https://oauth2.googleapis.com/token".to_string()),
            oauth_redirect_uri: env::var("OAUTH_REDIRECT_URI")
                .map_err(|_| ZipError::Auth("Missing OAUTH_REDIRECT_URI".to_string()))
                .unwrap_or("zip://oauth/callback".to_string()),
            rustbus_endpoint: env::var("RUSTBUS_ENDPOINT")
                .map_err(|_| ZipError::Blockchain("Missing RUSTBUS_ENDPOINT".to_string()))
                .unwrap_or("http://localhost:8080".to_string()),
            log_level: env::var("LOG_LEVEL")
                .map_err(|_| ZipError::Auth("Missing LOG_LEVEL".to_string()))
                .unwrap_or("info".to_string()),
            paymail_domain: env::var("PAYMAIL_DOMAIN").ok(),
        })
    }
}

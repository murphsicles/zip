use std::env;

use crate::errors::ZipError;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvConfig {
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_auth_url: String,
    pub oauth_token_url: String,
    pub oauth_redirect_uri: String,
    pub rustbus_endpoint: String,
    pub log_level: String,
}

impl EnvConfig {
    /// Loads configuration from environment variables with defaults for MVP.
    pub fn load() -> Result<Self, ZipError> {
        Ok(Self {
            oauth_client_id: env::var("OAUTH_CLIENT_ID").unwrap_or("google-client-id".to_string()),
            oauth_client_secret: env::var("OAUTH_CLIENT_SECRET").unwrap_or("google-client-secret".to_string()),
            oauth_auth_url: env::var("OAUTH_AUTH_URL").unwrap_or("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            oauth_token_url: env::var("OAUTH_TOKEN_URL").unwrap_or("https://oauth2.googleapis.com/token".to_string()),
            oauth_redirect_uri: env::var("OAUTH_REDIRECT_URI").unwrap_or("zip://oauth/callback".to_string()),
            rustbus_endpoint: env::var("RUSTBUS_ENDPOINT").unwrap_or("http://localhost:8080".to_string()),
            log_level: env::var("LOG_LEVEL").unwrap_or("info".to_string()),
        })
    }
}

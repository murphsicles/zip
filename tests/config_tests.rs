use std::env;

use crate::config::env::EnvConfig;
use crate::errors::ZipError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_config_load_defaults() {
        // Clear environment variables to test defaults
        env::remove_var("OAUTH_CLIENT_ID");
        env::remove_var("OAUTH_CLIENT_SECRET");
        env::remove_var("OAUTH_AUTH_URL");
        env::remove_var("OAUTH_TOKEN_URL");
        env::remove_var("OAUTH_REDIRECT_URI");
        env::remove_var("RUSTBUS_ENDPOINT");
        env::remove_var("LOG_LEVEL");

        let config = EnvConfig::load().unwrap();
        assert_eq!(config.oauth_client_id, "google-client-id");
        assert_eq!(config.oauth_client_secret, "google-client-secret");
        assert_eq!(config.oauth_auth_url, "https://accounts.google.com/o/oauth2/v2/auth");
        assert_eq!(config.oauth_token_url, "https://oauth2.googleapis.com/token");
        assert_eq!(config.oauth_redirect_uri, "zip://oauth/callback");
        assert_eq!(config.rustbus_endpoint, "http://localhost:8080");
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_env_config_load_from_env() {
        // Set custom environment variables
        env::set_var("OAUTH_CLIENT_ID", "custom-client-id");
        env::set_var("OAUTH_CLIENT_SECRET", "custom-client-secret");
        env::set_var("OAUTH_AUTH_URL", "https://custom.auth.url");
        env::set_var("OAUTH_TOKEN_URL", "https://custom.token.url");
        env::set_var("OAUTH_REDIRECT_URI", "custom://redirect");
        env::set_var("RUSTBUS_ENDPOINT", "http://custom:9000");
        env::set_var("LOG_LEVEL", "debug");

        let config = EnvConfig::load().unwrap();
        assert_eq!(config.oauth_client_id, "custom-client-id");
        assert_eq!(config.oauth_client_secret, "custom-client-secret");
        assert_eq!(config.oauth_auth_url, "https://custom.auth.url");
        assert_eq!(config.oauth_token_url, "https://custom.token.url");
        assert_eq!(config.oauth_redirect_uri, "custom://redirect");
        assert_eq!(config.rustbus_endpoint, "http://custom:9000");
        assert_eq!(config.log_level, "debug");

        // Cleanup
        env::remove_var("OAUTH_CLIENT_ID");
        env::remove_var("OAUTH_CLIENT_SECRET");
        env::remove_var("OAUTH_AUTH_URL");
        env::remove_var("OAUTH_TOKEN_URL");
        env::remove_var("OAUTH_REDIRECT_URI");
        env::remove_var("RUSTBUS_ENDPOINT");
        env::remove_var("LOG_LEVEL");
    }
}

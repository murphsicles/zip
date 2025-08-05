use std::collections::HashMap;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::config::Config;
use crate::errors::ZipError;
use crate::storage::ZipStorage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_start_flow() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        let (url, csrf) = oauth.start_oauth_flow();
        assert!(!url.is_empty());
        assert!(!csrf.is_empty());
    }

    #[tokio::test]
    async fn test_oauth_complete_flow() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        // Mock OAuth callback
        let code = "mock_code".to_string();
        let pkce_verifier = PkceCodeVerifier::new("mock_verifier".to_string());
        let csrf = "mock_csrf".to_string();
        let result = oauth.complete_oauth_flow(code, pkce_verifier, csrf).await;
        assert!(matches!(result, Err(ZipError::OAuth(_))));
    }

    #[tokio::test]
    async fn test_oauth_clear_session() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        let user_id = Uuid::new_v4();
        // Store some data
        oauth
            .storage
            .store_user_data(user_id, b"test_data")
            .unwrap();
        assert!(oauth.storage.get_user_data(user_id).unwrap().is_some());
        // Clear session
        oauth.clear_session(user_id).await.unwrap();
        assert!(oauth.storage.get_user_data(user_id).unwrap().is_none());
    }

    #[test]
    fn test_passkey_registration() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();
        let user_id = Uuid::new_v4();
        let (challenge, _) = passkey.start_registration(user_id, "test_user").unwrap();
        assert!(!challenge.public_key.challenge.is_empty());
    }

    #[tokio::test]
    async fn test_passkey_authentication() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();
        let user_id = Uuid::new_v4();
        let (challenge, state) = passkey.start_registration(user_id, "test_user").unwrap();
        let cred = CreationPublicKeyCredential::default();
        let reg = passkey.complete_registration(cred, state).unwrap();
        let (auth_challenge, auth_state) = passkey.start_authentication(user_id, Some("123456")).await.unwrap();
        let auth_cred = PublicKeyCredential::default();
        let result = passkey.complete_authentication(auth_cred, auth_state);
        assert!(matches!(result, Err(ZipError::Passkey(_))));
    }

    #[tokio::test]
    async fn test_passkey_2fa_authentication() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();
        let user_id = Uuid::new_v4();

        // Enable 2FA
        let mut prefs = HashMap::new();
        prefs.insert("2fa_enabled".to_string(), "mock_secret".to_string());
        let serialized = bincode::serialize(&prefs).unwrap();
        storage.store_user_data(user_id, &serialized).unwrap();

        // Test with invalid 2FA code
        let result = passkey.start_authentication(user_id, Some("wrong_code")).await;
        assert!(matches!(result, Err(ZipError::Passkey(_))));

        // Test with missing 2FA code
        let result = passkey.start_authentication(user_id, None).await;
        assert!(matches!(result, Err(ZipError::Passkey(_))));
    }

    #[tokio::test]
    async fn test_logout_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();
        let user_id = Uuid::new_v4();

        // Simulate authenticated state
        oauth
            .storage
            .store_user_data(user_id, b"test_user")
            .unwrap();

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(oauth).with_context(passkey)
        });
        let html = app.render_to_string();
        assert!(html.contains("Logout"));
        assert!(html.contains("Confirm Logout"));
    }
}

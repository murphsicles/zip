use tokio::runtime::Runtime;
use uuid::Uuid;
use zip::auth::{OAuthManager, PasskeyManager};
use zip::config::Config;
use zip::errors::ZipError;
use zip::storage::ZipStorage;

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
        // Mock OAuth callback (real test would use wiremock)
        let code = "mock_code".to_string();
        let pkce_verifier = PkceCodeVerifier::new("mock_verifier".to_string());
        let csrf = "mock_csrf".to_string();
        let result = oauth.complete_oauth_flow(code, pkce_verifier, csrf).await;
        assert!(matches!(result, Err(ZipError::OAuth(_))));
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
        let (auth_challenge, auth_state) = passkey.start_authentication(&[reg]).unwrap();
        let auth_cred = PublicKeyCredential::default();
        let result = passkey.complete_authentication(auth_cred, auth_state).await;
        assert!(matches!(result, Err(ZipError::Passkey(_))));
    }
}

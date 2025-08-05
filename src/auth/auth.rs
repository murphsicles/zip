use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager, SessionManager};
use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::storage::ZipStorage;
use crate::utils::metrics::Metrics;

pub struct AuthManager {
    oauth: OAuthManager,
    passkey: PasskeyManager,
    session: SessionManager,
    metrics: Metrics,
}

impl AuthManager {
    /// Initializes unified auth manager with metrics.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        Ok(Self {
            oauth: OAuthManager::new(Arc::clone(&storage))?,
            passkey: PasskeyManager::new(Arc::clone(&storage))?,
            session: SessionManager::new(Arc::clone(&storage)),
            metrics: Metrics::new(&config),
        })
    }

    /// Starts OAuth flow, tracks event, returns auth URL and CSRF token.
    pub fn start_oauth(&self) -> (String, String) {
        let (url, csrf) = self.oauth.start_oauth_flow();
        self.metrics.track_auth_event("anonymous", "oauth_start", true);
        (url, csrf)
    }

    /// Completes OAuth flow, creates session, and tracks event.
    pub async fn complete_oauth(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
        csrf_token: String,
    ) -> Result<Uuid, ZipError> {
        let (user_id, email) = self.oauth.complete_oauth_flow(code, pkce_verifier, csrf_token).await?;
        self.session.create_session(user_id, email).await?;
        self.metrics.track_auth_event(&user_id.to_string(), "oauth_complete", true);
        Ok(user_id)
    }

    /// Starts Passkey authentication with 2FA check, tracks event.
    pub async fn start_passkey_authentication(
        &self,
        user_id: Uuid,
        totp_code: Option<&str>,
    ) -> Result<(RequestChallengeResponse, PasskeyAuthenticationState), ZipError> {
        let result = self.passkey.start_authentication(user_id, totp_code).await;
        self.metrics.track_auth_event(
            &user_id.to_string(),
            "passkey_start",
            result.is_ok(),
        );
        result
    }

    /// Completes Passkey authentication, creates session, and tracks event.
    pub async fn complete_passkey_authentication(
        &self,
        user_id: Uuid,
        cred: PublicKeyCredential,
        state: PasskeyAuthenticationState,
    ) -> Result<(), ZipError> {
        let result = self.passkey.complete_authentication(cred, state);
        let email = self
            .session
            .get_session(user_id)
            .await?
            .map(|s| s.email)
            .unwrap_or("passkey_user@example.com".to_string());
        self.session.create_session(user_id, email).await?;
        self.metrics.track_auth_event(
            &user_id.to_string(),
            "passkey_complete",
            result.is_ok(),
        );
        result
    }

    /// Checks if user is authenticated.
    pub async fn is_authenticated(&self, user_id: Uuid) -> bool {
        self.session.is_authenticated(user_id).await
    }

    /// Clears session for logout and tracks event.
    pub async fn logout(&self, user_id: Uuid) -> Result<(), ZipError> {
        self.session.clear_session(user_id).await?;
        self.oauth.clear_session(user_id).await?;
        self.metrics.track_auth_event(&user_id.to_string(), "logout", true);
        Ok(())
    }
}

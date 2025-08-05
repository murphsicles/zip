use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager, SessionManager};
use crate::errors::ZipError;
use crate::storage::ZipStorage;

pub struct AuthManager {
    oauth: OAuthManager,
    passkey: PasskeyManager,
    session: SessionManager,
}

impl AuthManager {
    /// Initializes unified auth manager.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        Ok(Self {
            oauth: OAuthManager::new(Arc::clone(&storage))?,
            passkey: PasskeyManager::new(Arc::clone(&storage))?,
            session: SessionManager::new(Arc::clone(&storage)),
        })
    }

    /// Starts OAuth flow, returns auth URL and CSRF token.
    pub fn start_oauth(&self) -> (String, String) {
        self.oauth.start_oauth_flow()
    }

    /// Completes OAuth flow and creates session.
    pub async fn complete_oauth(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
        csrf_token: String,
    ) -> Result<Uuid, ZipError> {
        let (user_id, email) = self.oauth.complete_oauth_flow(code, pkce_verifier, csrf_token).await?;
        self.session.create_session(user_id, email).await?;
        Ok(user_id)
    }

    /// Starts Passkey authentication with 2FA check.
    pub async fn start_passkey_authentication(
        &self,
        user_id: Uuid,
        totp_code: Option<&str>,
    ) -> Result<(RequestChallengeResponse, PasskeyAuthenticationState), ZipError> {
        self.passkey.start_authentication(user_id, totp_code).await
    }

    /// Completes Passkey authentication and creates session.
    pub async fn complete_passkey_authentication(
        &self,
        user_id: Uuid,
        cred: PublicKeyCredential,
        state: PasskeyAuthenticationState,
    ) -> Result<(), ZipError> {
        let result = self.passkey.complete_authentication(cred, state)?;
        let email = "passkey_user@example.com"; // Placeholder, fetch from storage or external
        self.session.create_session(user_id, email.to_string()).await?;
        Ok(())
    }

    /// Checks if user is authenticated.
    pub async fn is_authenticated(&self, user_id: Uuid) -> bool {
        self.session.is_authenticated(user_id).await
    }

    /// Clears session for logout.
    pub async fn logout(&self, user_id: Uuid) -> Result<(), ZipError> {
        self.session.clear_session(user_id).await?;
        self.oauth.clear_session(user_id).await?;
        Ok(())
    }
}

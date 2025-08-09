use oauth2::PkceCodeVerifier;
use std::sync::Arc;
use uuid::Uuid;
use webauthn_rs::prelude::{PasskeyAuthentication, PublicKeyCredential, RequestChallengeResponse};

use crate::auth::{OAuthManager, PasskeyManager};
use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::storage::ZipStorage;
use crate::utils::rate_limiter::RateLimiter;
use crate::utils::security::Security;
use crate::utils::session::Session;
use crate::utils::telemetry::Telemetry;

#[derive(Clone)]
pub struct AuthManager {
    oauth: OAuthManager,
    passkey: PasskeyManager,
    session: Session,
    telemetry: Telemetry,
    rate_limiter: RateLimiter,
}

impl AuthManager {
    /// Initializes unified auth manager with session, telemetry, and rate limiter.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        Ok(Self {
            oauth: OAuthManager::new(Arc::clone(&storage))?,
            passkey: PasskeyManager::new(Arc::clone(&storage))?,
            session: Session::new(Arc::clone(&storage))?,
            telemetry: Telemetry::new(&config),
            rate_limiter: RateLimiter::new(5, 60), // 5 requests per minute
        })
    }

    /// Starts OAuth flow, checks rate limit, tracks event, returns auth URL and CSRF token.
    pub async fn start_oauth(&self, user_id: &str) -> Result<(String, String), ZipError> {
        self.rate_limiter.check(user_id).await?;
        let (url, csrf) = self.oauth.start_oauth_flow();
        let _ = self
            .telemetry
            .track_auth_event(user_id, "oauth_start", true)
            .await;
        Ok((url, csrf))
    }

    /// Completes OAuth flow, checks rate limit, creates session, and tracks event.
    pub async fn complete_oauth(
        &self,
        user_id: &str,
        code: String,
        pkce_verifier: PkceCodeVerifier,
        csrf_token: String,
    ) -> Result<Uuid, ZipError> {
        self.rate_limiter.check(user_id).await?;
        let result = self
            .oauth
            .complete_oauth_flow(code, pkce_verifier, csrf_token)
            .await;
        let success = result.is_ok();
        if let Ok((user_id, email)) = &result {
            let sanitized_email = Security::sanitize_input(email)?;
            Security::validate_email(&sanitized_email)?;
            self.session.create(*user_id, sanitized_email).await?;
            let _ = self
                .telemetry
                .track_auth_event(&user_id.to_string(), "oauth_complete", success)
                .await;
        }
        result.map(|(user_id, _)| user_id)
    }

    /// Starts Passkey authentication with 2FA check, checks rate limit, tracks event.
    pub async fn start_passkey_authentication(
        &self,
        user_id: Uuid,
        totp_code: Option<&str>,
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication), ZipError> {
        self.rate_limiter.check(&user_id.to_string()).await?;
        let result = self.passkey.start_authentication(user_id, totp_code).await;
        let _ = self
            .telemetry
            .track_auth_event(&user_id.to_string(), "passkey_start", result.is_ok())
            .await;
        result
    }

    /// Completes Passkey authentication, checks rate limit, creates session, and tracks event.
    pub async fn complete_passkey_authentication(
        &self,
        user_id: Uuid,
        cred: PublicKeyCredential,
        state: PasskeyAuthentication,
    ) -> Result<(), ZipError> {
        self.rate_limiter.check(&user_id.to_string()).await?;
        let result = self.passkey.complete_authentication(cred, state);
        let success = result.is_ok();
        if success {
            let email = self
                .session
                .get(user_id)
                .await?
                .map(|s| s.email)
                .unwrap_or("passkey_user@example.com".to_string());
            let sanitized_email = Security::sanitize_input(&email)?;
            Security::validate_email(&sanitized_email)?;
            self.session.create(user_id, sanitized_email).await?;
            let _ = self
                .telemetry
                .track_auth_event(&user_id.to_string(), "passkey_complete", success)
                .await;
        }
        result
    }

    /// Checks if user is authenticated.
    pub async fn is_authenticated(&self, user_id: Uuid) -> bool {
        self.session.is_authenticated(user_id).await
    }

    /// Clears session for logout, checks rate limit, and tracks event.
    pub async fn logout(&self, user_id: Uuid) -> Result<(), ZipError> {
        self.rate_limiter.check(&user_id.to_string()).await?;
        let result = self.session.clear(user_id).await;
        let success = result.is_ok();
        let _ = self
            .telemetry
            .track_auth_event(&user_id.to_string(), "logout", success)
            .await;
        result
    }
}

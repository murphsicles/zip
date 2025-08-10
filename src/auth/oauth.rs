use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope};
use openidconnect::IdTokenClaims;
use openidconnect::core::{CoreAdditionalClaims, CoreGenderClaim};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::storage::ZipStorage;

#[derive(Clone)]
pub struct OAuthManager {
    client: Arc<BasicClient>,
    storage: Arc<ZipStorage>,
}

impl OAuthManager {
    /// Initializes OAuth client for Google using environment config.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        let client = BasicClient::new(
            config.oauth_client_id.into(),
            Some(config.oauth_client_secret.into()),
            config
                .oauth_auth_url
                .parse()
                .map_err(|_| ZipError::OAuth("Invalid auth URL".to_string().into()))?,
            Some(
                config
                    .oauth_token_url
                    .parse()
                    .map_err(|_| ZipError::OAuth("Invalid token URL".to_string().into()))?,
            ),
        )
        .set_redirect_uri(
            config
                .oauth_redirect_uri
                .parse()
                .map_err(|_| ZipError::OAuth("Invalid redirect URI".to_string().into()))?,
        );
        Ok(Self {
            client: Arc::new(client),
            storage,
        })
    }

    /// Starts OAuth flow, returns auth URL and CSRF token.
    pub fn start_oauth_flow(&self) -> (String, String) {
        let (pkce_challenge, _) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid email".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        (auth_url.to_string(), csrf_token.secret().clone())
    }

    /// Completes OAuth flow, stores user data, returns user ID and email.
    pub async fn complete_oauth_flow(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
        csrf_token: String,
    ) -> Result<(Uuid, String), ZipError> {
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(http_client)
            .await
            .map_err(|e| ZipError::OAuth(Box::new(e)))?;

        let claims: IdTokenClaims<CoreAdditionalClaims, CoreGenderClaim> = token
            .id_token()
            .ok_or_else(|| ZipError::OAuth("Missing ID token".to_string().into()))?
            .claims(&self.client.id_token_verifier(), None)
            .map_err(|e| ZipError::OAuth(Box::new(e)))?;
        let email = claims
            .email()
            .and_then(|e| e.get(None))
            .map(|e| e.to_string())
            .unwrap_or("unknown".to_string());

        let user_id = Uuid::new_v4();
        self.storage.store_user_data(user_id, email.as_bytes())?;

        Ok((user_id, email))
    }

    /// Clears session data for logout.
    pub async fn clear_session(&self, user_id: Uuid) -> Result<(), ZipError> {
        self.storage.store_user_data(user_id, &[])?;
        Ok(())
    }
}

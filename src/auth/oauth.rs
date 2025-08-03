use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use openidconnect::IdTokenClaims;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::errors::ZipError;
use crate::storage::ZipStorage;

const CLIENT_ID: &str = "google-client-id"; // Env var in prod
const CLIENT_SECRET: &str = "google-client-secret";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URI: &str = "zip://oauth/callback"; // Custom scheme for native

pub struct OAuthManager {
    client: BasicClient,
    storage: Arc<ZipStorage>,
}

impl OAuthManager {
    /// Initializes OAuth client for Google.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let client = BasicClient::new(
            CLIENT_ID.into(),
            Some(CLIENT_SECRET.into()),
            AUTH_URL.parse()?,
            Some(TOKEN_URL.parse()?),
        ).set_redirect_uri(REDIRECT_URI.parse()?);
        Ok(Self { client, storage })
    }

    /// Starts OAuth flow, returns auth URL and CSRF token.
    pub fn start_oauth_flow(&self) -> (String, String) {
        let (pkce_challenge, _) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = self.client
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
        let token = self.client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| ZipError::OAuth(Box::new(e)))?;

        let claims: IdTokenClaims = token.id_token().unwrap().claims(&self.client.id_token_verifier(), None)?;
        let email = claims.email().unwrap().get(None).unwrap_or("unknown");

        let user_id = Uuid::new_v4();
        self.storage.store_user_data(user_id, email.as_bytes())?;

        Ok((user_id, email.to_string()))
    }
}

use oauth2::basic::BasicClient;
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
};
use openidconnect::IdTokenClaims;
use openidconnect::core::{CoreAdditionalClaims, CoreGenderClaim};
use std::sync::Arc;

use crate::errors::ZipError;
use crate::storage::ZipStorage;

pub struct OAuth {
    client: Arc<BasicClient>,
    storage: Arc<ZipStorage>,
}

impl OAuth {
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        let client = BasicClient::new(
            config
                .oauth_client_id
                .parse()
                .map_err(|_| ZipError::OAuth("Invalid client ID".to_string().into()))?,
        )
        .set_client_secret(Some(config.oauth_client_secret.parse().map_err(|_| {
            ZipError::OAuth("Invalid client secret".to_string().into())
        })?))
        .set_auth_url(
            Url::parse(&config.oauth_auth_url)
                .map_err(|_| ZipError::OAuth("Invalid auth URL".to_string().into()))?,
        )
        .set_token_url(Some(Url::parse(&config.oauth_token_url).map_err(|_| {
            ZipError::OAuth("Invalid token URL".to_string().into())
        })?))
        .set_redirect_uri(RedirectUrl::from_url(
            Url::parse(&config.oauth_redirect_uri)
                .map_err(|_| ZipError::OAuth("Invalid redirect URI".to_string().into()))?,
        ));
        Ok(Self {
            client: Arc::new(client),
            storage,
        })
    }
}

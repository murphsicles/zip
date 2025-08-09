use bincode;
use std::collections::HashMap;
use std::sync::Arc;
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::errors::ZipError;
use crate::storage::ZipStorage;
use crate::utils::misc::generate_salt;

#[derive(Clone)]
pub struct PasskeyManager {
    webauthn: Arc<Webauthn>,
    storage: Arc<ZipStorage>,
}

impl PasskeyManager {
    /// Initializes Passkey manager with relying party details.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let webauthn = WebauthnBuilder::new("zip-app", &Url::parse("https://zip-app.com")?)?
            .build()?;
        Ok(Self {
            webauthn: Arc::new(webauthn),
            storage,
        })
    }

    /// Starts Passkey registration for a user.
    pub fn start_registration(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration), ZipError> {
        self.webauthn
            .start_passkey_registration(user_id, username, username, None)
            .map_err(|e| ZipError::Passkey(e))
    }

    /// Completes Passkey registration and stores credential.
    pub fn complete_registration(
        &self,
        user_id: Uuid,
        cred: PublicKeyCredential,
        state: PasskeyRegistration,
    ) -> Result<Passkey, ZipError> {
        let passkey = self
            .webauthn
            .finish_passkey_registration(&cred, &state)
            .map_err(|e| ZipError::Passkey(e))?;
        let serialized = bincode::serialize(&passkey)
            .map_err(|e| ZipError::Passkey(WebauthnError::Unknown))?;
        self.storage.store_user_data(user_id, &serialized)?;
        Ok(passkey)
    }

    /// Starts Passkey authentication with stored credentials and checks 2FA if enabled.
    pub async fn start_authentication(
        &self,
        user_id: Uuid,
        totp_code: Option<&str>,
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication), ZipError> {
        // Check 2FA if enabled
        if let Some(data) = self.storage.get_user_data(user_id)? {
            let prefs: HashMap<String, String> = bincode::deserialize(&data).unwrap_or_default();
            if let Some(secret) = prefs.get("2fa_enabled") {
                let secret = Secret::Encoded(secret.to_string());
                let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.to_bytes().unwrap())?;
                if let Some(code) = totp_code {
                    if !totp.check_current(code)? {
                        return Err(ZipError::Passkey(WebauthnError::Unknown));
                    }
                } else {
                    return Err(ZipError::Passkey(WebauthnError::Unknown));
                }
            }
        }

        let cached = self
            .storage
            .get_user_data(user_id)?
            .ok_or(ZipError::Passkey(WebauthnError::CredentialRetrievalError))?;
        let passkeys: Vec<Passkey> = bincode::deserialize(&cached)
            .map_err(|_| ZipError::Passkey(WebauthnError::Unknown))?;
        self.webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| ZipError::Passkey(e))
    }

    /// Completes Passkey authentication.
    pub fn complete_authentication(
        &self,
        cred: PublicKeyCredential,
        state: PasskeyAuthentication,
    ) -> Result<AuthenticationResult, ZipError> {
        Ok(self
            .webauthn
            .finish_passkey_authentication(&cred, &state)
            .map_err(|e| ZipError::Passkey(e))?)
    }

    /// Generates 2FA setup data if not already enabled.
    pub fn setup_2fa(&self, user_id: Uuid, username: &str) -> Result<(String, String), ZipError> {
        let secret = Secret::Raw(generate_salt(20));
        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.to_bytes().unwrap())?;
        let qr_code = totp.get_url();
        let secret_base32 = totp.get_secret_base32();
        Ok((qr_code, secret_base32))
    }
}

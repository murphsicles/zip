use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::errors::ZipError;
use crate::storage::ZipStorage;
use crate::utils::generate_salt;

pub struct PasskeyManager {
    webauthn: Webauthn,
    storage: Arc<ZipStorage>,
}

impl PasskeyManager {
    /// Initializes Passkey manager with relying party details.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let webauthn = WebauthnBuilder::new("zip-app", "https://zip-app.com")?.build()?;
        Ok(Self { webauthn, storage })
    }

    /// Starts Passkey registration for a user.
    pub fn start_registration(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<(CreationChallengeResponse, PasskeyRegistrationState), ZipError> {
        self.webauthn
            .start_passkey_registration(user_id, username, None, None)
    }

    /// Completes Passkey registration and stores credential.
    pub fn complete_registration(
        &self,
        cred: CreationPublicKeyCredential,
        state: PasskeyRegistrationState,
    ) -> Result<Passkey, ZipError> {
        let passkey = self.webauthn.finish_passkey_registration(&cred, &state)?;
        let serialized = bincode::serialize(&passkey)
            .map_err(|e| ZipError::Passkey(WebauthnError::Other(e.to_string())))?;
        self.storage.store_user_data(passkey.user_id, &serialized)?;
        Ok(passkey)
    }

    /// Starts Passkey authentication with stored credentials and checks 2FA if enabled.
    pub async fn start_authentication(
        &self,
        user_id: Uuid,
        totp_code: Option<&str>,
    ) -> Result<(RequestChallengeResponse, PasskeyAuthenticationState), ZipError> {
        // Check 2FA if enabled
        if let Some(data) = self.storage.get_user_data(user_id)? {
            let prefs: HashMap<String, String> = bincode::deserialize(&data).unwrap_or_default();
            if let Some(secret) = prefs.get("2fa_enabled") {
                let totp = TOTP::new_from_secret(secret)?;
                if let Some(code) = totp_code {
                    if !totp.check_current(code)? {
                        return Err(ZipError::Passkey(WebauthnError::Other(
                            "Invalid 2FA code".to_string(),
                        )));
                    }
                } else {
                    return Err(ZipError::Passkey(WebauthnError::Other(
                        "2FA code required".to_string(),
                    )));
                }
            }
        }

        let cached = self
            .storage
            .get_user_data(user_id)?
            .ok_or(ZipError::Passkey(WebauthnError::CredentialRetrievalError))?;
        let passkeys: Vec<Passkey> = bincode::deserialize(&cached)
            .map_err(|e| ZipError::Passkey(WebauthnError::Other(e.to_string())))?;
        self.webauthn.start_passkey_authentication(&passkeys)
    }

    /// Completes Passkey authentication.
    pub fn complete_authentication(
        &self,
        cred: PublicKeyCredential,
        state: PasskeyAuthenticationState,
    ) -> Result<AuthenticationResult, ZipError> {
        self.webauthn.finish_passkey_authentication(&cred, &state)
    }

    /// Generates 2FA setup data if not already enabled.
    pub fn setup_2fa(&self, user_id: Uuid, username: &str) -> Result<(String, String), ZipError> {
        let secret = Secret::Raw(generate_salt(20));
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().unwrap(),
            Some("Zip Wallet".to_string()),
            username.to_string(),
        )?;
        let qr_code = totp.get_qr()?;
        let secret_base32 = totp.secret_base32()?;
        Ok((qr_code, secret_base32))
    }
}

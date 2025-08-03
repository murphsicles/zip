use webauthn_rs::prelude::*;
use uuid::Uuid;

use crate::errors::ZipError;
use crate::storage::ZipStorage;

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
    pub fn start_registration(&self, user_id: Uuid, username: &str) -> Result<(CreationChallengeResponse, RegisterPasskeyState), ZipError> {
        self.webauthn.start_passkey_registration(user_id, username, None, None)
    }

    /// Completes Passkey registration.
    pub fn complete_registration(
        &self,
        cred: CreationPublicKeyCredential,
        state: RegisterPasskeyState,
    ) -> Result<Passkey, ZipError> {
        let passkey = self.webauthn.finish_passkey_registration(&cred, &state)?;
        // Store serialized passkey in storage
        let serialized = bincode::serialize(&passkey).map_err(|e| ZipError::Passkey(WebauthnError::from(e)))?;
        let user_id = passkey.user_id();
        self.storage.store_user_data(user_id, &serialized)?;
        Ok(passkey)
    }

    /// Starts Passkey authentication.
    pub fn start_authentication(&self, passkeys: &[Passkey]) -> Result<(AuthenticationChallengeResponse, AuthPasskeyState), ZipError> {
        self.webauthn.start_passkey_authentication(passkeys)
    }

    /// Completes Passkey authentication.
    pub fn complete_authentication(
        &self,
        cred: PublicKeyCredential,
        state: AuthPasskeyState,
    ) -> Result<AuthenticationResult, ZipError> {
        self.webauthn.finish_passkey_authentication(&cred, &state)
    }
}

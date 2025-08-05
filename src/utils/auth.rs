use std::sync::Arc;
use uuid::Uuid;

use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::storage::ZipStorage;
use crate::utils::session::Session;
use crate::utils::telemetry::Telemetry;

pub struct AuthUtils {
    session: Session,
    telemetry: Telemetry,
}

impl AuthUtils {
    /// Initializes auth utilities with session and telemetry.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        Ok(Self {
            session: Session::new(Arc::clone(&storage))?,
            telemetry: Telemetry::new(&config),
        })
    }

    /// Validates a TOTP code for 2FA.
    pub async fn validate_totp(&self, user_id: Uuid, code: &str) -> Result<bool, ZipError> {
        let session_data = self.session.get(user_id).await?;
        if let Some(data) = session_data {
            if let Some(secret) = data.email.as_bytes().get(0..20) {
                let totp = totp_rs::TOTP::new(
                    totp_rs::Algorithm::SHA1,
                    6,
                    1,
                    30,
                    secret.to_vec(),
                    Some("Zip Wallet".to_string()),
                    data.email.clone(),
                )
                .map_err(|e| ZipError::Auth(e.to_string()))?;
                let result = totp.check_current(code).map_err(|e| ZipError::Auth(e.to_string()))?;
                let _ = self.telemetry.track_auth_event(&user_id.to_string(), "totp_validation", result).await;
                Ok(result)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Generates a new TOTP secret and QR code for 2FA setup.
    pub async fn generate_totp(&self, user_id: Uuid, email: &str) -> Result<(String, String), ZipError> {
        let secret = Secret::Raw(generate_salt(20));
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().unwrap(),
            Some("Zip Wallet".to_string()),
            email.to_string(),
        )
        .map_err(|e| ZipError::Auth(e.to_string()))?;
        let qr_code = totp.get_qr().map_err(|e| ZipError::Auth(e.to_string()))?;
        let secret_base32 = totp.secret_base32().unwrap();
        let _ = self.telemetry.track_auth_event(&user_id.to_string(), "totp_generate", true).await;
        Ok((secret_base32, qr_code))
    }
}

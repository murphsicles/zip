use std::error::Error as StdError;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ZipError {
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Blockchain operation failed: {0}")]
    Blockchain(String),
    #[error("Keyring access failed: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("OAuth operation failed: {0}")]
    OAuth(Box<dyn StdError + Send + Sync>),
    #[error("Passkey operation failed: {0}")]
    Passkey(#[from] webauthn_rs::error::WebauthnError),
    #[error("Storage operation failed: {0}")]
    Storage(#[from] sled::Error),
}

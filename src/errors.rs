use std::error::Error as StdError;
use thiserror::Error;
use webauthn_rs::error::WebauthnError;

#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum ZipError {
    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Keyring access failed: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("OAuth operation failed: {0}")]
    OAuth(Box<dyn StdError + Send + Sync>),

    #[error("Passkey operation failed: {0}")]
    Passkey(#[from] WebauthnError),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] sled::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

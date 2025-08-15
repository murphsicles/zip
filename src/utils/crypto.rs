use rand::RngCore;
use rand::rngs::OsRng;
use sv::private_key::PrivateKey;
use sv::public_key::PublicKey;
use sv::util::hash160;
use crate::errors::ZipError;

pub struct Crypto;

impl Crypto {
    /// Generates a cryptographically secure private key.
    pub fn generate_private_key() -> Result<PrivateKey, ZipError> {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes); // Ensure rand 0.9.3 or later for OsRng
        PrivateKey::from_bytes(bytes).map_err(|e| {
            // TODO: Update src/errors.rs to include ZipError::Crypto variant
            ZipError::Crypto(e.to_string())
        })
    }

    /// Derives a public key from a private key.
    pub fn derive_public_key(private_key: &PrivateKey) -> PublicKey {
        private_key.public_key()
    }

    /// Generates a BSV address from a public key.
    pub fn generate_address(public_key: &PublicKey) -> String {
        let pubkey_hash = hash160(public_key.to_bytes());
        sv::address::Address::p2pkh(&pubkey_hash, sv::network::Network::Mainnet)
            .to_string()
            .unwrap_or_default()
    }

    /// Signs a message with a private key.
    pub fn sign_message(private_key: &PrivateKey, message: &[u8]) -> Result<Vec<u8>, ZipError> {
        let signature = private_key
            .sign(message)
            .map_err(|e| {
                // TODO: Update src/errors.rs to include ZipError::Crypto variant
                ZipError::Crypto(e.to_string())
            })?;
        Ok(signature)
    }

    /// Verifies a signature with a public key.
    pub fn verify_signature(
        public_key: &PublicKey,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, ZipError> {
        let result = public_key
            .verify(message, signature)
            .map_err(|e| {
                // TODO: Update src/errors.rs to include ZipError::Crypto variant
                ZipError::Crypto(e.to_string())
            })?;
        Ok(result)
    }
}

use rand::RngCore;
use rand::rngs::OsRng;
use secp256k1::{Secp256k1, SecretKey}; // Replaced sv::keypair::PrivateKey
use sv::public_key::PublicKey;
use sv::util::hash160;
use crate::errors::ZipError;

pub struct Crypto;

impl Crypto {
    /// Generates a cryptographically secure private key.
    pub fn generate_private_key() -> Result<SecretKey, ZipError> {
        let secp = Secp256k1::new();
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes); // Ensure rand 0.9.3 or later for OsRng
        SecretKey::from_slice(&bytes)
            .map_err(|e| ZipError::Crypto(e.to_string()))
    }

    /// Derives a public key from a private key.
    pub fn derive_public_key(private_key: &SecretKey) -> PublicKey {
        let secp = Secp256k1::new();
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, private_key);
        PublicKey::from_slice(&public_key.serialize()).unwrap() // Convert to sv::public_key
    }

    /// Generates a BSV address from a public key.
    pub fn generate_address(public_key: &PublicKey) -> String {
        let pubkey_hash = hash160(public_key.to_bytes());
        sv::address::Address::p2pkh(&pubkey_hash, sv::network::Network::Mainnet)
            .to_string()
            .unwrap_or_default()
    }

    /// Signs a message with a private key.
    pub fn sign_message(private_key: &SecretKey, message: &[u8]) -> Result<Vec<u8>, ZipError> {
        let secp = Secp256k1::new();
        let message_hash = sv::util::sha256d(message).0; // Assuming sha256d returns Hash256
        let sig = secp.sign_ecdsa(&secp256k1::Message::from_slice(&message_hash).unwrap(), private_key);
        Ok(sig.serialize_compact().to_vec())
    }

    /// Verifies a signature with a public key.
    pub fn verify_signature(
        public_key: &PublicKey,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, ZipError> {
        let secp = Secp256k1::new();
        let message_hash = sv::util::sha256d(message).0;
        let sig = secp256k1::Signature::from_compact(signature)
            .map_err(|e| ZipError::Crypto(e.to_string()))?;
        let pubkey = secp256k1::PublicKey::from_slice(public_key.to_bytes())
            .map_err(|e| ZipError::Crypto(e.to_string()))?;
        Ok(secp.verify_ecdsa(&secp256k1::Message::from_slice(&message_hash).unwrap(), &sig, &pubkey).is_ok())
    }
}

use crate::errors::ZipError;
use crate::utils::error::format_zip_error;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use webauthn_rs::error::WebauthnError;

    #[test]
    fn test_zip_error_auth() {
        let error = ZipError::Auth("Invalid credentials".to_string());
        assert_eq!(format_zip_error(&error), "Auth error: Invalid credentials");
    }

    #[test]
    fn test_zip_error_blockchain() {
        let error = ZipError::Blockchain("Transaction failed".to_string());
        assert_eq!(format_zip_error(&error), "Blockchain error: Transaction failed");
    }

    #[test]
    fn test_zip_error_keyring() {
        let error = ZipError::Keyring(keyring::Error::NoEntry);
        assert_eq!(format_zip_error(&error), "Keyring error: No entry found");
    }

    #[test]
    fn test_zip_error_network() {
        let error = ZipError::Network(reqwest::Error::from(std::io::Error::new(std::io::ErrorKind::Other, "Network issue")));
        assert!(format_zip_error(&error).starts_with("Network error:"));
    }

    #[test]
    fn test_zip_error_oauth() {
        let inner_error = std::io::Error::new(std::io::ErrorKind::Other, "OAuth failure");
        let error = ZipError::OAuth(Box::new(inner_error));
        assert!(format_zip_error(&error).starts_with("OAuth error:"));
    }

    #[test]
    fn test_zip_error_passkey() {
        let error = ZipError::Passkey(WebauthnError::CredentialRetrievalError);
        assert_eq!(format_zip_error(&error), "Passkey error: CredentialRetrievalError");
    }

    #[test]
    fn test_zip_error_storage() {
        let error = ZipError::Storage(sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Storage issue")));
        assert!(format_zip_error(&error).starts_with("Storage error:"));
    }
}

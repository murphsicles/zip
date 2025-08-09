use rand::rngs::OsRng;
use rand::RngCore;
use regex::Regex;

use crate::errors::ZipError;

/// Security utilities for the Zip wallet.
pub struct Security;

impl Security {
    /// Generates a cryptographically secure random salt.
    pub fn generate_salt(len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    /// Sanitizes user input to prevent injection attacks (e.g., XSS, SQL-like).
    pub fn sanitize_input(input: &str) -> Result<String, ZipError> {
        // Remove potentially dangerous characters (<, >, &, ", ', /)
        let sanitized = input
            .chars()
            .filter(|c| !['<', '>', '&', '"', '\'', '/'].contains(c))
            .collect::<String>();
        if sanitized.is_empty() {
            Err(ZipError::Validation("Input is empty after sanitization".to_string()))
        } else {
            Ok(sanitized)
        }
    }

    /// Validates email format (basic regex check).
    pub fn validate_email(email: &str) -> Result<(), ZipError> {
        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|e| ZipError::Validation(format!("Invalid regex: {}", e)))?;
        if re.is_match(email) {
            Ok(())
        } else {
            Err(ZipError::Validation("Invalid email format".to_string()))
        }
    }
}

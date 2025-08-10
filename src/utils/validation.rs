use regex::Regex;

use crate::errors::ZipError;

/// Input validation utilities for the Zip wallet.
pub struct Validation;

impl Validation {
    /// Validates a PayMail alias prefix (minimum 5 digits).
    pub fn validate_paymail_prefix(prefix: &str) -> Result<(), ZipError> {
        let re = Regex::new(r"^\d{5,}$")
            .map_err(|e| ZipError::Validation(format!("Invalid regex: {}", e)))?;
        if re.is_match(prefix) {
            Ok(())
        } else {
            Err(ZipError::Validation(
                "PayMail prefix must be 5 or more digits".to_string(),
            ))
        }
    }

    /// Validates a TOTP code (6 digits).
    pub fn validate_totp_code(code: &str) -> Result<(), ZipError> {
        let re = Regex::new(r"^\d{6}$")
            .map_err(|e| ZipError::Validation(format!("Invalid regex: {}", e)))?;
        if re.is_match(code) {
            Ok(())
        } else {
            Err(ZipError::Validation(
                "TOTP code must be 6 digits".to_string(),
            ))
        }
    }

    /// Validates a currency code (3 letters, e.g., USD, EUR).
    pub fn validate_currency(currency: &str) -> Result<(), ZipError> {
        let re = Regex::new(r"^[A-Z]{3}$")
            .map_err(|e| ZipError::Validation(format!("Invalid regex: {}", e)))?;
        if re.is_match(currency) {
            Ok(())
        } else {
            Err(ZipError::Validation(
                "Currency must be a 3-letter code (e.g., USD)".to_string(),
            ))
        }
    }

    /// Validates a payment amount (positive non-zero integer).
    pub fn validate_amount(amount: u64) -> Result<(), ZipError> {
        if amount > 0 {
            Ok(())
        } else {
            Err(ZipError::Validation(
                "Amount must be greater than zero".to_string(),
            ))
        }
    }
}

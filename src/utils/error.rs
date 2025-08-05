use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZipError {
    Blockchain(String),
    Auth(String),
    OAuth(Box<dyn std::error::Error + Send + Sync>),
    Passkey(String),
    Config(String),
}

impl std::fmt::Display for ZipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZipError::Blockchain(msg) => write!(f, "Blockchain error: {}", msg),
            ZipError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            ZipError::OAuth(err) => write!(f, "OAuth error: {}", err),
            ZipError::Passkey(msg) => write!(f, "Passkey error: {}", msg),
            ZipError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ZipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ZipError::OAuth(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

/// Converts errors to ZipError for unified handling.
pub fn to_zip_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> ZipError {
    ZipError::OAuth(Box::new(error))
}

/// Formats error for user-friendly display.
pub fn format_error(error: &ZipError) -> String {
    match error {
        ZipError::Blockchain(msg) => format!("Blockchain issue: {}", msg),
        ZipError::Auth(msg) => format!("Authentication failed: {}", msg),
        ZipError::OAuth(err) => format!("OAuth issue: {}", err),
        ZipError::Passkey(msg) => format!("Passkey issue: {}", msg),
        ZipError::Config(msg) => format!("Configuration issue: {}", msg),
    }
}

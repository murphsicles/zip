use rand::RngCore;
use rand::rngs::OsRng;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

/// Generates a cryptographically secure random salt.
pub fn generate_salt(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Sets up logging with tracing-subscriber based on EnvConfig log level.
pub fn setup_logging(
    config: &crate::config::env::EnvConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_new(&config.log_level)?;
    tracing_subscriber::registry().with(filter).init();
    Ok(())
}

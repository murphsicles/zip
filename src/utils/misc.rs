use rand::RngCore;
use rand::rngs::OsRng;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::config::env::EnvConfig;

/// Generates a cryptographically secure random salt.
pub fn generate_salt(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes); // Changed from OsRng to thread_rng
    bytes
}

/// Sets up logging with tracing-subscriber based on EnvConfig log level.
pub fn setup_logging(config: &EnvConfig) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_new(&config.log_level)?;
    tracing_subscriber::registry().with(filter).init();
    Ok(())
}

use rand::rngs::thread_rng;
use rand::RngCore;
use tracing_subscriber::FmtSubscriber;

use crate::config::EnvConfig;
use crate::errors::ZipError;

pub fn setup_logging(config: &EnvConfig) -> Result<(), ZipError> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(config.log_level.clone())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| ZipError::Auth(e.to_string()))?;
    Ok(())
}

pub fn generate_salt(size: usize) -> Vec<u8> {
    let mut salt = vec![0u8; size];
    thread_rng().fill_bytes(&mut salt);
    salt
}

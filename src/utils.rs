use tracing::{Level, Subscriber};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::config::Config;
use crate::errors::ZipError;

pub fn setup_logging(config: &Config) -> Result<(), ZipError> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new(&config.log_level))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| ZipError::Auth(e.to_string()))?;
    Ok(())
}

pub fn generate_salt(size: usize) -> Vec<u8> {
    let mut salt = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

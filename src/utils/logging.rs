use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use crate::config::EnvConfig;
use crate::errors::ZipError;

pub fn setup_logging(config: &EnvConfig) -> Result<(), ZipError> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));
    fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|e| ZipError::Config(e.to_string()))?;
    Ok(())
}

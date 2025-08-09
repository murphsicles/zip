use dioxus::prelude::*;
use dioxus_desktop;
use tracing::info;

use crate::config::env::EnvConfig;
use crate::ui::components::App;
use crate::utils::misc::setup_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EnvConfig::load()?;
    setup_logging(&config)?;

    info!("Initializing Zip wallet");

    dioxus_desktop::launch(App);

    Ok(())
}

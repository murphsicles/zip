use dioxus::prelude::*;
use tracing::info;

use zip::config::env::EnvConfig;
use zip::ui::components::App;
use zip::utils::logging::setup_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EnvConfig::load()?;
    setup_logging(&config)?;

    info!("Initializing Zip wallet");

    dioxus_desktop::launch(App);

    Ok(())
}

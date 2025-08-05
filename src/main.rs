use dioxus::prelude::*;
use tracing::info;

use zip::auth::AuthManager;
use zip::blockchain::{PaymailManager, TransactionManager, WalletManager};
use zip::config::Config;
use zip::integrations::RustBusIntegrator;
use zip::storage::ZipStorage;
use zip::ui::components::App;
use zip::utils::setup_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    setup_logging(&config)?;

    info!("Initializing Zip wallet");

    dioxus_desktop::launch(App);

    Ok(())
}

use dioxus::prelude::*;
use tracing::info;
use zip::auth::OAuthManager;
use zip::blockchain::WalletManager;
use zip::config::Config;
use zip::storage::ZipStorage;
use zip::ui::router::AppRouter;
use zip::utils::setup_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    setup_logging(&config)?;

    info!("Initializing Zip wallet");

    let storage = Arc::new(ZipStorage::new()?);
    let oauth = OAuthManager::new(Arc::clone(&storage))?;
    let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage)));
    let wallet = WalletManager::new(Arc::clone(&storage), tx_manager)?;

    // Launch Dioxus app
    dioxus_desktop::launch(AppRouter, |c| c.with_context(wallet).with_context(oauth));

    Ok(())
}

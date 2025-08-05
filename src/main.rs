use dioxus::prelude::*;
use tracing::info;
use zip::auth::{OAuthManager, PasskeyManager};
use zip::blockchain::{PaymailManager, TransactionManager, WalletManager};
use zip::config::Config;
use zip::integrations::RustBusIntegrator;
use zip::storage::ZipStorage;
use zip::ui::router::AppRouter;
use zip::utils::setup_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    setup_logging(&config)?;

    info!("Initializing Zip wallet");

    let storage = Arc::new(ZipStorage::new()?);
    let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080")?);
    let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
    let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus)))?;
    let oauth = OAuthManager::new(Arc::clone(&storage))?;
    let passkey = PasskeyManager::new(Arc::clone(&storage))?;
    let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));

    dioxus_desktop::launch(
        AppRouter,
        |c| {
            c.with_context(wallet)
                .with_context(oauth)
                .with_context(passkey)
                .with_context(paymail)
                .with_context(tx_manager)
                .with_context(rustbus)
        },
    );

    Ok(())
}

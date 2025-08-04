use zip::blockchain::WalletManager;
use zip::config::Config;
use zip::storage::ZipStorage;
use zip::utils::setup_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    setup_logging(&config)?;

    let storage = Arc::new(ZipStorage::new()?);
    let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage)));
    let wallet = WalletManager::new(Arc::clone(&storage), tx_manager)?;

    println!("Wallet address: {}", wallet.get_address());

    Ok(())
}

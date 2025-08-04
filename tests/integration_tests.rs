use tokio::runtime::Runtime;
use uuid::Uuid;
use zip::auth::OAuthManager;
use zip::blockchain::WalletManager;
use zip::config::Config;
use zip::storage::ZipStorage;
use zip::utils::setup_logging;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_to_wallet_flow() {
        let config = Config::load().unwrap();
        setup_logging(&config).unwrap();

        let storage = Arc::new(ZipStorage::new().unwrap());
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        // Mock OAuth complete
        let (user_id, _) = oauth.complete_oauth_flow("mock_code".to_string(), PkceCodeVerifier::new("mock_verifier".to_string()), "mock_csrf".to_string()).await.unwrap();

        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage)));
        let wallet = WalletManager::new(Arc::clone(&storage), tx_manager).unwrap();
        let balance = wallet.update_balance(user_id).await.unwrap();
        assert_eq!(balance, 0); // Placeholder

        let address = wallet.get_address();
        assert!(!address.is_empty());
    }

    #[tokio::test]
    async fn test_payment_flow() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage)));
        let wallet = WalletManager::new(Arc::clone(&storage), tx_manager).unwrap();

        let recipient_script = Script::default(); // Mock
        let tx_hex = wallet.send_payment(recipient_script, 10000, 1000).await.unwrap();
        assert!(!tx_hex.is_empty());
    }
}

use tokio::runtime::Runtime;
use uuid::Uuid;
use zip::blockchain::{PaymailManager, TransactionManager, WalletManager};
use zip::storage::ZipStorage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_create_utxos() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let tx_manager = TransactionManager::new(Arc::clone(&storage));
        let result = tx_manager.pre_create_utxos("parent_txid", 0, 100000, 5, 10000).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_resolve_paymail() {
        let priv_key = PrivateKey::new();
        let paymail = PaymailManager::new(priv_key);
        // Mock PayMail resolution (real test with wiremock)
        let result = paymail.resolve_paymail("mock@paymail.com", 10000).await;
        assert!(matches!(result, Err(ZipError::Blockchain(_))));
    }

    #[tokio::test]
    async fn test_send_p2p_tx() {
        let priv_key = PrivateKey::new();
        let paymail = PaymailManager::new(priv_key);
        let metadata = Value::Null;
        let result = paymail.send_p2p_tx("mock@paymail.com", "mock_tx_hex", metadata, "ref").await.unwrap();
        assert_eq!(result, "fallback_txid");
    }

    #[test]
    fn test_wallet_address() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage)));
        let wallet = WalletManager::new(Arc::clone(&storage), tx_manager).unwrap();
        let address = wallet.get_address();
        assert!(!address.is_empty());
    }

    #[tokio::test]
    async fn test_update_balance() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage)));
        let wallet = WalletManager::new(Arc::clone(&storage), tx_manager).unwrap();
        let user_id = Uuid::new_v4();
        let balance = wallet.update_balance(user_id).await.unwrap();
        assert_eq!(balance, 0); // Placeholder assertion
    }
}

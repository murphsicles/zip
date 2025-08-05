use std::sync::Arc;
use uuid::Uuid;

use rust_decimal::Decimal;

use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_create_utxos() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = None::<Arc<RustBusIntegrator>>;
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), rustbus));
        let user_id = Uuid::new_v4();
        let result = tx_manager.pre_create_utxos(user_id, 5, 10000).block_on().unwrap();
        assert_eq!(result.len(), 5);
        let cached = storage.get_utxos(user_id).unwrap().unwrap();
        let utxos: Vec<TxOut> = bincode::deserialize(&cached).unwrap();
        assert_eq!(utxos.len(), 5);
    }

    #[tokio::test]
    async fn test_build_payment_tx() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = None::<Arc<RustBusIntegrator>>;
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), rustbus));
        let user_id = Uuid::new_v4();
        // Pre-create UTXOs
        tx_manager.pre_create_utxos(user_id, 5, 10000).await.unwrap();
        let script = Script::default();
        let result = tx_manager.build_payment_tx(user_id, script, 8000, 1000).await.unwrap();
        assert!(!result.to_hex().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_resolve_paymail() {
        let priv_key = PrivateKey::new();
        let storage = Arc::new(ZipStorage::new().unwrap());
        let paymail = PaymailManager::new(priv_key, Arc::clone(&storage));
        // Mock PayMail resolution
        let result = paymail.resolve_paymail("mock@paymail.com", 10000).await;
        assert!(matches!(result, Err(ZipError::Blockchain(_))));
    }

    #[tokio::test]
    async fn test_send_p2p_tx() {
        let priv_key = PrivateKey::new();
        let storage = Arc::new(ZipStorage::new().unwrap());
        let paymail = PaymailManager::new(priv_key, Arc::clone(&storage));
        let metadata = Value::Null;
        let result = paymail
            .send_p2p_tx("mock@paymail.com", "mock_tx_hex", metadata, "ref")
            .await
            .unwrap();
        assert_eq!(result, "fallback_txid");
    }

    #[test]
    fn test_wallet_address() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = None::<Arc<RustBusIntegrator>>;
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), rustbus));
        let wallet = WalletManager::new(Arc::clone(&storage), tx_manager, None).unwrap();
        let address = wallet.get_address().unwrap();
        assert!(!address.is_empty());
    }

    #[tokio::test]
    async fn test_update_balance() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = None::<Arc<RustBusIntegrator>>;
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), rustbus));
        let wallet = WalletManager::new(Arc::clone(&storage), tx_manager, None).unwrap();
        let user_id = Uuid::new_v4();
        let (balance, _) = wallet.update_balance(user_id, "USD").await.unwrap();
        assert_eq!(balance, 0);
    }

    #[tokio::test]
    async fn test_create_default_alias() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let priv_key = PrivateKey::new();
        let paymail = PaymailManager::new(priv_key, Arc::clone(&storage));
        let user_id = Uuid::new_v4();
        let (alias, price) = paymail.create_default_alias(user_id, None).await.unwrap();
        assert!(alias.starts_with("101@"));
        assert_eq!(price, Decimal::ZERO);

        let (bespoke, price) = paymail.create_default_alias(user_id, Some("12345")).await.unwrap();
        assert_eq!(bespoke, "12345@zip.io");
        assert_eq!(price, Decimal::from(10));
    }

    #[tokio::test]
    async fn test_create_paid_alias() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let priv_key = PrivateKey::new();
        let paymail = PaymailManager::new(priv_key, Arc::clone(&storage));
        let user_id = Uuid::new_v4();
        let (alias, price) = paymail.create_paid_alias(user_id, "54321").await.unwrap();
        assert_eq!(alias, "54321@zip.io");
        assert_eq!(price, Decimal::from(10));

        let (alias, price) = paymail.create_paid_alias(user_id, "john").await.unwrap();
        assert_eq!(alias, "john@zip.io");
        assert_eq!(price, Decimal::from(300));

        let result = paymail.create_paid_alias(user_id, "a").await;
        assert!(matches!(result, Err(ZipError::Blockchain(_))));
    }

    #[tokio::test]
    async fn test_confirm_alias() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let priv_key = PrivateKey::new();
        let paymail = PaymailManager::new(priv_key, Arc::clone(&storage));
        let user_id = Uuid::new_v4();
        paymail.create_paid_alias(user_id, "54321").await.unwrap();
        let result = paymail.confirm_alias(user_id, "54321@zip.io").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wallet_payment_rate_limit() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new().unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let user_id = Uuid::new_v4();
        let script = Script::default();
        let amount = 1000;
        let fee = 100;

        // Test rate limit (5 requests per minute)
        for _ in 0..5 {
            let result = wallet.send_payment(user_id, script.clone(), amount, fee).await;
            assert!(result.is_err()); // Mock transaction failure
        }
        let result = wallet.send_payment(user_id, script, amount, fee).await;
        assert!(matches!(result, Err(ZipError::RateLimit(_))));
    }

    #[tokio::test]
    async fn test_wallet_balance_rate_limit() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new().unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let user_id = Uuid::new_v4();

        // Test rate limit (5 requests per minute)
        for _ in 0..5 {
            let result = wallet.update_balance(user_id, "USD").await;
            assert!(result.is_ok()); // Mock balance update
        }
        let result = wallet.update_balance(user_id, "USD").await;
        assert!(matches!(result, Err(ZipError::RateLimit(_))));
    }
}

use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::config::Config;
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::utils::setup_logging;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_to_wallet_flow() {
        let config = Config::load().unwrap();
        setup_logging(&config).unwrap();

        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();

        // Mock OAuth signup
        let (user_id, _) = oauth
            .complete_oauth_flow(
                "mock_code".to_string(),
                PkceCodeVerifier::new("mock_verifier".to_string()),
                "mock_csrf".to_string(),
            )
            .await
            .unwrap();

        // Mock Passkey login with 2FA
        let result = passkey.start_authentication(user_id, Some("123456")).await;
        assert!(result.is_ok());

        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let (balance, _) = wallet.update_balance(user_id, "USD").await.unwrap();
        assert_eq!(balance, 0);

        let address = wallet.get_address().unwrap();
        assert!(!address.is_empty());
    }

    #[tokio::test]
    async fn test_payment_flow() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
        let user_id = Uuid::new_v4();

        // Pre-create UTXOs
        tx_manager.pre_create_utxos(user_id, 5, 10000).await.unwrap();

        // Resolve PayMail and send payment
        let (recipient_script, _) = paymail.resolve_paymail("000@zip.io", 10000).await.unwrap();
        let tx_hex = wallet.send_payment(user_id, recipient_script, 8000, 1000).await.unwrap();
        assert!(!tx_hex.is_empty());
    }

    #[tokio::test]
    async fn test_paymail_alias_purchase() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let user_id = Uuid::new_v4();

        // Create default alias (free)
        let (alias, price) = paymail.create_default_alias(user_id, None).await.unwrap();
        assert!(alias.starts_with("101@"));
        assert_eq!(price, Decimal::ZERO);

        // Create paid alias
        tx_manager.pre_create_utxos(user_id, 5, 10000).await.unwrap();
        let (alias, price) = paymail.create_paid_alias(user_id, "54321").await.unwrap();
        assert_eq!(alias, "54321@zip.io");
        assert_eq!(price, Decimal::from(10));

        let (recipient_script, _) = paymail.resolve_paymail("000@zip.io", 10000).await.unwrap();
        wallet.send_payment(user_id, recipient_script, 10000, 1000).await.unwrap();
        paymail.confirm_alias(user_id, &alias).await.unwrap();

        let aliases = paymail.get_user_aliases(user_id).await.unwrap();
        assert!(aliases.contains("54321@zip.io"));
    }
}

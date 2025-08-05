use dioxus::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::config::Config;
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::ui::components::{AuthForm, Dashboard, History, Logout, NavBar, PaymentForm, Settings, WalletOverview};
use crate::ui::router::AppRouter;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_form_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(oauth).with_context(passkey)
        });
        let html = app.render_to_string();
        assert!(html.contains("Sign Up with OAuth"));
        assert!(html.contains("Login with Passkey"));
        assert!(html.contains("2FA Code"));
    }

    #[tokio::test]
    async fn test_wallet_overview_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(wallet).with_context(paymail)
        });
        let html = app.render_to_string();
        assert!(html.contains("Wallet Overview"));
        assert!(html.contains("Primary PayMail"));
    }

    #[tokio::test]
    async fn test_payment_form_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let priv_key = PrivateKey::new();
        let paymail = PaymailManager::new(priv_key, Arc::clone(&storage));

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(paymail)
        });
        let html = app.render_to_string();
        assert!(html.contains("Recipient PayMail"));
        assert!(html.contains("Swipe to Pay"));
    }

    #[tokio::test]
    async fn test_history_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(wallet).with_context(rustbus)
        });
        let html = app.render_to_string();
        assert!(html.contains("Token"));
        assert!(html.contains("Amount"));
        assert!(html.contains("TXID"));
    }

    #[tokio::test]
    async fn test_settings_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(wallet).with_context(paymail).with_context(passkey)
        });
        let html = app.render_to_string();
        assert!(html.contains("Default Currency"));
        assert!(html.contains("PayMail Addresses"));
        assert!(html.contains("Enable 2FA"));
    }

    #[tokio::test]
    async fn test_paymail_alias_purchase() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
        let user_id = Uuid::new_v4();

        // Create default alias
        let (alias, price) = paymail.create_default_alias(user_id, None).await.unwrap();
        assert!(alias.starts_with("101@"));

        // Simulate Settings UI interaction
        let (alias, price) = paymail.create_paid_alias(user_id, "54321").await.unwrap();
        let satoshis = (price * Decimal::from(100_000_000) / wallet.fetch_price("USD").await.unwrap_or(Decimal::ONE))
            .to_u64()
            .unwrap_or(0);
        let (script, _) = paymail.resolve_paymail("000@zip.io", satoshis).await.unwrap();
        wallet.send_payment(user_id, script, satoshis, 1000).await.unwrap();
        paymail.confirm_alias(user_id, &alias).await.unwrap();

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(wallet).with_context(paymail)
        });
        let html = app.render_to_string();
        assert!(html.contains("54321@zip.io"));
        assert!(html.contains("Pay 10 USD"));
    }

    #[tokio::test]
    async fn test_navbar_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
        let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
        let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(wallet)
                .with_context(paymail)
                .with_context(oauth)
                .with_context(passkey)
                .with_context(tx_manager)
                .with_context(rustbus)
        });
        let html = app.render_to_string();
        assert!(html.contains("Home"));
        assert!(html.contains("Wallet"));
        assert!(html.contains("Send"));
        assert!(html.contains("History"));
        assert!(html.contains("Settings"));
    }

    #[tokio::test]
    async fn test_logout_render() {
        let storage = Arc::new(ZipStorage::new().unwrap());
        let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
        let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();

        let app = VirtualDom::new_with_props(AppRouter, |c| {
            c.with_context(oauth).with_context(passkey)
        });
        let html = app.render_to_string();
        assert!(html.contains("Confirm Logout"));
    }
}

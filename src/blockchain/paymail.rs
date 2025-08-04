use paymail::{PaymailClient, PaymentRequest};
use rust_decimal::Decimal;
use rust_sv::private_key::PrivateKey;
use rust_sv::script::Script;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::errors::ZipError;
use crate::paymail_config::PaymailConfig;
use crate::storage::ZipStorage;

pub struct PaymailManager {
    client: Mutex<PaymailClient>,
    config: PaymailConfig,
    storage: Arc<ZipStorage>,
}

impl PaymailManager {
    /// Initializes Paymail client with private key and configuration.
    pub fn new(priv_key: PrivateKey, storage: Arc<ZipStorage>) -> Self {
        Self {
            client: Mutex::new(PaymailClient::new(&priv_key)),
            config: PaymailConfig::load(),
            storage,
        }
    }

    /// Resolves PayMail to payment script and amount.
    pub async fn resolve_paymail(&self, handle: &str, amount: u64) -> Result<(Script, u64), ZipError> {
        let guard = self.client.lock().await;
        let req = PaymentRequest {
            amount: Some(amount),
            ..Default::default()
        };
        let output = guard
            .get_payment_destination(handle, req)
            .await
            .map_err(|e| ZipError::Blockchain(e.to_string()))?;
        Ok((output.script, output.amount.unwrap_or(amount)))
    }

    /// Sends transaction P2P if supported, else returns placeholder for node broadcast.
    pub async fn send_p2p_tx(
        &self,
        handle: &str,
        tx_hex: &str,
        metadata: Value,
        reference: &str,
    ) -> Result<String, ZipError> {
        let guard = self.client.lock().await;
        if guard
            .has_capability(handle, "p2pTx")
            .await
            .map_err(|e| ZipError::Blockchain(e.to_string()))?
        {
            let txid = guard
                .send_p2p_tx(handle, tx_hex, metadata, reference)
                .await
                .map_err(|e| ZipError::Blockchain(e.to_string()))?;
            Ok(txid)
        } else {
            Ok("fallback_txid".to_string()) // Placeholder
        }
    }

    /// Creates a new PayMail alias and returns its price in USD.
    pub async fn create_alias(&self, user_id: Uuid, prefix: &str) -> Result<(String, Decimal), ZipError> {
        self.config.validate_prefix(prefix)?;
        let aliases = self.get_user_aliases(user_id).await?;
        let is_first = aliases.is_empty();
        let price = self.config.get_prefix_price(prefix, is_first);
        let alias = if is_first && prefix == "101" {
            format!("101@{}", self.config.domain)
        } else {
            format!("{}@{}", prefix, self.config.domain)
        };

        // Store alias (pending payment)
        let mut aliases = aliases;
        aliases.insert(alias.clone());
        let serialized = bincode::serialize(&aliases).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage
            .store_user_data(user_id, &serialized)
            .map_err(|e| ZipError::Storage(e))?;

        Ok((alias, price))
    }

    /// Confirms alias after payment.
    pub async fn confirm_alias(&self, user_id: Uuid, alias: &str) -> Result<(), ZipError> {
        let aliases = self.get_user_aliases(user_id).await?;
        if !aliases.contains(alias) {
            return Err(ZipError::Blockchain("Alias not found".to_string()));
        }
        // Notify PayMail service (placeholder for external API call)
        Ok(())
    }

    /// Retrieves user's PayMail aliases.
    pub async fn get_user_aliases(&self, user_id: Uuid) -> Result<HashSet<String>, ZipError> {
        let data = self.storage.get_user_data(user_id)?;
        Ok(data
            .map(|d| bincode::deserialize(&d).unwrap_or_default())
            .unwrap_or_default())
    }
}

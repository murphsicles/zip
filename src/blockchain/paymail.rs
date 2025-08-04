use paymail::{PaymailClient, PaymentRequest};
use rust_decimal::Decimal;
use rust_sv::private_key::PrivateKey;
use rust_sv::script::Script;
use serde_json::Value;
use std::collections::HashSet;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::errors::ZipError;
use crate::paymail_config::PaymailConfig;
use crate::storage::ZipStorage;

pub struct PaymailManager {
    client: Mutex<PaymailClient>,
    config: PaymailConfig,
    storage: Arc<ZipStorage>,
    next_prefix: Mutex<u64>, // Sequential prefix starting from 101
}

impl PaymailManager {
    /// Initializes Paymail client with private key and configuration.
    pub fn new(priv_key: PrivateKey, storage: Arc<ZipStorage>) -> Self {
        Self {
            client: Mutex::new(PaymailClient::new(&priv_key)),
            config: PaymailConfig::load(),
            storage,
            next_prefix: Mutex::new(101),
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
            Ok("fallback_txid".to_string())
        }
    }

    /// Assigns sequential default PayMail prefix (101, 102, etc.) and creates free alias if requested.
    pub async fn create_default_alias(&self, user_id: Uuid, bespoke_prefix: Option<&str>) -> Result<(String, Decimal), ZipError> {
        let prefix = {
            let mut next = self.next_prefix.lock().await;
            let p = *next;
            *next += 1;
            p.to_string()
        };
        let default_alias = format!("{}@{}", prefix, self.config.domain);

        let aliases = self.get_user_aliases(user_id).await?;
        let is_first = aliases.is_empty();
        let mut new_aliases = aliases;

        // Store default alias
        new_aliases.insert(default_alias.clone());
        let serialized = bincode::serialize(&new_aliases).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;

        // Handle bespoke alias (free if first, 5+ digits, non-excluded)
        if let Some(prefix) = bespoke_prefix {
            self.config.validate_prefix(prefix)?;
            let price = self.config.get_prefix_price(prefix, is_first);
            let bespoke_alias = format!("{}@{}", prefix, self.config.domain);
            new_aliases.insert(bespoke_alias.clone());
            let serialized = bincode::serialize(&new_aliases).map_err(|e| ZipError::Blockchain(e.to_string()))?;
            self.storage.store_user_data(user_id, &serialized)?;
            Ok((bespoke_alias, price))
        } else {
            Ok((default_alias, Decimal::ZERO))
        }
    }

    /// Creates a paid PayMail alias, requiring payment to 000@zip.io.
    pub async fn create_paid_alias(&self, user_id: Uuid, prefix: &str) -> Result<(String, Decimal), ZipError> {
        self.config.validate_prefix(prefix)?;
        let aliases = self.get_user_aliases(user_id).await?;
        let is_first = aliases.is_empty();
        let price = self.config.get_prefix_price(prefix, is_first);
        let alias = format!("{}@{}", prefix, self.config.domain);

        // Store pending alias
        let mut new_aliases = aliases;
        new_aliases.insert(alias.clone());
        let serialized = bincode::serialize(&new_aliases).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;

        Ok((alias, price))
    }

    /// Confirms alias after payment to 000@zip.io.
    pub async fn confirm_alias(&self, user_id: Uuid, alias: &str) -> Result<(), ZipError> {
        let aliases = self.get_user_aliases(user_id).await?;
        if !aliases.contains(alias) {
            return Err(ZipError::Blockchain("Alias not found".to_string()));
        }
        // Notify PayMail service (placeholder)
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

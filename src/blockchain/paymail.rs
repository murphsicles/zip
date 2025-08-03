use paymail::{PaymailClient, PaymentRequest};
use rust_sv::private_key::PrivateKey;
use rust_sv::script::Script;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::errors::ZipError;

pub struct PaymailManager {
    client: Mutex<PaymailClient>,
}

impl PaymailManager {
    /// Initializes Paymail client with private key.
    pub fn new(priv_key: PrivateKey) -> Self {
        Self {
            client: Mutex::new(PaymailClient::new(&priv_key)),
        }
    }

    /// Resolves PayMail to payment script and amount.
    pub async fn resolve_paymail(
        &self,
        handle: &str,
        amount: u64,
    ) -> Result<(Script, u64), ZipError> {
        let req = PaymentRequest {
            amount: Some(amount),
            ..Default::default()
        };
        let guard = self.client.lock().await;
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
            Ok("fallback_txid".to_string()) // Placeholder for node broadcast
        }
    }
}

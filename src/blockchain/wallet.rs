use parking_lot::RwLock;
use rust_sv::address::{addr_encode, AddressType};
use rust_sv::private_key::PrivateKey;
use rust_sv::public_key::PublicKey;
use rust_sv::util::hash160;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::blockchain::transaction::TransactionManager;
use crate::errors::ZipError;
use crate::storage::ZipStorage;

#[derive(Serialize, Deserialize)]
pub struct WalletData {
    pub address: String,
    pub balance: u64, // In satoshis
}

pub struct WalletManager {
    storage: Arc<ZipStorage>,
    tx_manager: Arc<TransactionManager>,
    priv_key: RwLock<PrivateKey>,
}

impl WalletManager {
    /// Initializes wallet with stored or new private key.
    pub fn new(storage: Arc<ZipStorage>, tx_manager: Arc<TransactionManager>) -> Result<Self, ZipError> {
        let priv_key_bytes = storage.get_private_key().unwrap_or_else(|_| {
            let new_key = PrivateKey::new();
            let bytes = Secret::new(new_key.to_bytes());
            storage.store_private_key(bytes).unwrap();
            Secret::new(new_key.to_bytes())
        });
        let priv_key = PrivateKey::from_bytes(priv_key_bytes.expose_secret().clone())?;
        Ok(Self {
            storage,
            tx_manager,
            priv_key: RwLock::new(priv_key),
        })
    }

    /// Generates and returns wallet address.
    pub fn get_address(&self) -> String {
        let pubkey = self.priv_key.read().public_key();
        let pubkey_hash = hash160(pubkey.to_bytes());
        addr_encode(&pubkey_hash, AddressType::P2PKH, rust_sv::network::Network::Mainnet)
    }

    /// Updates and caches balance (placeholder for RustBus integration).
    pub async fn update_balance(&self, user_id: Uuid) -> Result<u64, ZipError> {
        // Placeholder: Query RustBus or node for UTXOs
        let balance = 0; // Replace with actual balance fetch
        let data = WalletData {
            address: self.get_address(),
            balance,
        };
        let serialized = bincode::serialize(&data).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;
        Ok(balance)
    }

    /// Initiates payment using pre-created UTXOs and PayMail script.
    pub async fn send_payment(
        &self,
        recipient_script: Script,
        amount: u64,
        fee: u64,
    ) -> Result<String, ZipError> {
        let tx = self.tx_manager.build_payment_tx(recipient_script, amount, fee).await?;
        let tx_hex = tx.to_hex()?;
        // Placeholder for broadcast; integrate RustBus or node
        Ok(tx_hex) // Return txid after broadcast
    }
}

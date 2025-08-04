use parking_lot::RwLock;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_sv::address::{addr_encode, AddressType};
use rust_sv::private_key::PrivateKey;
use rust_sv::public_key::PublicKey;
use rust_sv::util::hash160;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::blockchain::transaction::TransactionManager;
use crate::errors::ZipError;
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;

#[derive(Serialize, Deserialize)]
pub struct WalletData {
    pub address: String,
    pub balance: u64, // In satoshis
    pub currency: String, // e.g., "USD", "GBP"
    pub balance_converted: Decimal,
}

pub struct WalletManager {
    storage: Arc<ZipStorage>,
    tx_manager: Arc<TransactionManager>,
    rustbus: Option<Arc<RustBusIntegrator>>,
    priv_key: RwLock<PrivateKey>,
    price_cache: RwLock<HashMap<String, Decimal>>,
}

impl WalletManager {
    /// Initializes wallet with stored or new private key and optional RustBus.
    pub fn new(
        storage: Arc<ZipStorage>,
        tx_manager: Arc<TransactionManager>,
        rustbus: Option<Arc<RustBusIntegrator>>,
    ) -> Result<Self, ZipError> {
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
            rustbus,
            priv_key: RwLock::new(priv_key),
            price_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Generates and returns wallet address.
    pub fn get_address(&self) -> String {
        let pubkey = self.priv_key.read().public_key();
        let pubkey_hash = hash160(pubkey.to_bytes());
        addr_encode(&pubkey_hash, AddressType::P2PKH, rust_sv::network::Network::Mainnet)
    }

    /// Fetches BSV price in specified currency, caches for 5min.
    async fn fetch_price(&self, currency: &str) -> Result<Decimal, ZipError> {
        let cache_key = currency.to_string();
        {
            let cache = self.price_cache.read();
            if let Some(price) = cache.get(&cache_key) {
                return Ok(*price);
            }
        }

        let client = Client::new();
        let resp = client
            .get(format!(
                "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin-sv&vs_currencies={}",
                currency.to_lowercase()
            ))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let price = resp["bitcoin-sv"][currency.to_lowercase()]
            .as_f64()
            .ok_or(ZipError::Blockchain("Invalid price data".to_string()))?;
        let price = Decimal::from_f64(price).unwrap_or_default();

        {
            let mut cache = self.price_cache.write();
            cache.insert(cache_key, price);
            // Clear cache after 5min (async cleanup)
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                cache.remove(&cache_key);
            });
        }

        Ok(price)
    }

    /// Updates and caches balance in satoshis and converted currency.
    pub async fn update_balance(&self, user_id: Uuid, currency: &str) -> Result<(u64, Decimal), ZipError> {
        let balance = if let Some(r) = &self.rustbus {
            r.query_balance(&self.get_address()).await?
        } else {
            0 // Fallback
        };
        let price = self.fetch_price(currency).await?;
        let balance_converted = Decimal::from(balance) / Decimal::from(100_000_000) * price;

        let data = WalletData {
            address: self.get_address(),
            balance,
            currency: currency.to_string(),
            balance_converted,
        };
        let serialized = bincode::serialize(&data).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;

        Ok((balance, balance_converted))
    }

    /// Initiates payment using pre-created UTXOs and PayMail script.
    pub async fn send_payment(&self, user_id: Uuid, recipient_script: Script, amount: u64, fee: u64) -> Result<String, ZipError> {
        let tx = self.tx_manager.build_payment_tx(user_id, recipient_script, amount, fee).await?;
        let tx_hex = tx.to_hex()?;
        Ok(tx_hex) // Placeholder for broadcast
    }
}

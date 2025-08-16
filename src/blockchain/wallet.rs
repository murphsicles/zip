use bincode;
use parking_lot::RwLock;
use reqwest::Client;
use rust_decimal::Decimal;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use sv::wallet::ExtendedPrivateKey;
use sv::transaction::Transaction; // Correct import
use sv::messages::Tx; // Added for manual serialization
use sv::script::Script;
use uuid::Uuid;
use hex;

use crate::blockchain::TransactionManager;
use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::integrations::rustbus::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::utils::cache::Cache;
use crate::utils::crypto::Crypto;
use crate::utils::rate_limiter::RateLimiter;
use crate::utils::telemetry::Telemetry;

#[derive(Serialize, Deserialize)]
pub struct WalletData {
    pub address: String,
    pub balance: u64,
    pub currency: String,
    pub balance_converted: Decimal,
    pub derivation_path: String,
}

#[derive(Clone)]
pub struct WalletManager {
    storage: Arc<ZipStorage>,
    tx_manager: Arc<TransactionManager>,
    rustbus: Option<Arc<RustBusIntegrator>>,
    hd_key: Arc<RwLock<ExtendedPrivateKey>>,
    derivation_index: Arc<RwLock<u32>>,
    price_cache: Arc<Cache<String, Decimal>>,
    telemetry: Telemetry,
    rate_limiter: RateLimiter,
}

impl Clone for WalletManager {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            tx_manager: Arc::clone(&self.tx_manager),
            rustbus: self.rustbus.clone().map(Arc::clone),
            hd_key: Arc::clone(&self.hd_key),
            derivation_index: Arc::clone(&self.derivation_index),
            price_cache: Arc::clone(&self.price_cache),
            telemetry: self.telemetry.clone(),
            rate_limiter: self.rate_limiter.clone(),
        }
    }
}

impl WalletManager {
    /// Initializes wallet with HD key or generates new one.
    pub fn new(
        storage: Arc<ZipStorage>,
        tx_manager: Arc<TransactionManager>,
        rustbus: Option<Arc<RustBusIntegrator>>,
    ) -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        let priv_key_bytes = storage.get_private_key().unwrap_or_else(|_| {
            let private_key = Crypto::generate_private_key()?;
            let seed = private_key.to_bytes();
            let hd_key = ExtendedPrivateKey::new_seed(&seed, sv::network::Network::Mainnet)?;
            let bytes = Secret::new(hd_key.to_bytes());
            storage.store_private_key(&bytes)?;
            Secret::new(hd_key.to_bytes())
        });
        let hd_key = ExtendedPrivateKey::from_bytes(priv_key_bytes.expose_secret().clone())?;
        Ok(Self {
            storage,
            tx_manager,
            rustbus,
            hd_key: Arc::new(RwLock::new(hd_key)),
            derivation_index: Arc::new(RwLock::new(0)),
            price_cache: Arc::new(Cache::new(300)), // 5min TTL
            telemetry: Telemetry::new(&config),
            rate_limiter: RateLimiter::new(5, 60), // 5 payments per minute
        })
    }

    /// Generates a new child address for privacy (no reuse).
    pub fn get_address(&self) -> Result<String, ZipError> {
        let index = {
            let mut idx = self.derivation_index.write();
            *idx += 1;
            *idx
        };
        let child_key = self
            .hd_key
            .read()
            .derive_private_key(&[sv::wallet::ChildNumber::Normal { index }])?;
        let pubkey = Crypto::derive_public_key(&child_key);
        let address = Crypto::generate_address(&pubkey);
        // Store derivation path
        let data = WalletData {
            address: address.clone(),
            balance: 0,
            currency: "USD".to_string(),
            balance_converted: Decimal::ZERO,
            derivation_path: format!("m/44'/0'/0'/0/{}", index),
        };
        let serialized =
            bincode::serialize(&data).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        let user_id = Uuid::new_v4();
        self.storage.store_user_data(user_id, &serialized)?;
        let _ = self
            .telemetry
            .track_payment_event(&user_id.to_string(), "address_generated", 0, true)
            .await;
        Ok(address)
    }

    /// Fetches BSV price in specified currency, caches for 5min.
    pub async fn fetch_price(&self, currency: &str) -> Result<Decimal, ZipError> {
        let cache_key = currency.to_string();
        if let Some(price) = self.price_cache.get(&cache_key).await {
            return Ok(price);
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
        let price = Decimal::from_f64(price).ok_or_else(|| ZipError::Blockchain("Invalid price conversion".to_string()))?; // Fixed from try_from_f64
        self.price_cache.insert(cache_key, price).await;
        Ok(price)
    }

    /// Updates and caches balance in satoshis and converted currency.
    pub async fn update_balance(
        &self,
        user_id: Uuid,
        currency: &str,
    ) -> Result<(u64, Decimal), ZipError> {
        self.rate_limiter.check(&user_id.to_string()).await?;
        let address = self.get_address()?;
        let balance = if let Some(r) = &self.rustbus {
            r.query_balance(&address).await?
        } else {
            0
        };
        let price = self.fetch_price(currency).await?;
        let balance_converted = Decimal::from(balance) / Decimal::from(100_000_000) * price;
        let data = WalletData {
            address,
            balance,
            currency: currency.to_string(),
            balance_converted,
            derivation_path: format!("m/44'/0'/0'/0/{}", *self.derivation_index.read()),
        };
        let serialized =
            bincode::serialize(&data).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;
        let _ = self
            .telemetry
            .track_payment_event(&user_id.to_string(), "balance_update", balance, true)
            .await;
        Ok((balance, balance_converted))
    }

    /// Initiates payment using pre-created UTXOs and PayMail script.
    pub async fn send_payment(
        &self,
        user_id: Uuid,
        recipient_script: Script,
        amount: u64,
        fee: u64,
    ) -> Result<String, ZipError> {
        self.rate_limiter.check(&user_id.to_string()).await?;
        let result = self
            .tx_manager
            .build_payment_tx(user_id, recipient_script, amount, fee)
            .await;
        let success = result.is_ok();
        let tx_id = result
            .as_ref()
            .map(|tx| {
                // Manual serialization since to_bytes is not available
                let mut bytes = vec![];
                bytes.extend_from_slice(&tx.version.to_le_bytes());
                bytes.extend_from_slice(&(tx.inputs.len() as u32).to_le_bytes());
                for input in &tx.inputs {
                    bytes.extend_from_slice(&input.previous_output.to_bytes());
                    bytes.extend_from_slice(&input.script.0);
                    bytes.extend_from_slice(&input.sequence.to_le_bytes());
                }
                bytes.extend_from_slice(&(tx.outputs.len() as u32).to_le_bytes());
                for output in &tx.outputs {
                    bytes.extend_from_slice(&output.value.to_le_bytes());
                    bytes.extend_from_slice(&output.script.0);
                }
                bytes.extend_from_slice(&tx.lock_time.to_le_bytes());
                hex::encode(bytes)
            })
            .unwrap_or(Ok(String::new()))?;
        let _ = self
            .telemetry
            .track_payment_event(&user_id.to_string(), &tx_id, amount, success)
            .await;
        result.map(|_| tx_id) // Adjusted to return tx_id on success
    }
}

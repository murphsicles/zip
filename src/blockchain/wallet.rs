use parking_lot::RwLock;
use reqwest::Client;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use rust_sv::address::{addr_encode, AddressType};
use rust_sv::bip32::{ChildNumber, ExtendedPrivateKey};
use rust_sv::private_key::PrivateKey;
use rust_sv::public_key::PublicKey;
use rust_sv::util::hash160;

use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::utils::metrics::Metrics;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct WalletData {
    pub address: String,
    pub balance: u64,
    pub currency: String,
    pub balance_converted: Decimal,
    pub derivation_path: String,
}

pub struct WalletManager {
    storage: Arc<ZipStorage>,
    tx_manager: Arc<TransactionManager>,
    rustbus: Option<Arc<RustBusIntegrator>>,
    hd_key: RwLock<ExtendedPrivateKey>,
    derivation_index: RwLock<u32>,
    price_cache: RwLock<HashMap<String, Decimal>>,
    metrics: Metrics,
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
            let seed = generate_salt(64);
            let hd_key = ExtendedPrivateKey::new_seed(&seed, rust_sv::network::Network::Mainnet)?;
            let bytes = Secret::new(hd_key.to_bytes());
            storage.store_private_key(bytes).unwrap();
            Secret::new(hd_key.to_bytes())
        });
        let hd_key = ExtendedPrivateKey::from_bytes(priv_key_bytes.expose_secret().clone())?;
        Ok(Self {
            storage,
            tx_manager,
            rustbus,
            hd_key: RwLock::new(hd_key),
            derivation_index: RwLock::new(0),
            price_cache: RwLock::new(HashMap::new()),
            metrics: Metrics::new(&config),
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
            .derive_private_key(&[ChildNumber::Normal { index }])?;
        let pubkey = child_key.public_key();
        let pubkey_hash = hash160(pubkey.to_bytes());
        let address = addr_encode(&pubkey_hash, AddressType::P2PKH, rust_sv::network::Network::Mainnet);

        // Store derivation path
        let data = WalletData {
            address: address.clone(),
            balance: 0,
            currency: "USD".to_string(),
            balance_converted: Decimal::ZERO,
            derivation_path: format!("m/44'/0'/0'/0/{}", index),
        };
        let serialized = bincode::serialize(&data).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.store_user_data(Uuid::new_v4(), &serialized)?;
        self.metrics.track_payment_event(&Uuid::new_v4().to_string(), "address_generated", 0, true);
        Ok(address)
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
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                cache.remove(&cache_key);
            });
        }

        Ok(price)
    }

    /// Updates and caches balance in satoshis and converted currency.
    pub async fn update_balance(&self, user_id: Uuid, currency: &str) -> Result<(u64, Decimal), ZipError> {
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
        let serialized = bincode::serialize(&data).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;
        self.metrics.track_payment_event(&user_id.to_string(), "balance_update", balance, true);
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
        let result = self.tx_manager.build_payment_tx(user_id, recipient_script, amount, fee).await;
        let tx_id = match &result {
            Ok(tx) => tx.to_hex()?,
            Err(_) => "".to_string(),
        };
        self.metrics.track_payment_event(&user_id.to_string(), &tx_id, amount, result.is_ok());
        result
    }
            }

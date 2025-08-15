use bincode;
use itertools::Itertools;
use parking_lot::RwLock;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sv::messages::{Transaction, TxIn, TxOut};
use sv::private_key::PrivateKey;
use sv::script::Script;
use sv::transaction::p2pkh::create_lock_script;
use sv::util::hash160;
use uuid::Uuid;

use crate::errors::ZipError;
use crate::integrations::rustbus::RustBusIntegrator;
use crate::storage::ZipStorage;

#[derive(Serialize, Deserialize)]
struct UTXO {
    txid: String,
    vout: u32,
    amount: u64,
    script: Script,
}

pub struct TransactionManager {
    storage: Arc<ZipStorage>,
    rustbus: Option<Arc<RustBusIntegrator>>,
    rng: RwLock<OsRng>,
}

impl TransactionManager {
    /// Creates a new transaction manager with optional RustBus for on-chain data.
    pub fn new(storage: Arc<ZipStorage>, rustbus: Option<Arc<RustBusIntegrator>>) -> Self {
        Self {
            storage,
            rustbus,
            rng: RwLock::new(OsRng),
        }
    }

    /// Pre-creates UTXOs by splitting parent, optimized with coin selection from RustBus.
    pub async fn pre_create_utxos(
        &self,
        user_id: Uuid,
        num_utxos: usize,
        utxo_value: u64,
    ) -> Result<Vec<TxOut>, ZipError> {
        let address = "user_address"; // From wallet
        let balance = if let Some(r) = &self.rustbus {
            r.query_balance(&address).await?
        } else {
            0 // Fallback
        };
        if balance < (utxo_value * num_utxos as u64) {
            return Err(ZipError::Blockchain("Insufficient balance".to_string()));
        }
        let utxos = self.fetch_utxos(user_id).await?;
        let mut tx = Transaction::new();
        let mut input_amount = 0u64;
        for utxo in utxos
            .iter()
            .take_while(|_| input_amount < (utxo_value * num_utxos as u64))
        {
            let input = TxIn::new(&utxo.txid, utxo.vout, utxo.script.clone(), 0xFFFFFFFF);
            tx.add_input(input);
            input_amount += utxo.amount;
        }
        let mut outputs = Vec::with_capacity(num_utxos);
        for _ in 0..num_utxos {
            let mut rng = self.rng.write();
            let mut salt = [0u8; 32];
            (*rng).fill_bytes(&mut salt); // Dereference to access OsRng
            let pubkey_hash = hash160(&salt);
            let script = create_lock_script(&pubkey_hash); // Use p2pkh from transaction module
            let out = TxOut::new(script, utxo_value);
            tx.add_output(out.clone())?;
            outputs.push(out);
        }
        let change = input_amount - (utxo_value * num_utxos as u64);
        if change > 0 {
            let change_script = create_lock_script(&hash160(&[0u8; 20])); // Placeholder
            tx.add_output(TxOut::new(change_script, change))?;
        }
        let priv_key_bytes = self.storage.get_private_key()?;
        let priv_key = PrivateKey::from_bytes(priv_key_bytes.expose_secret().clone())?;
        tx.sign(&priv_key)?;
        let serialized =
            bincode::serialize(&outputs).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        self.storage.cache_utxos(user_id, &serialized)?;
        Ok(outputs)
    }

    /// Builds and signs a payment transaction using optimized coin selection.
    pub async fn build_payment_tx(
        &self,
        user_id: Uuid,
        recipient_script: Script,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction, ZipError> {
        let utxos = self.fetch_utxos(user_id).await?;
        let priv_key_bytes = self.storage.get_private_key()?;
        let priv_key = PrivateKey::from_bytes(priv_key_bytes.expose_secret().clone())?;
        let mut tx = Transaction::new();
        let mut input_amount = 0u64;
        // Optimized coin selection (minimal UTXOs, prefer small for low fees)
        let sorted_utxos = utxos.iter().sorted_by_key(|u| u.amount).collect::<Vec<_>>();
        for utxo in sorted_utxos
            .iter()
            .take_while(|_| input_amount < amount + fee)
        {
            let input = TxIn::new(&utxo.txid, utxo.vout, utxo.script.clone(), 0xFFFFFFFF);
            tx.add_input(input);
            input_amount += utxo.amount;
        }
        tx.add_output(TxOut::new(recipient_script, amount))?;
        let change = input_amount - amount - fee;
        if change > 0 {
            let change_script = create_lock_script(&hash160(priv_key.public_key().to_bytes()));
            tx.add_output(TxOut::new(change_script, change))?;
        }
        tx.sign(&priv_key)?;
        Ok(tx)
    }

    /// Fetches UTXOs from cache or RustBus.
    async fn fetch_utxos(&self, user_id: Uuid) -> Result<Vec<UTXO>, ZipError> {
        if let Some(cached) = self.storage.get_utxos(user_id)? {
            return bincode::deserialize(&cached).map_err(|e| ZipError::Blockchain(e.to_string()));
        }
        if let Some(r) = &self.rustbus {
            let address = "user_address"; // From wallet
            let balance = r.query_balance(&address).await?;
            // Placeholder: Fetch actual UTXOs from RustBus
            let utxos = vec![UTXO {
                txid: "mock".to_string(),
                vout: 0,
                amount: balance,
                script: Script::default(),
            }];
            let serialized =
                bincode::serialize(&utxos).map_err(|e| ZipError::Blockchain(e.to_string()))?;
            self.storage.cache_utxos(user_id, &serialized)?;
            Ok(utxos)
        } else {
            Err(ZipError::Blockchain("No RustBus integrator".to_string()))
        }
    }
}

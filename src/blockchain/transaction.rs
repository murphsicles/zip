use parking_lot::RwLock;
use rand::rngs::OsRng;
use rand::RngCore;
use rust_sv::private_key::PrivateKey;
use rust_sv::script::Script;
use rust_sv::transaction::{Transaction, TxIn, TxOut};
use rust_sv::util::hash160;
use std::sync::Arc;

use crate::errors::ZipError;
use crate::storage::ZipStorage;

pub struct TransactionManager {
    storage: Arc<ZipStorage>,
    rng: RwLock<OsRng>,  // Efficient RNG for UTXO splitting
}

impl TransactionManager {
    /// Creates a new transaction manager with shared storage.
    pub fn new(storage: Arc<ZipStorage>) -> Self {
        Self {
            storage,
            rng: RwLock::new(OsRng),
        }
    }

    /// Pre-creates UTXOs by splitting a parent UTXO into smaller ones for rapid transactions.
    pub fn pre_create_utxos(
        &self,
        parent_txid: &str,
        parent_vout: u32,
        parent_amount: u64,
        num_utxos: usize,
        utxo_value: u64,
    ) -> Result<Vec<TxOut>, ZipError> {
        let priv_key_bytes = self.storage.get_private_key()?;
        let priv_key = PrivateKey::from_bytes(priv_key_bytes.expose_secret().clone())?;

        let mut tx = Transaction::new();
        let input = TxIn::new(parent_txid, parent_vout, Script::default(), 0xFFFFFFFF);
        tx.add_input(input);

        let total_split = (utxo_value * num_utxos as u64).min(parent_amount);
        let mut utxos = Vec::with_capacity(num_utxos);

        for _ in 0..num_utxos {
            let mut rng = self.rng.write();
            let mut salt = [0u8; 32];
            rng.fill_bytes(&mut salt);

            let pubkey_hash = hash160(&salt);  // Unique per UTXO for efficiency
            let script = Script::p2pkh(pubkey_hash);
            let out = TxOut::new(script, utxo_value);
            tx.add_output(out.clone())?;
            utxos.push(out);
        }

        // Change output if needed
        let change = parent_amount - total_split;
        if change > 0 {
            let change_script = Script::p2pkh(hash160(priv_key.public_key().to_bytes()));
            tx.add_output(TxOut::new(change_script, change))?;
        }

        tx.sign(&priv_key)?;

        // Cache UTXOs (serialize and store)
        let serialized = bincode::serialize(&utxos).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        let user_id = Uuid::new_v4();  // Assume from context; replace with actual
        self.storage.cache_utxos(user_id, &serialized)?;

        Ok(utxos)
    }

    /// Builds and signs a simple payment transaction using pre-created UTXOs.
    pub fn build_payment_tx(
        &self,
        recipient_script: Script,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction, ZipError> {
        let user_id = Uuid::new_v4();  // Assume from context
        let cached = self.storage.get_utxos(user_id)?
            .ok_or_else(|| ZipError::Blockchain("No UTXOs cached".to_string()))?;
        let utxos: Vec<TxOut> = bincode::deserialize(&cached).map_err(|e| ZipError::Blockchain(e.to_string()))?;

        let priv_key_bytes = self.storage.get_private_key()?;
        let priv_key = PrivateKey::from_bytes(priv_key_bytes.expose_secret().clone())?;

        let mut tx = Transaction::new();
        let mut input_amount = 0u64;

        // Select minimal UTXOs (coin selection for efficiency)
        for utxo in utxos.iter().take_while(|_| input_amount < amount + fee) {
            let txin = TxIn::new("parent_txid", 0, utxo.script.clone(), 0xFFFFFFFF);  // Placeholder txid/vout
            tx.add_input(txin);
            input_amount += utxo.value;
        }

        tx.add_output(TxOut::new(recipient_script, amount))?;

        let change = input_amount - amount - fee;
        if change > 0 {
            let change_script = Script::p2pkh(hash160(priv_key.public_key().to_bytes()));
            tx.add_output(TxOut::new(change_script, change))?;
        }

        tx.sign(&priv_key)?;

        Ok(tx)
    }
}

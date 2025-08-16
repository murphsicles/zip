use bincode;
use itertools::Itertools;
use parking_lot::RwLock;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sv::messages::{Tx, TxIn, TxOut}; // Renamed Transaction to Tx
use sv::script::Script;
use sv::transaction::{generate_signature, sighash, SigHashCache, SIGHASH_FORKID, SIGHASH_ALL};
use sv::util::hash160;
use secp256k1::{Secp256k1, SecretKey};
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
        let mut tx = Tx {
            version: 2, // Default version
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
        };
        let mut input_amount = 0u64;
        for utxo in utxos
            .iter()
            .take_while(|_| input_amount < (utxo_value * num_utxos as u64))
        {
            let input = TxIn::new(&utxo.txid, utxo.vout, utxo.script.clone(), 0xFFFFFFFF);
            tx.inputs.push(input);
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
            tx.outputs.push(out.clone()); // Direct field mutation
            outputs.push(out);
        }
        let change = input_amount - (utxo_value * num_utxos as u64);
        if change > 0 {
            let change_script = create_lock_script(&hash160(&[0u8; 20])); // Placeholder
            tx.outputs.push(TxOut::new(change_script, change)); // Direct field mutation
        }
        let secp = Secp256k1::new();
        let priv_key_bytes = self.storage.get_private_key()?;
        let priv_key = SecretKey::from_slice(priv_key_bytes.expose_secret().as_ref())
            .map_err(|e| ZipError::Blockchain(e.to_string()))?;
        let mut cache = SigHashCache::new();
        for i in 0..tx.inputs.len() {
            let lock_script = if i < utxos.len() {
                utxos[i].script.clone()
            } else {
                create_lock_script(&hash160(&[0u8; 20])) // Placeholder for missing UTXOs
            };
            let sighash = sighash(&tx, i, &lock_script, 0, SIGHASH_FORKID | SIGHASH_ALL, &mut cache)
                .map_err(|e| ZipError::Blockchain(e.to_string()))?;
            let signature = generate_signature(&priv_key[..], &sighash, SIGHASH_FORKID | SIGHASH_ALL)
                .map_err(|e| ZipError::Blockchain(e.to_string()))?;
            let public_key = secp256k1::PublicKey::from_secret_key(&secp, &priv_key).serialize();
            tx.inputs[i].unlock_script = create_unlock_script(&signature, &public_key);
        }
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
    ) -> Result<Tx, ZipError> {
        let utxos = self.fetch_utxos(user_id).await?;
        let secp = Secp256k1::new();
        let priv_key_bytes = self.storage.get_private_key()?;
        let priv_key = SecretKey::from_slice(priv_key_bytes.expose_secret().as_ref())
            .map_err(|e| ZipError::Blockchain(e.to_string()))?;
        let mut tx = Tx {
            version: 2, // Default version
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
        };
        let mut input_amount = 0u64;
        // Optimized coin selection (minimal UTXOs, prefer small for low fees)
        let sorted_utxos = utxos.iter().sorted_by_key(|u| u.amount).collect::<Vec<_>>();
        for utxo in sorted_utxos
            .iter()
            .take_while(|_| input_amount < amount + fee)
        {
            let input = TxIn::new(&utxo.txid, utxo.vout, utxo.script.clone(), 0xFFFFFFFF);
            tx.inputs.push(input);
            input_amount += utxo.amount;
        }
        tx.outputs.push(TxOut::new(recipient_script, amount)); // Direct field mutation
        let change = input_amount - amount - fee;
        if change > 0 {
            let change_script = create_lock_script(&hash160(
                secp.generate_keypair(&mut OsRng)
                    .1
                    .serialize_uncompressed()
                    .as_ref(),
            ));
            tx.outputs.push(TxOut::new(change_script, change)); // Direct field mutation
        }
        let mut cache = SigHashCache::new();
        for i in 0..tx.inputs.len() {
            let lock_script = if i < utxos.len() {
                utxos[i].script.clone()
            } else {
                create_lock_script(&hash160(&[0u8; 20])) // Placeholder for missing UTXOs
            };
            let sighash = sighash(&tx, i, &lock_script, 0, SIGHASH_FORKID | SIGHASH_ALL, &mut cache)
                .map_err(|e| ZipError::Blockchain(e.to_string()))?;
            let signature = generate_signature(&priv_key[..], &sighash, SIGHASH_FORKID | SIGHASH_ALL)
                .map_err(|e| ZipError::Blockchain(e.to_string()))?;
            let public_key = secp256k1::PublicKey::from_secret_key(&secp, &priv_key).serialize();
            tx.inputs[i].unlock_script = create_unlock_script(&signature, &public_key);
        }
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

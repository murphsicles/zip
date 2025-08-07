use keyring::Entry;
use secrecy::{ExposeSecret, Secret};
use sled::{Db, Tree};
use uuid::Uuid;

use crate::errors::ZipError;

pub struct ZipStorage {
    db: Db,        // Embedded Sled KV store
    secure: Entry, // OS secure storage for private keys
}

impl ZipStorage {
    /// Initializes Sled database and keyring entry for secure key storage.
    pub fn new() -> Result<Self, ZipError> {
        let db = sled::open("zip_db")?;
        let secure = Entry::new("zip", "wallet_keys")?;
        Ok(Self { db, secure })
    }

    /// Stores user data (e.g., PayMail, auth metadata) in Sled.
    pub fn store_user_data(&self, user_id: Uuid, data: &[u8]) -> Result<(), ZipError> {
        self.db.insert(user_id.as_bytes(), data)?;
        self.db.flush()?;
        Ok(())
    }

    /// Retrieves user data by ID.
    pub fn get_user_data(&self, user_id: Uuid) -> Result<Option<sled::IVec>, ZipError> {
        Ok(self.db.get(user_id.as_bytes())?)
    }

    /// Stores private key in OS secure storage.
    pub fn store_private_key(&self, key: Secret<Vec<u8>>) -> Result<(), ZipError> {
        self.secure.set_password(key.expose_secret())?;
        Ok(())
    }

    /// Retrieves private key from secure storage.
    pub fn get_private_key(&self) -> Result<Secret<Vec<u8>>, ZipError> {
        let key = self.secure.get_password()?;
        Ok(Secret::new(key.into_bytes()))
    }

    /// Caches UTXOs for rapid transaction construction.
    pub fn cache_utxos(&self, user_id: Uuid, utxos: &[u8]) -> Result<(), ZipError> {
        let key = format!("utxo:{}", user_id);
        self.db.insert(key.as_bytes(), utxos)?;
        self.db.flush()?;
        Ok(())
    }

    /// Retrieves cached UTXOs.
    pub fn get_utxos(&self, user_id: Uuid) -> Result<Option<sled::IVec>, ZipError> {
        let key = format!("utxo:{}", user_id);
        Ok(self.db.get(key.as_bytes())?)
    }
}

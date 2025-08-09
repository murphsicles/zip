use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ZipError;
use crate::storage::ZipStorage;

#[derive(Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: Uuid,
    pub email: String,
    pub is_authenticated: bool,
}

#[derive(Clone)]
pub struct Session {
    storage: Arc<ZipStorage>,
}

impl Session {
    /// Initializes session manager with storage.
    pub fn new(storage: Arc<ZipStorage>) -> Self {
        Self { storage }
    }

    /// Creates a new session for a user.
    pub async fn create(&self, user_id: Uuid, email: String) -> Result<(), ZipError> {
        let session = SessionData {
            user_id,
            email,
            is_authenticated: true,
        };
        let serialized = bincode::serialize(&session).map_err(|e| ZipError::Auth(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;
        Ok(())
    }

    /// Retrieves session for a user.
    pub async fn get(&self, user_id: Uuid) -> Result<Option<SessionData>, ZipError> {
        let data = self.storage.get_user_data(user_id)?;
        Ok(data.map(|d| bincode::deserialize(&d).unwrap_or_default()))
    }

    /// Clears session for a user.
    pub async fn clear(&self, user_id: Uuid) -> Result<(), ZipError> {
        self.storage.store_user_data(user_id, &[])?;
        Ok(())
    }

    /// Checks if a user is authenticated.
    pub async fn is_authenticated(&self, user_id: Uuid) -> bool {
        self.get(user_id)
            .await
            .map(|s| s.map(|session| session.is_authenticated).unwrap_or(false))
            .unwrap_or(false)
    }
}

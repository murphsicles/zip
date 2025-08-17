use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::EnvConfig;
use crate::errors::ZipError;
use crate::storage::ZipStorage;
use crate::utils::telemetry::Telemetry;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct SessionData {
    pub user_id: Uuid,
    pub email: String,
    pub is_authenticated: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct Session {
    storage: Arc<ZipStorage>,
    telemetry: Telemetry,
}

impl Session {
    /// Initializes session with storage and telemetry.
    pub fn new(storage: Arc<ZipStorage>) -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        Ok(Self {
            storage,
            telemetry: Telemetry::new(&config),
        })
    }

    /// Creates a new session for a user.
    pub async fn create(&self, user_id: Uuid, email: String) -> Result<(), ZipError> {
        let session = SessionData {
            user_id,
            email,
            is_authenticated: true,
            created_at: chrono::Utc::now(),
        };
        let serialized =
            bincode::serialize(&session).map_err(|e| ZipError::Storage(e.to_string()))?;
        self.storage.store_user_data(user_id, &serialized)?;
        let _ = self
            .telemetry
            .track_auth_event(&user_id.to_string(), "session_create", true)
            .await;
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
        let _ = self
            .telemetry
            .track_auth_event(&user_id.to_string(), "session_clear", true)
            .await;
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

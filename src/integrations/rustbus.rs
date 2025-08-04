use rustbus::{Client, Query};
use tokio::sync::Mutex;

use crate::errors::ZipError;

pub struct RustBusIntegrator {
    client: Mutex<Client>,
}

impl RustBusIntegrator {
    /// Initializes RustBus client for indexing queries.
    pub fn new(endpoint: &str) -> Result<Self, ZipError> {
        let client = Client::new(endpoint)?;
        Ok(Self { client: Mutex::new(client) })
    }

    /// Queries balance for a given address.
    pub async fn query_balance(&self, address: &str) -> Result<u64, ZipError> {
        let guard = self.client.lock().await;
        let query = Query::balance(address);
        let response = guard.execute(query).await
            .map_err(|e| ZipError::Blockchain(e.to_string()))?;
        response.balance.ok_or(ZipError::Blockchain("No balance found".to_string()))
    }

    /// Queries transaction history for a user.
    pub async fn query_tx_history(&self, user_id: Uuid) -> Result<Vec<String>, ZipError> {
        let guard = self.client.lock().await;
        let query = Query::tx_history(user_id.to_string());
        let response = guard.execute(query).await
            .map_err(|e| ZipError::Blockchain(e.to_string()))?;
        Ok(response.txs)
    }
}

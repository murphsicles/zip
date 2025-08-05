use rustbus::{Client, Query};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::EnvConfig;
use crate::errors::ZipError;

#[derive(Serialize, Deserialize)]
struct BalanceResponse {
    balance: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct TxHistoryResponse {
    txs: Vec<String>,
}

pub struct RustBusIntegrator {
    client: Mutex<Client>,
}

impl RustBusIntegrator {
    /// Initializes RustBus client with endpoint from environment config.
    pub fn new() -> Result<Self, ZipError> {
        let config = EnvConfig::load()?;
        let client = Client::new(&config.rustbus_endpoint).map_err(|e| ZipError::Blockchain(e.to_string()))?;
        Ok(Self {
            client: Mutex::new(client),
        })
    }

    /// Queries balance for an address.
    pub async fn query_balance(&self, address: &str) -> Result<u64, ZipError> {
        let guard = self.client.lock().await;
        let query = Query::balance(address);
        let response: BalanceResponse = guard
            .execute(query)
            .await
            .map_err(|e| ZipError::Blockchain(e.to_string()))?;
        response
            .balance
            .ok_or_else(|| ZipError::Blockchain("No balance found".to_string()))
    }

    /// Queries transaction history for a user.
    pub async fn query_tx_history(&self, user_id: Uuid) -> Result<Vec<String>, ZipError> {
        let guard = self.client.lock().await;
        let query = Query::tx_history(user_id.to_string());
        let response: TxHistoryResponse = guard
            .execute(query)
            .await
            .map_err(|e| ZipError::Blockchain(e.to_string()))?;
        Ok(response.txs)
    }
}

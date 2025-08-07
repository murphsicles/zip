use nprint::{Runtime, Template};
use tokio::runtime::Handle;

use crate::errors::ZipError;

pub struct NPrintIntegrator {
    runtime: Runtime,
}

impl NPrintIntegrator {
    /// Initializes nPrint runtime for scripting.
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    /// Deploys a payment script using nPrint template.
    pub async fn deploy_payment_script(
        &self,
        template: Template,
        node_url: &str,
    ) -> Result<String, ZipError> {
        let handle = Handle::current();
        let txid = handle
            .spawn_blocking(move || {
                let script = template.compile()?;
                self.runtime.deploy(&script, node_url)
            })
            .await
            .map_err(|e| ZipError::Blockchain(e.to_string()))??;
        Ok(txid)
    }

    /// Verifies a deployed script.
    pub async fn verify_script(&self, txid: &str, node_url: &str) -> Result<bool, ZipError> {
        let handle = Handle::current();
        let valid = handle
            .spawn_blocking(move || self.runtime.verify(txid, node_url))
            .await
            .map_err(|e| ZipError::Blockchain(e.to_string()))??;
        Ok(valid)
    }
}

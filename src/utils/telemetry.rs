use tracing::info;
use uuid::Uuid;

use crate::config::EnvConfig;
use crate::errors::ZipError;

pub struct Telemetry {
    enabled: bool,
    endpoint: Option<String>,
}

impl Telemetry {
    /// Initializes telemetry with optional external endpoint from config.
    pub fn new(config: &EnvConfig) -> Self {
        let enabled = config.log_level == "debug";
        let endpoint = if enabled {
            env::var("TELEMETRY_ENDPOINT").ok()
        } else {
            None
        };
        Self { enabled, endpoint }
    }

    /// Tracks an authentication event (e.g., login success/failure) and sends to endpoint if configured.
    pub async fn track_auth_event(&self, user_id: &str, event: &str, success: bool) -> Result<(), ZipError> {
        if !self.enabled {
            return Ok(());
        }
        info!(
            event = event,
            user_id = user_id,
            success = success,
            "Authentication event"
        );
        if let Some(endpoint) = &self.endpoint {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "event": event,
                "user_id": user_id,
                "success": success,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            client
                .post(endpoint)
                .json(&payload)
                .send()
                .await
                .map_err(|e| ZipError::Network(e))?;
        }
        Ok(())
    }

    /// Tracks a payment event (e.g., transaction success/failure) and sends to endpoint if configured.
    pub async fn track_payment_event(&self, user_id: &str, tx_id: &str, amount: u64, success: bool) -> Result<(), ZipError> {
        if !self.enabled {
            return Ok(());
        }
        info!(
            event = "payment",
            user_id = user_id,
            tx_id = tx_id,
            amount = amount,
            success = success,
            "Payment event"
        );
        if let Some(endpoint) = &self.endpoint {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "event": "payment",
                "user_id": user_id,
                "tx_id": tx_id,
                "amount": amount,
                "success": success,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            client
                .post(endpoint)
                .json(&payload)
                .send()
                .await
                .map_err(|e| ZipError::Network(e))?;
        }
        Ok(())
    }
}

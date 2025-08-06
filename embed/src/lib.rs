use dioxus::prelude::*;
use dioxus_motion::use_gesture;
use paymail::{PaymailClient, PaymentRequest};
use rust_sv::private_key::PrivateKey;
use rust_sv::script::Script;
use rust_sv::transaction::{Transaction, TxOut};
use serde::{Deserialize, Serialize};
use zip::blockchain::transaction::TransactionManager;
use zip::errors::ZipError;
use zip::storage::ZipStorage;

#[derive(Deserialize, Serialize)]
pub struct SwipeButtonProps {
    pub recipient: String,
    pub amount: u64,
    pub private_key: Vec<u8>,
    pub endpoint: Option<String>, // Optional RustBus endpoint for on-chain data
}

#[component]
pub fn SwipeButton(props: SwipeButtonProps) -> Element {
    let storage = use_memo(|| Arc::new(ZipStorage::new().unwrap()));
    let tx_manager = use_memo(|| TransactionManager::new(Arc::clone(&storage), None)); // No RustBus for embed by default
    let is_swiped = use_signal(|| false);

    let gesture = use_gesture(|g| {
        g.on_swipe(|evt| {
            if evt.delta_x > 100.0 && !is_swiped() {
                spawn(async move {
                    let priv_key = PrivateKey::from_bytes(props.private_key.clone())
                        .map_err(|e| ZipError::Blockchain(e.to_string()))?;
                    let client = PaymailClient::new(&priv_key);
                    let req = PaymentRequest {
                        amount: Some(props.amount),
                        ..Default::default()
                    };
                    let (script, resolved_amount) = client
                        .get_payment_destination(&props.recipient, req)
                        .await
                        .map_err(|e| ZipError::Blockchain(e.to_string()))?;

                    // Pre-create UTXOs if needed (minimal for embed)
                    let user_id = Uuid::new_v4();
                    tx_manager
                        .pre_create_utxos(user_id, 10, props.amount / 10)
                        .await
                        .unwrap_or_default();

                    let tx = tx_manager
                        .build_payment_tx(user_id, script, resolved_amount, 1000)
                        .await
                        .map_err(|e| ZipError::Blockchain(e.to_string()))?;

                    let tx_hex = tx.to_hex()?;
                    client
                        .send_p2p_tx(&props.recipient, &tx_hex, Value::Null, "embed-ref")
                        .await
                        .map_err(|e| ZipError::Blockchain(e.to_string()))?;

                    is_swiped.set(true);
                });
            }
        })
    });

    rsx! {
        div {
            class: "swipe-button",
            gesture: "{gesture}",
            style: if is_swiped() { "transform: translateX(100px); transition: transform 0.3s ease;" } else { "" },
            "Swipe to Pay {props.amount} satoshis to {props.recipient}"
        }
    }
}

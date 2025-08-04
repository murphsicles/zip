use dioxus::prelude::*;
use paymail::{PaymailClient, PaymentRequest};
use rust_sv::private_key::PrivateKey;
use rust_sv::script::Script;
use serde::{Deserialize, Serialize};
use zip::errors::ZipError;

#[derive(Deserialize, Serialize)]
pub struct SwipeButtonProps {
    pub recipient: String,
    pub amount: u64,
    pub private_key: Vec<u8>,
}

#[component]
pub fn SwipeButton(props: SwipeButtonProps) -> Element {
    let is_swiped = use_signal(|| false);

    let on_swipe = move |_| async move {
        if !is_swiped() {
            let priv_key = PrivateKey::from_bytes(props.private_key.clone())
                .map_err(|e| ZipError::Blockchain(e.to_string()))?;
            let client = PaymailClient::new(&priv_key);
            let req = PaymentRequest {
                amount: Some(props.amount),
                ..Default::default()
            };
            let (script, amount) = client
                .get_payment_destination(&props.recipient, req)
                .await
                .map_err(|e| ZipError::Blockchain(e.to_string()))?;

            // Placeholder: Build and sign tx (integrate with zip::blockchain)
            let tx = Transaction::new();
            tx.add_output(TxOut::new(script, amount))?;
            tx.sign(&priv_key)?;
            // Broadcast tx (P2P or node)

            is_swiped.set(true);
        }
    };

    rsx! {
        div {
            class: "swipe-button",
            style: if is_swiped() { "transform: translateX(100px); transition: transform 0.3s;" } else { "" },
            ongesture: on_swipe,
            "Swipe to Pay {props.amount} satoshis to {props.recipient}"
        }
    }
}

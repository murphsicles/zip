use dioxus::prelude::*;
use dioxus_motion::use_gesture;
use zip::blockchain::{PaymailManager, WalletManager};
use zip::errors::ZipError;

#[component]
pub fn SwipeButton(recipient: String, amount: u64) -> Element {
    let wallet = use_context::<WalletManager>();
    let paymail = use_context::<PaymailManager>();
    let is_swiped = use_signal(|| false);

    let gesture = use_gesture(|g| {
        g.on_swipe(|evt| {
            if evt.delta_x > 100.0 && !is_swiped() {
                spawn(async move {
                    let (script, resolved_amount) = paymail
                        .resolve_paymail(&recipient, amount)
                        .await
                        .unwrap_or_else(|e| panic!("Paymail resolution failed: {}", e));
                    let txid = wallet
                        .send_payment(script, resolved_amount, 1000)
                        .await
                        .unwrap_or_else(|e| panic!("Payment failed: {}", e));
                    is_swiped.set(true);
                });
            }
        })
    });

    rsx! {
        div {
            class: "swipe-button",
            gesture: "{gesture}",
            style: if is_swiped() {
                "transform: translateX(100px); transition: transform 0.3s ease;"
            } else {
                ""
            },
            "Swipe to Pay {amount} satoshis to {recipient}"
        }
    }
}

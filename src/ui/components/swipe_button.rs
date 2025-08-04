use dioxus::prelude::*;
use dioxus_motion::use_gesture;
use zip::blockchain::WalletManager;

#[component]
pub fn SwipeButton(recipient: String, amount: u64) -> Element {
    let wallet = use_context::<WalletManager>();
    let is_swiped = use_signal(|| false);

    let gesture = use_gesture(|g| g.on_swipe(|evt| {
        if evt.delta_x > 100.0 {
            is_swiped.set(true);
            // Trigger payment
            spawn(async move {
                let script = Script::default(); // From PayMail resolution
                wallet.send_payment(script, amount, 1000).await.unwrap();
            });
        }
    }));

    rsx! {
        div {
            class: "swipe-button",
            gesture: "{gesture}",
            style: if is_swiped { "transform: translateX(100px); transition: transform 0.3s;" } else { "" },
            "Swipe to Pay {amount} satoshis to {recipient}"
        }
    }
}

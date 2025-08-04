use dioxus::prelude::*;
use zip::blockchain::WalletManager;

#[component]
pub fn Dashboard() -> Element {
    let wallet = use_context::<WalletManager>();
    let balance = use_signal(|| 0u64);

    use_effect(move || async move {
        let user_id = Uuid::new_v4(); // From state
        let new_balance = wallet.update_balance(user_id).await.unwrap();
        balance.set(new_balance);
    });

    rsx! {
        div {
            class: "dashboard",
            h2 { "Wallet Dashboard" }
            p { "Balance: {balance} satoshis" }
            p { "Address: {wallet.get_address()}" }
        }
    }
}

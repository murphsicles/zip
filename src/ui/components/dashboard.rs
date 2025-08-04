use dioxus::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::blockchain::WalletManager;
use crate::ui::styles::global_styles;
use crate::ui::transitions::fade_in;

#[component]
pub fn Dashboard() -> Element {
    let wallet = use_context::<WalletManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let balance = use_signal(|| 0u64);
    let balance_converted = use_signal(|| Decimal::ZERO);
    let currency = use_signal(|| "USD".to_string());

    use_effect(move || async move {
        let (bsv, usd) = wallet
            .update_balance(*user_id.read(), &currency.read())
            .await
            .unwrap_or((0, Decimal::ZERO));
        balance.set(bsv);
        balance_converted.set(usd);
    });

    fade_in(rsx! {
        div {
            class: "dashboard",
            style: "{global_styles()} .dashboard { display: flex; flex-direction: column; align-items: center; padding: 20px; } .balance-main { font-size: 2.5em; font-weight: bold; color: #333; } .balance-sub { font-size: 1.2em; color: #666; } @media (max-width: 600px) { .balance-main { font-size: 2em; } }",
            h2 { class: "title", "Welcome to Your Wallet" }
            div { class: "balance-main", "${balance_converted:.2} {currency}" }
            div { class: "balance-sub", "{balance} satoshis (BSV)" }
            Link { to: Route::Payment, "Send Payment" }
            Link { to: Route::History, "Transaction History" }
            Link { to: Route::Settings, "Settings" }
        }
    })
}

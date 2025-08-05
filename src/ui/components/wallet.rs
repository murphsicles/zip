use dioxus::prelude::*;
use dioxus_motion::use_animated;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::blockchain::{PaymailManager, WalletManager};
use crate::errors::ZipError;
use crate::ui::components::{ErrorDisplay, Notification};
use crate::ui::styles::global_styles;
use crate::ui::transitions::fade_in;

#[component]
pub fn WalletOverview() -> Element {
    let wallet = use_context::<WalletManager>();
    let paymail = use_context::<PaymailManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let balance = use_signal(|| 0u64);
    let balance_converted = use_signal(|| Decimal::ZERO);
    let currency = use_signal(|| "USD".to_string());
    let primary_paymail = use_signal(|| String::new());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));

    use_effect(move || async move {
        // Load balance and currency
        match wallet.update_balance(*user_id.read(), &currency.read()).await {
            Ok((bsv, usd)) => {
                balance.set(bsv);
                balance_converted.set(usd);
            }
            Err(e) => error.set(Some(e)),
        }
        // Load primary PayMail
        match paymail.get_user_aliases(*user_id.read()).await {
            Ok(aliases) => {
                if let Some(primary) = aliases.iter().next() {
                    primary_paymail.set(primary.clone());
                }
            }
            Err(e) => error.set(Some(e)),
        }
    });

    rsx! {
        fade_in(rsx! {
            div {
                class: "wallet-overview",
                style: "{global_styles()} .wallet-overview { display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 15px; } .balance-main { font-size: 2.5em; font-weight: bold; color: #333; } .balance-sub { font-size: 1.2em; color: #666; } .paymail { font-size: 1.1em; color: #444; } @media (max-width: 600px) { .balance-main { font-size: 2em; } }",
                style: "{animated}",
                h2 { class: "title", "Wallet Overview" }
                div { class: "balance-main", "${balance_converted:.2} {currency}" }
                div { class: "balance-sub", "{balance} satoshis (BSV)" }
                div { class: "paymail", "Primary PayMail: {primary_paymail}" }
                Link { to: Route::Payment, "Send Payment" }
                Link { to: Route::History, "Transaction History" }
                Link { to: Route::Settings, "Settings" }
                ErrorDisplay { error: *error.read() }
                Notification { message: *notification.read(), is_success: true }
            }
        })
    }
}

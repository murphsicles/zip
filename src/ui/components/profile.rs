use dioxus::prelude::*;
use dioxus_motion::use_animated;
use dioxus_router::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::auth::{AuthManager, SessionManager};
use crate::blockchain::{PaymailManager, WalletManager};
use crate::errors::ZipError;
use crate::ui::components::{ErrorDisplay, Loading, Notification};
use crate::ui::styles::global_styles;

#[component]
pub fn Profile() -> Element {
    let auth = use_context::<AuthManager>();
    let wallet = use_context::<WalletManager>();
    let paymail = use_context::<PaymailManager>();
    let session = use_context::<SessionManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let email = use_signal(|| String::new());
    let primary_paymail = use_signal(|| String::new());
    let balance = use_signal(|| 0u64);
    let balance_converted = use_signal(|| Decimal::ZERO);
    let currency = use_signal(|| "USD".to_string());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let is_loading = use_signal(|| true);
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));

    use_effect(move || async move {
        // Check authentication
        if !session.is_authenticated(*user_id.read()).await {
            use_router().push(Route::Auth);
            return;
        }

        // Load session data
        match session.get_session(*user_id.read()).await {
            Ok(Some(session_data)) => {
                email.set(session_data.email);
            }
            Ok(None) => error.set(Some(ZipError::Auth("No session found".to_string()))),
            Err(e) => error.set(Some(e)),
        }

        // Load balance
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
                } else {
                    primary_paymail.set("None".to_string());
                }
            }
            Err(e) => error.set(Some(e)),
        }

        is_loading.set(false);
        notification.set(Some("Profile loaded".to_string()));
    });

    rsx! {
        div {
            class: "profile",
            style: "{global_styles()} .profile { display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 15px; max-width: 400px; margin: auto; } .info { font-size: 1.1em; color: #444; } .balance-main { font-size: 2em; font-weight: bold; color: #333; } .balance-sub { font-size: 1.1em; color: #666; } @media (max-width: 600px) { .balance-main { font-size: 1.5em; } }",
            style: "{animated}",
            h2 { class: "title", "Your Profile" }
            div { class: "info", "Email: {email}" }
            div { class: "info", "Primary PayMail: {primary_paymail}" }
            div { class: "balance-main", "${balance_converted:.2} {currency}" }
            div { class: "balance-sub", "{balance} satoshis (BSV)" }
            Link { to: Route::SettingsRoute, "Edit Settings" }
            if *is_loading.read() {
                Loading { message: "Loading profile".to_string() }
            }
            ErrorDisplay { error: *error.read() }
            Notification { message: *notification.read(), is_success: true }
        }
    }
}

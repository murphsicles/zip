use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashSet;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::auth::PasskeyManager;
use crate::blockchain::PaymailManager;
use crate::storage::ZipStorage;
use crate::ui::styles::global_styles;
use crate::utils::generate_salt;

#[component]
pub fn Settings() -> Element {
    let storage = use_context::<ZipStorage>();
    let paymail = use_context::<PaymailManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let currencies = ["USD", "GBP", "EUR", "JPY", "CAD", "AUD", "CHF", "CNY", "SEK", "NZD"];
    let selected_currency = use_signal(|| "USD".to_string());
    let paymail_aliases = use_signal(|| HashSet::new());
    let primary_paymail = use_signal(|| String::new());
    let two_fa_enabled = use_signal(|| false);
    let two_fa_secret = use_signal(|| None::<String>);
    let two_fa_code = use_signal(|| String::new());
    let qr_code = use_signal(|| String::new());

    use_effect(move || async move {
        // Load preferences
        if let Some(data) = storage.get_user_data(*user_id.read())? {
            let prefs: HashMap<String, String> = bincode::deserialize(&data).unwrap_or_default();
            selected_currency.set(prefs.get("currency").cloned().unwrap_or("USD".to_string()));
            two_fa_enabled.set(prefs.get("2fa_enabled").is_some());
            // Load PayMail aliases (placeholder for PayMail service fetch)
            paymail_aliases.set(["user@domain.com".to_string(), "alias@domain.com".to_string()].iter().cloned().collect());
            primary_paymail.set("user@domain.com".to_string());
        }
    });

    let on_currency_change = move |evt: Event<FormData>| {
        let new_currency = evt.value;
        selected_currency.set(new_currency);
        // Save to storage
        let mut prefs = HashMap::new();
        prefs.insert("currency".to_string(), new_currency);
        let serialized = bincode::serialize(&prefs).unwrap();
        storage.store_user_data(*user_id.read(), &serialized).unwrap();
    };

    let on_primary_paymail_change = move |alias: String| {
        primary_paymail.set(alias.clone());
        // Update PayMail service if needed
    };

    let on_two_fa_toggle = move |_| {
        if *two_fa_enabled.read() {
            two_fa_enabled.set(false);
            // Disable 2FA
            storage.store_user_data(*user_id.read(), b"").unwrap();
        } else {
            let secret = Secret::Raw(generate_salt(20));
            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret.to_bytes().unwrap(),
                Some("Zip Wallet".to_string()),
                "user@domain.com".to_string(),
            ).unwrap();
            let qrcode = totp.get_qr().unwrap();
            qr_code.set(qrcode);
            two_fa_secret.set(Some(totp.secret_base32().unwrap()));
        }
    };

    let on_verify_two_fa = move |_| {
        if let Some(secret) = &*two_fa_secret.read() {
            let totp = TOTP::new_from_secret(secret).unwrap();
            if totp.check_current(&two_fa_code.read())? {
                two_fa_enabled.set(true);
                let mut prefs = HashMap::new();
                prefs.insert("2fa_enabled".to_string(), secret.clone());
                let serialized = bincode::serialize(&prefs).unwrap();
                storage.store_user_data(*user_id.read(), &serialized).unwrap();
                two_fa_secret.set(None);
            }
        }
    };

    rsx! {
        div {
            class: "settings-page",
            style: "{global_styles()} .settings-page { display: flex; flex-direction: column; gap: 20px; padding: 20px; } .section { border: 1px solid #ddd; padding: 15px; border-radius: 8px; } .paymail-list { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }",
            div { class: "section",
                h3 { "Default Currency" }
                select { onchange: on_currency_change,
                    for curr in currencies.iter() {
                        option { selected: *curr == *selected_currency.read(), "{curr}" }
                    }
                }
            }
            div { class: "section",
                h3 { "PayMail Addresses" }
                div { class: "paymail-list",
                    for alias in paymail_aliases.read().iter() {
                        label {
                            input { r#type: "radio", name: "primary_paymail", checked: *alias == *primary_paymail.read(), onclick: move |_| on_primary_paymail_change(alias.clone()) }
                            "{alias} {if *alias == *primary_paymail.read() { '(Primary)' } else { '' }}"
                        }
                    }
                }
            }
            div { class: "section",
                h3 { "Enable 2FA" }
                toggle { checked: *two_fa_enabled.read(), onchange: on_two_fa_toggle }
                if let Some(_) = *two_fa_secret.read() {
                    img { src: "data:image/png;base64,{qr_code.read()}", alt: "2FA QR Code" }
                    input { r#type: "text", placeholder: "Enter verification code", oninput: move |evt| two_fa_code.set(evt.value) }
                    button { onclick: on_verify_two_fa, "Verify" }
                }
            }
        }
    }
}

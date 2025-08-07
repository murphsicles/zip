use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::auth::AuthManager;
use crate::blockchain::{PaymailManager, WalletManager};
use crate::errors::ZipError;
use crate::storage::ZipStorage;
use crate::ui::components::{
    ErrorDisplay, Notification, SwipeButton, Theme, ThemeProvider, ThemeSwitcher,
};
use crate::ui::styles::global_styles;
use crate::utils::auth::AuthUtils;
use crate::utils::validation::Validation;

#[component]
pub fn Settings() -> Element {
    let storage = use_context::<ZipStorage>();
    let paymail = use_context::<PaymailManager>();
    let wallet = use_context::<WalletManager>();
    let auth = use_context::<AuthManager>();
    let auth_utils = use_context::<AuthUtils>();
    let user_id = use_signal(|| Uuid::new_v4());
    let currencies = [
        "USD", "GBP", "EUR", "JPY", "CAD", "AUD", "CHF", "CNY", "SEK", "NZD",
    ];
    let selected_currency = use_signal(|| "USD".to_string());
    let selected_theme = use_signal(|| Theme::Light);
    let paymail_aliases = use_signal(|| HashSet::new());
    let primary_paymail = use_signal(|| String::new());
    let new_alias = use_signal(|| String::new());
    let alias_price = use_signal(|| Decimal::ZERO);
    let two_fa_enabled = use_signal(|| false);
    let two_fa_secret = use_signal(|| None::<String>);
    let two_fa_code = use_signal(|| String::new());
    let qr_code = use_signal(|| String::new());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);

    use_effect(move || async move {
        // Load preferences
        if let Some(data) = storage.get_user_data(*user_id.read()).unwrap_or_default() {
            let prefs: HashMap<String, String> = bincode::deserialize(&data).unwrap_or_default();
            let currency = prefs.get("currency").cloned().unwrap_or("USD".to_string());
            if Validation::validate_currency(&currency).is_ok() {
                selected_currency.set(currency);
            }
            selected_theme.set(
                prefs
                    .get("theme")
                    .map(|t| {
                        if t == "dark" {
                            Theme::Dark
                        } else {
                            Theme::Light
                        }
                    })
                    .unwrap_or(Theme::Light),
            );
            two_fa_enabled.set(prefs.get("2fa_enabled").is_some());
        }
        // Load PayMail aliases
        match paymail.get_user_aliases(*user_id.read()).await {
            Ok(aliases) => {
                paymail_aliases.set(aliases);
                if let Some(primary) = paymail_aliases.read().iter().next() {
                    primary_paymail.set(primary.clone());
                }
            }
            Err(e) => error.set(Some(e)),
        }
        // Assign default PayMail if none exists
        if paymail_aliases.read().is_empty() {
            match paymail.create_default_alias(*user_id.read(), None).await {
                Ok((alias, _)) => {
                    let mut aliases = paymail_aliases.read().clone();
                    aliases.insert(alias.clone());
                    paymail_aliases.set(aliases);
                    primary_paymail.set("101@zip.io".to_string());
                    notification.set(Some(format!("Default PayMail assigned: {}", alias)));
                }
                Err(e) => error.set(Some(e)),
            }
        }
    });

    let on_currency_change = move |evt: Event<FormData>| {
        let new_currency = evt.value.clone();
        if Validation::validate_currency(&new_currency).is_ok() {
            selected_currency.set(new_currency.clone());
            let mut prefs = HashMap::new();
            prefs.insert("currency".to_string(), new_currency);
            if let Some(secret) = two_fa_secret.read().as_ref() {
                prefs.insert("2fa_enabled".to_string(), secret.clone());
            }
            prefs.insert(
                "theme".to_string(),
                match *selected_theme.read() {
                    Theme::Light => "light".to_string(),
                    Theme::Dark => "dark".to_string(),
                },
            );
            let serialized = bincode::serialize(&prefs).unwrap();
            storage
                .store_user_data(*user_id.read(), &serialized)
                .unwrap();
            notification.set(Some("Currency updated".to_string()));
        } else {
            error.set(Some(ZipError::Validation(
                "Invalid currency code".to_string(),
            )));
        }
    };

    let on_theme_change = move |new_theme: Theme| {
        selected_theme.set(new_theme);
        let mut prefs = HashMap::new();
        prefs.insert("currency".to_string(), selected_currency.read().clone());
        if let Some(secret) = two_fa_secret.read().as_ref() {
            prefs.insert("2fa_enabled".to_string(), secret.clone());
        }
        prefs.insert(
            "theme".to_string(),
            match new_theme {
                Theme::Light => "light".to_string(),
                Theme::Dark => "dark".to_string(),
            },
        );
        let serialized = bincode::serialize(&prefs).unwrap();
        storage
            .store_user_data(*user_id.read(), &serialized)
            .unwrap();
        notification.set(Some("Theme updated".to_string()));
    };

    let on_new_alias = move |evt: Event<FormData>| {
        spawn(async move {
            let prefix = evt.value;
            if Validation::validate_paymail_prefix(&prefix).is_ok() {
                new_alias.set(prefix.clone());
                match paymail.create_paid_alias(*user_id.read(), &prefix).await {
                    Ok((_, price)) => alias_price.set(price),
                    Err(e) => error.set(Some(e)),
                }
            } else {
                error.set(Some(ZipError::Validation(
                    "PayMail prefix must be 5 or more digits".to_string(),
                )));
            }
        });
    };

    let on_pay_alias = move || async move {
        if *two_fa_enabled.read() {
            match auth_utils
                .validate_totp(*user_id.read(), &two_fa_code.read())
                .await
            {
                Ok(true) => {}
                Ok(false) | Err(_) => {
                    error.set(Some(ZipError::Auth("Invalid 2FA code".to_string())));
                    return;
                }
            }
        }
        let (alias, price) = match paymail
            .create_paid_alias(*user_id.read(), &new_alias.read())
            .await
        {
            Ok(result) => result,
            Err(e) => {
                error.set(Some(e));
                return;
            }
        };
        let satoshis = (price * Decimal::from(100_000_000)
            / wallet
                .fetch_price(&selected_currency.read())
                .await
                .unwrap_or(Decimal::ONE))
        .to_u64()
        .unwrap_or(0);
        match Validation::validate_amount(satoshis) {
            Ok(()) => {}
            Err(e) => {
                error.set(Some(e));
                return;
            }
        }
        match paymail.resolve_paymail("000@zip.io", satoshis).await {
            Ok((script, _)) => {
                match wallet
                    .send_payment(*user_id.read(), script, satoshis, 1000)
                    .await
                {
                    Ok(_) => {
                        if paymail.confirm_alias(*user_id.read(), &alias).await.is_ok() {
                            let mut aliases = paymail_aliases.read().clone();
                            aliases.insert(alias.clone());
                            paymail_aliases.set(aliases);
                            new_alias.set(String::new());
                            alias_price.set(Decimal::ZERO);
                            notification.set(Some(format!("Alias purchased: {}", alias)));
                        } else {
                            error.set(Some(ZipError::Blockchain(
                                "Failed to confirm alias".to_string(),
                            )));
                        }
                    }
                    Err(e) => error.set(Some(e)),
                }
            }
            Err(e) => error.set(Some(e)),
        }
    };

    let on_primary_paymail_change = move |alias: String| {
        spawn(async move {
            if *two_fa_enabled.read() {
                match auth_utils
                    .validate_totp(*user_id.read(), &two_fa_code.read())
                    .await
                {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        error.set(Some(ZipError::Auth("Invalid 2FA code".to_string())));
                        return;
                    }
                }
            }
            primary_paymail.set(alias.clone());
            notification.set(Some(format!("Primary PayMail set: {}", alias)));
        });
    };

    let on_two_fa_toggle = move |_| {
        spawn(async move {
            if *two_fa_enabled.read() {
                two_fa_enabled.set(false);
                two_fa_secret.set(None);
                storage.store_user_data(*user_id.read(), b"").unwrap();
                notification.set(Some("2FA disabled".to_string()));
            } else {
                match auth_utils
                    .generate_totp(*user_id.read(), &primary_paymail.read())
                    .await
                {
                    Ok((secret, qr)) => {
                        qr_code.set(qr);
                        two_fa_secret.set(Some(secret));
                        notification.set(Some("2FA setup initiated".to_string()));
                    }
                    Err(e) => error.set(Some(e)),
                }
            }
        });
    };

    let on_verify_two_fa = move |_| {
        spawn(async move {
            if let Some(secret) = &*two_fa_secret.read() {
                match auth_utils
                    .validate_totp(*user_id.read(), &two_fa_code.read())
                    .await
                {
                    Ok(true) => {
                        two_fa_enabled.set(true);
                        let mut prefs = HashMap::new();
                        prefs.insert("currency".to_string(), selected_currency.read().clone());
                        prefs.insert("2fa_enabled".to_string(), secret.clone());
                        prefs.insert(
                            "theme".to_string(),
                            match *selected_theme.read() {
                                Theme::Light => "light".to_string(),
                                Theme::Dark => "dark".to_string(),
                            },
                        );
                        let serialized = bincode::serialize(&prefs).unwrap();
                        storage
                            .store_user_data(*user_id.read(), &serialized)
                            .unwrap();
                        two_fa_secret.set(None);
                        qr_code.set(String::new());
                        notification.set(Some("2FA enabled".to_string()));
                    }
                    Ok(false) | Err(_) => {
                        error.set(Some(ZipError::Auth("Invalid 2FA code".to_string())));
                    }
                }
            }
        });
    };

    rsx! {
        ThemeProvider { theme: *selected_theme.read(),
            div {
                class: "settings-page",
                style: "{global_styles()}",
                div { class: "section",
                    h3 { "Default Currency" }
                    select { onchange: on_currency_change,
                        for curr in currencies.iter() {
                            option { selected: *curr == *selected_currency.read(), "{curr}" }
                        }
                    }
                }
                div { class: "section",
                    h3 { "Theme" }
                    ThemeSwitcher {
                        current_theme: *selected_theme.read(),
                        on_theme_change: move |new_theme| on_theme_change(new_theme)
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
                        div { class: "alias-input",
                            input { r#type: "text", placeholder: "New alias prefix (5+ digits)", oninput: on_new_alias }
                            if alias_price.read() > Decimal::ZERO {
                                SwipeButton {
                                    recipient: "000@zip.io",
                                    amount: (alias_price.read() * Decimal::from(100_000_000) / wallet.fetch_price(&selected_currency.read()).await.unwrap_or(Decimal::ONE)).to_u64().unwrap_or(0),
                                    "Pay {alias_price} {selected_currency} for {new_alias}@zip.io"
                                }
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
                ErrorDisplay { error: *error.read() }
                Notification { message: *notification.read(), is_success: true }
            }
        }
    }
}

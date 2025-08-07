use dioxus::prelude::*;
use dioxus_motion::use_animated;
use rust_decimal::Decimal;

use crate::blockchain::{PaymailManager, WalletManager};
use crate::errors::ZipError;
use crate::ui::components::{ErrorDisplay, Loading, Notification, SwipeButton};
use crate::ui::styles::global_styles;

#[component]
pub fn PaymentForm() -> Element {
    let paymail = use_context::<PaymailManager>();
    let wallet = use_context::<WalletManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let recipient = use_signal(|| String::new());
    let amount = use_signal(|| 0u64);
    let currency = use_signal(|| "USD".to_string());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let is_loading = use_signal(|| false);
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));

    let on_submit = move |_| async move {
        if recipient.read().is_empty() || *amount.read() == 0 {
            error.set(Some(ZipError::Blockchain(
                "Invalid recipient or amount".to_string(),
            )));
            return;
        }
        is_loading.set(true);
        match paymail
            .resolve_paymail(&recipient.read(), *amount.read())
            .await
        {
            Ok((script, resolved_amount)) => {
                match wallet
                    .send_payment(*user_id.read(), script, resolved_amount, 1000)
                    .await
                {
                    Ok(txid) => {
                        notification.set(Some(format!("Payment sent: TXID {}", txid)));
                        recipient.set(String::new());
                        amount.set(0);
                    }
                    Err(e) => error.set(Some(e)),
                }
            }
            Err(e) => error.set(Some(e)),
        }
        is_loading.set(false);
    };

    let on_recipient_change = move |evt: Event<FormData>| {
        recipient.set(evt.value.clone());
    };

    let on_amount_change = move |evt: Event<FormData>| {
        let value = evt.value.parse::<u64>().unwrap_or(0);
        amount.set(value);
    };

    rsx! {
        div {
            class: "payment-form",
            style: "{global_styles()} .payment-form { display: flex; flex-direction: column; gap: 10px; padding: 20px; max-width: 400px; margin: auto; }",
            style: "{animated}",
            h2 { class: "title", "Send Payment" }
            input {
                r#type: "text",
                placeholder: "Recipient PayMail (e.g., user@zip.io)",
                oninput: on_recipient_change,
                disabled: *is_loading.read()
            }
            input {
                r#type: "number",
                placeholder: "Amount in satoshis",
                oninput: on_amount_change,
                disabled: *is_loading.read()
            }
            button { onclick: on_submit, disabled: *is_loading.read(), "Submit" }
            if *amount.read() > 0 && !recipient.read().is_empty() {
                SwipeButton {
                    recipient: recipient.read().clone(),
                    amount: *amount.read(),
                    "Pay {amount} satoshis to {recipient}"
                }
            }
            ErrorDisplay { error: *error.read() }
            Notification { message: *notification.read(), is_success: true }
            if *is_loading.read() {
                Loading { message: "Processing payment".to_string() }
            }
        }
    }
}

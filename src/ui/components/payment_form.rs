use dioxus::prelude::*;
use zip::blockchain::PaymailManager;

#[component]
pub fn PaymentForm() -> Element {
    let paymail = use_context::<PaymailManager>();
    let amount = use_signal(|| 0u64);
    let handle = use_signal(|| String::new());

    let on_submit = move |_| async move {
        let (script, amt) = paymail.resolve_paymail(&handle, amount).await.unwrap();
        // Trigger payment with script and amt
    };

    rsx! {
        div {
            class: "payment-form",
            input { r#type: "text", placeholder: "PayMail handle", oninput: move |evt| handle.set(evt.value) }
            input { r#type: "number", placeholder: "Amount in satoshis", oninput: move |evt| amount.set(evt.value.parse().unwrap_or(0)) }
            button { onclick: on_submit, "Submit Payment" }
        }
    }
}

use dioxus::prelude::*;
use dioxus_motion::use_gesture;
use uuid::Uuid;

use crate::blockchain::{PaymailManager, WalletManager};
use crate::errors::ZipError;

#[derive(Props, PartialEq, Clone)]
pub struct SwipeButtonProps {
    recipient: String,
    amount: u64,
    #[props(optional)]
    children: Option<Element>,
}

#[component]
pub fn SwipeButton(props: SwipeButtonProps) -> Element {
    let wallet = use_context::<WalletManager>();
    let paymail = use_context::<PaymailManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let is_swiped = use_signal(|| false);
    let error = use_signal(|| None::<ZipError>);

    let gesture = use_gesture(|g| {
        g.on_swipe(|evt| {
            if evt.delta_x > 100.0 && !*is_swiped.read() {
                spawn(async move {
                    match paymail
                        .resolve_paymail(&props.recipient, props.amount)
                        .await
                    {
                        Ok((script, resolved_amount)) => {
                            match wallet
                                .send_payment(*user_id.read(), script, resolved_amount, 1000)
                                .await
                            {
                                Ok(_) => is_swiped.set(true),
                                Err(e) => error.set(Some(e)),
                            }
                        }
                        Err(e) => error.set(Some(e)),
                    }
                });
            }
        })
    });

    rsx! {
        div {
            class: "swipe-button",
            ongesture: move |_| gesture.call(()),
            style: if *is_swiped.read() {
                "transform: translateX(100px); transition: transform 0.3s ease;"
            } else {
                ""
            },
            if let Some(children) = &props.children {
                {children}
            } else {
                "Swipe to Pay {props.amount} satoshis to {props.recipient}"
            }
            if let Some(err) = error.read().as_ref() {
                div { class: "error", "{err}" }
            }
        }
    }
}

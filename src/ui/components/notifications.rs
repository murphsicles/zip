use dioxus::prelude::*;
use dioxus_motion::use_animated;
use std::time::Duration;

use crate::errors::ZipError;

#[derive(Props, PartialEq)]
pub struct NotificationProps {
    #[props(default)]
    message: Option<String>,
    #[props(default)]
    error: Option<ZipError>,
    #[props(default = false)]
    is_success: bool,
}

#[component]
pub fn Notification(props: NotificationProps) -> Element {
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5).translate_y(-10.0));
    let is_visible = use_signal(|| props.message.is_some() || props.error.is_some());

    // Auto-dismiss after 5 seconds with smooth fade-out
    use_effect(move || {
        if *is_visible.read() {
            spawn(async move {
                tokio::time::sleep(Duration::from_secs(5)).await;
                is_visible.set(false);
            });
        }
    });

    rsx! {
        if *is_visible.read() {
            div {
                class: if props.is_success { "notification success" } else { "notification error" },
                style: "
                    position: fixed; bottom: 20px; right: 20px; padding: 15px; border-radius: 8px;
                    box-shadow: 0 2px 8px rgba(0,0,0,0.2); min-width: 200px; max-width: 400px;
                    font-size: 0.9em; transition: opacity 0.5s ease, transform 0.5s ease; {animated}
                    .success { background-color: #e6ffed; color: #2e7d32; border: 1px solid #4caf50; }
                    .error { background-color: #ffe6e6; color: #d32f2f; border: 1px solid #f44336; }
                ",
                if let Some(msg) = &props.message {
                    "{msg}"
                } else if let Some(err) = &props.error {
                    "{err}"
                }
            }
        }
    }
}

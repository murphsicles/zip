use dioxus::prelude::*;
use dioxus_motion::use_animated;

use crate::errors::ZipError;

#[derive(Props, PartialEq)]
pub struct ErrorProps {
    #[props(default)]
    error: Option<ZipError>,
}

#[component]
pub fn ErrorDisplay(props: ErrorProps) -> Element {
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));

    rsx! {
        if let Some(error) = &props.error {
            div {
                class: "error",
                style: "display: flex; justify-content: center; padding: 10px; background-color: #ffe6e6; border: 1px solid #ff4d4d; border-radius: 4px; color: #ff4d4d; font-size: 0.9em; {animated}",
                "{error}"
            }
        }
    }
}

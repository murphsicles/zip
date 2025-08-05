use dioxus::prelude::*;
use dioxus_motion::use_animated;

use crate::errors::ZipError;
use crate::utils::error::format_zip_error;

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
                style: "display: flex; justify-content: center; padding: 10px; border-radius: 4px; font-size: 0.9em; {animated}",
                "{format_zip_error(error)}"
            }
        }
    }
}

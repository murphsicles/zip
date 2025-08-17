use crate::errors::ZipError;
use dioxus::prelude::*;
use dioxus_motion::use_animate;

#[derive(Props, PartialEq, Clone)]
pub struct ErrorProps {
    #[props(default)]
    error: Option<ZipError>,
}

#[component]
pub fn ErrorDisplay(props: ErrorProps) -> Element {
    let animated = use_animate(|style| style.opacity(1.0).duration(0.5));
    rsx! {
        if let Some(error) = &props.error {
            div {
                class: "error",
                style: "display: flex; justify-content: center; padding: 10px; border-radius: 4px; font-size: 0.9em; {animated}",
                "{error.to_string()}"
            }
        }
    }
}

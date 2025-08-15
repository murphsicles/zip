use dioxus::prelude::*;
use dioxus_motion::use_animated;
use crate::ui::styles::global_styles;

#[derive(Props, PartialEq, Clone)]
pub struct LoadingProps {
    #[props(default = "Loading...".to_string())]
    message: String,
}

#[component]
pub fn Loading(props: LoadingProps) -> Element {
    let animated = use_animated(|style| style.opacity(1.0).duration(0.3));
    rsx! {
        div {
            class: "loading",
            style: "{{{global_styles()}}} .loading {{ display: flex; justify-content: center; align-items: center; padding: 20px; background-color: #f0f0f0; border-radius: 8px; font-size: 1.1em; color: #666; {animated} }} .loading::after {{ content: '...'; animation: dots 1.5s infinite; }} @keyframes dots {{ 0% {{ content: '.'; }} 33% {{ content: '..'; }} 66% {{ content: '...'; }} }}",
            "{props.message}"
        }
    }
}

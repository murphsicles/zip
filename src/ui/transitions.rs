use dioxus::prelude::*;
use dioxus_motion::use_animated;

#[derive(Props, PartialEq, Clone)]
pub struct TransitionProps {
    #[props(optional)]
    pub children: Option<Element>,
}

pub fn fade_in(cx: Scope<TransitionProps>) -> Element {
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));
    cx.render(rsx! {
        div {
            style: "{animated}",
            if let Some(children) = &cx.props.children {
                {children}
            }
        }
    })
}

pub fn slide_right(cx: Scope<TransitionProps>) -> Element {
    let animated = use_animated(|style| style.translate_x(0.0).duration(0.3));
    cx.render(rsx! {
        div {
            style: "{animated}",
            if let Some(children) = &cx.props.children {
                {children}
            }
        }
    })
}

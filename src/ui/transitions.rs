use dioxus::prelude::*;
use dioxus_motion::{use_transition, Transition};

pub fn fade_in(cx: Scope, children: Element) -> Element {
    let transition = use_transition(|t| t.opacity(1.0).duration(0.5));
    rsx! {
        Transition { style: "{transition}", {children} }
    }
}

pub fn slide_left(cx: Scope, children: Element) -> Element {
    let transition = use_transition(|t| t.translate_x(0.0).duration(0.3));
    rsx! {
        Transition { style: "{transition}", {children} }
    }
}

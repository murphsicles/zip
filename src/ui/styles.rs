use dioxus::prelude::*;

pub fn global_styles() -> String {
    r#"
        .auth-form { display: flex; flex-direction: column; align-items: center; padding: 20px; }
        .dashboard { padding: 20px; background-color: #f0f0f0; border-radius: 8px; }
        .payment-form { display: flex; flex-direction: column; gap: 10px; }
        .swipe-button { width: 200px; height: 50px; background-color: #4caf50; color: white; text-align: center; line-height: 50px; transition: transform 0.3s ease; }
        @media (max-width: 600px) { .swipe-button { width: 100%; } }
    "#.to_string()
}

#[component]
pub fn Styles(cx: Scope) -> Element {
    cx.render(rsx! {
        style { "{global_styles()}" }
    })
}

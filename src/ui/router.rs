use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::ui::components::{AuthForm, Dashboard, PaymentForm, SwipeButton};
use crate::ui::transitions::{fade_in, slide_left};

#[derive(Routable, Clone)]
pub enum Route {
    #[route("/")]
    Home,
    #[route("/auth")]
    Auth,
    #[route("/dashboard")]
    DashboardRoute,
    #[route("/payment")]
    Payment,
}

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    slide_left(rsx! { h1 { "Welcome to Zip Wallet" } })
}

#[component]
fn Auth() -> Element {
    fade_in(rsx! { AuthForm {} })
}

#[component]
fn DashboardRoute() -> Element {
    fade_in(rsx! { Dashboard {} })
}

#[component]
fn Payment() -> Element {
    fade_in(rsx! {
        PaymentForm {}
        SwipeButton { recipient: "example@paymail.com", amount: 1000 }
    })
}

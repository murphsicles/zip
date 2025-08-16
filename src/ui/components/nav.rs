use dioxus::prelude::*;
use dioxus_router::*;
use uuid::Uuid;

use crate::ui::router::Route;
use crate::ui::styles::global_styles;
use crate::utils::session::Session;

#[component]
pub fn NavBar(cx: Scope) -> Element {
    let session = use_context::<Session>();
    let user_id = use_signal(|| Uuid::new_v4());
    let is_authenticated = use_signal(|| false);

    use_effect(move || {
        async move {
            is_authenticated.set(session.is_authenticated(*user_id.read()).await);
        }
    });

    rsx! {
        nav {
            class: "navbar",
            style: "{{{global_styles()}}}",
            Link { to: Route::Home, class: "nav-link", "Home" }
            if *is_authenticated.read() {
                rsx! {
                    Link { to: Route::DashboardRoute, class: "nav-link", "Wallet" }
                    Link { to: Route::Payment, class: "nav-link", "Send" }
                    Link { to: Route::HistoryRoute, class: "nav-link", "History" }
                    Link { to: Route::ProfileRoute, class: "nav-link", "Profile" }
                    Link { to: Route::SettingsRoute, class: "nav-link", "Settings" }
                    Link { to: Route::LogoutRoute, class: "nav-link", "Logout" }
                }
            } else {
                rsx! {
                    Link { to: Route::Auth, class: "nav-link", "Sign Up / Login" }
                }
            }
        }
    }
}

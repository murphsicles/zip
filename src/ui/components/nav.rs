use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::ui::styles::global_styles;

#[component]
pub fn NavBar() -> Element {
    rsx! {
        nav {
            class: "navbar",
            style: "{global_styles()} .navbar { display: flex; justify-content: space-around; padding: 10px; background-color: #4caf50; color: white; } .nav-link { color: white; text-decoration: none; padding: 8px 16px; border-radius: 4px; } .nav-link:hover { background-color: #388e3c; } @media (max-width: 600px) { .navbar { flex-direction: column; gap: 10px; } }",
            Link { to: Route::Home, class: "nav-link", "Home" }
            Link { to: Route::DashboardRoute, class: "nav-link", "Wallet" }
            Link { to: Route::Payment, class: "nav-link", "Send" }
            Link { to: Route::HistoryRoute, class: "nav-link", "History" }
            Link { to: Route::SettingsRoute, class: "nav-link", "Settings" }
            Link { to: Route::ProfileRoute, class: "nav-link", "Profile" }
            Link { to: Route::LogoutRoute, class: "nav-link", "Logout" }
        }
    }
}

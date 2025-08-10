use dioxus::prelude::*;
use dioxus_router::prelude::*;
use uuid::Uuid;

use crate::auth::Session;
use crate::ui::router::Route;
use crate::ui::styles::global_styles;
use crate::ui::transitions::slide_right;

#[component]
pub fn Home() -> Element {
    let session = use_context::<Session>();
    let is_authenticated = use_signal(|| false);
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(
        to_owned![session, user_id, is_authenticated],
        || async move {
            is_authenticated.set(session.is_authenticated(*user_id.read()).await);
        },
    );

    slide_right(
        cx,
        rsx! {
            div {
                class: "home",
                style: "{{{global_styles()}}} .home {{ display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 15px; }} .title {{ font-size: 2.5em; color: #333; }} .nav-link {{ color: #007bff; text-decoration: none; padding: 8px 16px; border-radius: 4px; }} .nav-link:hover {{ background-color: #e6f3ff; }} @media (max-width: 600px) {{ .title {{ font-size: 2em; }} }}",
                h1 { class: "title", "Zip Wallet" }
                Link { to: Route::Auth, class: "nav-link", "Sign Up / Login" }
                Link { to: Route::Payment, class: "nav-link", "Make a Payment" }
                Link { to: Route::HistoryRoute, class: "nav-link", "View History" }
                Link { to: Route::SettingsRoute, class: "nav-link", "Settings" }
                if *is_authenticated.read() {
                    Link { to: Route::DashboardRoute, class: "nav-link", "Wallet" }
                    Link { to: Route::LogoutRoute, class: "nav-link", "Logout" }
                }
            }
        },
    )
}

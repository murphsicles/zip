use dioxus::prelude::*;
use dioxus_router::components::RouteRenderer;
use dioxus_router::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;
use sv::private_key::PrivateKey;
use uuid::Uuid;

use crate::auth::AuthManager;
use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::ui::components::{
    Auth, AuthCallback, Dashboard, History, Home, Logout, NavBar, PaymentForm, Profile, Settings,
    SwipeButton, WalletOverview,
};
use crate::ui::styles::global_styles;
use crate::utils::session::Session;

#[derive(Routable, Clone)]
pub enum Route {
    #[route("/")]
    Home,
    #[route("/auth")]
    Auth,
    #[route("/auth/callback")]
    AuthCallbackRoute,
    #[route("/dashboard")]
    DashboardRoute,
    #[route("/payment")]
    Payment,
    #[route("/history")]
    HistoryRoute,
    #[route("/settings")]
    SettingsRoute,
    #[route("/logout")]
    LogoutRoute,
    #[route("/profile")]
    ProfileRoute,
}

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {
            ContextProvider {
                value: {
                    let storage = Arc::new(ZipStorage::new().unwrap());
                    let rustbus = Arc::new(RustBusIntegrator::new().unwrap());
                    let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
                    let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
                    let auth = AuthManager::new(Arc::clone(&storage)).unwrap();
                    let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
                    let session = Session::new(Arc::clone(&storage)).unwrap();
                    (wallet, auth, paymail, tx_manager, rustbus, session)
                },
                div {
                    class: "app-container",
                    style: "{{{global_styles()}}} .app-container {{ display: flex; flex-direction: column; min-height: 100vh; }} .content {{ flex: 1; padding: 20px; }}",
                    NavBar {}
                    div { class: "content",
                        RouteRenderer {}
                    }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! { Home {} }
}

#[component]
fn Auth() -> Element {
    rsx! { Auth {} }
}

#[component]
fn AuthCallbackRoute() -> Element {
    rsx! { AuthCallback {} }
}

#[component]
fn DashboardRoute() -> Element {
    let session = use_context::<Session>();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(to_owned![session, user_id], || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    rsx! { WalletOverview {} }
}

#[component]
fn Payment() -> Element {
    let session = use_context::<Session>();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(to_owned![session, user_id], || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    rsx! {
        PaymentForm {}
        SwipeButton {
            recipient: "example@paymail.com",
            amount: 1000
        }
    }
}

#[component]
fn HistoryRoute() -> Element {
    let session = use_context::<Session>();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(to_owned![session, user_id], || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    rsx! { History {} }
}

#[component]
fn SettingsRoute() -> Element {
    let session = use_context::<Session>();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(to_owned![session, user_id], || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    rsx! { Settings {} }
}

#[component]
fn LogoutRoute() -> Element {
    let session = use_context::<Session>();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(to_owned![session, user_id], || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    rsx! { Logout {} }
}

#[component]
fn ProfileRoute() -> Element {
    let session = use_context::<Session>();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(to_owned![session, user_id], || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    rsx! { Profile {} }
}

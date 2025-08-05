use dioxus::prelude::*;
use dioxus_router::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::ui::components::{AuthCallback, AuthForm, Dashboard, History, NavBar, PaymentForm, Settings, SwipeButton, WalletOverview};
use crate::ui::styles::global_styles;
use crate::ui::transitions::{fade_in, slide_right};

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
}

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {
            ContextProvider {
                value: {
                    let storage = Arc::new(ZipStorage::new().unwrap());
                    let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
                    let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
                    let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
                    let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
                    let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();
                    let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
                    (wallet, oauth, passkey, paymail, tx_manager, rustbus)
                },
                div {
                    class: "app-container",
                    style: "{global_styles()} .app-container { display: flex; flex-direction: column; min-height: 100vh; } .content { flex: 1; padding: 20px; }",
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
    slide_right(rsx! {
        h1 { class: "title", "Zip Wallet" }
        Link { to: Route::Auth, class: "nav-link", "Sign Up / Login" }
        Link { to: Route::Payment, class: "nav-link", "Make a Payment" }
        Link { to: Route::History, class: "nav-link", "View History" }
        Link { to: Route::Settings, class: "nav-link", "Settings" }
    })
}

#[component]
fn Auth() -> Element {
    fade_in(rsx! { AuthForm {} })
}

#[component]
fn AuthCallbackRoute() -> Element {
    fade_in(rsx! { AuthCallback {} })
}

#[component]
fn DashboardRoute() -> Element {
    fade_in(rsx! { WalletOverview {} })
}

#[component]
fn Payment() -> Element {
    fade_in(rsx! {
        PaymentForm {}
        SwipeButton {
            recipient: "example@paymail.com",
            amount: 1000
        }
    })
}

#[component]
fn HistoryRoute() -> Element {
    fade_in(rsx! { History {} })
}

#[component]
fn SettingsRoute() -> Element {
    fade_in(rsx! { Settings {} })
}

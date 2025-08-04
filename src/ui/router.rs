use dioxus::prelude::*;
use dioxus_router::prelude::*;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::storage::ZipStorage;
use crate::ui::components::{AuthForm, Dashboard, History, PaymentForm, SwipeButton};
use crate::ui::transitions::{fade_in, slide_right};

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
    #[route("/history")]
    HistoryRoute,
}

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {
            ContextProvider {
                value: {
                    let storage = Arc::new(ZipStorage::new().unwrap());
                    let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), None));
                    let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager)).unwrap();
                    let oauth = OAuthManager::new(Arc::clone(&storage)).unwrap();
                    let passkey = PasskeyManager::new(Arc::clone(&storage)).unwrap();
                    let paymail = PaymailManager::new(PrivateKey::new());
                    (wallet, oauth, passkey, paymail, tx_manager)
                },
                div {
                    class: "app-container",
                    RouteRenderer {}
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    slide_right(rsx! {
        h1 { class: "title", "Zip Wallet" }
        Link { to: Route::Auth, "Sign Up / Login" }
        Link { to: Route::Payment, "Make a Payment" }
        Link { to: Route::History, "View History" }
    })
}

#[component]
fn Auth() -> Element {
    fade_in(rsx! { AuthForm {} })
}

#[component]
fn DashboardRoute() -> Element {
    let wallet = use_context::<WalletManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let balance = use_signal(|| 0u64);

    use_effect(move || async move {
        balance.set(wallet.update_balance(*user_id.read()).await.unwrap_or(0));
    });

    fade_in(rsx! { Dashboard {} })
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

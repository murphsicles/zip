use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::auth::auth::AuthManager;
use crate::blockchain::{
    paymail::PaymailManager,
    transaction::TransactionManager,
    wallet::WalletManager,
};
use crate::integrations::rustbus::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::ui::components::{
    auth::Auth,
    auth_callback::AuthCallback,
    dashboard::Dashboard,
    error::Error,
    history::History,
    home::Home,
    logout::Logout,
    nav::NavBar,
    payment_form::PaymentForm,
    profile::Profile,
    settings::Settings,
    swipe_button::SwipeButton,
};
use crate::ui::styles::global_styles;
use crate::utils::session::Session;
use std::sync::Arc;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Nav)]
    #[route("/")]
    Home,
    #[layout(Nav)]
    #[route("/auth")]
    Auth,
    #[layout(Nav)]
    #[route("/auth/callback")]
    AuthCallbackRoute,
    #[layout(Nav)]
    #[route("/dashboard")]
    DashboardRoute,
    #[layout(Nav)]
    #[route("/payment")]
    Payment,
    #[layout(Nav)]
    #[route("/history")]
    HistoryRoute,
    #[layout(Nav)]
    #[route("/settings")]
    SettingsRoute,
    #[layout(Nav)]
    #[route("/logout")]
    LogoutRoute,
    #[layout(Nav)]
    #[route("/profile")]
    ProfileRoute,
    #[route("/error/:error")]
    Error { error: String },
}

#[component]
fn Nav(cx: Scope) -> Element {
    cx.render(rsx! { NavBar {} })
}

pub fn AppRouter(cx: Scope) -> Element {
    let storage = Arc::new(ZipStorage::new().expect("Failed to initialize storage"));
    let rustbus = Arc::new(RustBusIntegrator::new().expect("Failed to initialize RustBus"));
    let tx_manager = Arc::new(TransactionManager::new(
        Arc::clone(&storage),
        Some(Arc::clone(&rustbus)),
    ));
    let wallet = Arc::new(
        WalletManager::new(
            Arc::clone(&storage),
            Arc::clone(&tx_manager),
            Some(Arc::clone(&rustbus)),
        )
        .expect("Failed to initialize wallet"),
    );
    let auth = AuthManager::new(Arc::clone(&storage)).expect("Failed to initialize auth");
    let paymail = PaymailManager::new(Arc::clone(&storage));
    let session = Session::new(Arc::clone(&storage)).expect("Failed to initialize session");

    use_effect(cx, || async move {
        // Authentication check
    });

    use_effect(cx, || async move {
        // User ID update
    });

    use_effect(cx, || async move {
        // Wallet balance update
    });

    use_effect(cx, || async move {
        // Payment form initialization
    });

    use_effect(cx, || async move {
        // Profile update
    });

    use_effect(cx, || async move {
        // Settings update
    });

    cx.render(rsx! {
        Router::<Route> {
            ContextProvider {
                value: storage,
                ContextProvider {
                    value: rustbus,
                    ContextProvider {
                        value: tx_manager,
                        ContextProvider {
                            value: wallet,
                            ContextProvider {
                                value: auth,
                                ContextProvider {
                                    value: paymail,
                                    ContextProvider {
                                        value: session,
                                        Outlet::<Route> {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        match route() {
            Route::Error { error } => rsx! { Error { error: ZipError::from(error) } },
            Route::AuthCallbackRoute => rsx! { AuthCallback {} },
            _ => None,
        }
    })
}

#[component]
fn Home(cx: Scope) -> Element {
    cx.render(rsx! { Home {} })
}

#[component]
fn Auth(cx: Scope) -> Element {
    cx.render(rsx! { Auth {} })
}

#[component]
fn AuthCallbackRoute(cx: Scope) -> Element {
    cx.render(rsx! { AuthCallback {} })
}

#[component]
fn DashboardRoute(cx: Scope) -> Element {
    let session = use_context::<Session>().unwrap();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(cx, || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    cx.render(rsx! { Dashboard {} })
}

#[component]
fn Payment(cx: Scope) -> Element {
    let session = use_context::<Session>().unwrap();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(cx, || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    cx.render(rsx! {
        PaymentForm {}
        SwipeButton {
            recipient: "example@paymail.com",
            amount: 1000,
            "Pay 1000 satoshis to example@paymail.com"
        }
    })
}

#[component]
fn HistoryRoute(cx: Scope) -> Element {
    let session = use_context::<Session>().unwrap();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(cx, || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    cx.render(rsx! { History {} })
}

#[component]
fn SettingsRoute(cx: Scope) -> Element {
    let session = use_context::<Session>().unwrap();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(cx, || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    cx.render(rsx! { Settings {} })
}

#[component]
fn LogoutRoute(cx: Scope) -> Element {
    let session = use_context::<Session>().unwrap();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(cx, || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    cx.render(rsx! { Logout {} })
}

#[component]
fn ProfileRoute(cx: Scope) -> Element {
    let session = use_context::<Session>().unwrap();
    let user_id = use_signal(|| Uuid::new_v4());

    use_effect(cx, || async move {
        if !session.is_authenticated(*user_id.read()).await {
            router().push(Route::Auth);
        }
    });

    cx.render(rsx! { Profile {} })
}

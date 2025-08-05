use dioxus::prelude::*;
use dioxus_motion::use_animated;
use dioxus_router::prelude::*;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::errors::ZipError;
use crate::ui::components::{ErrorDisplay, Loading, Notification};
use crate::ui::styles::global_styles;

#[component]
pub fn Logout() -> Element {
    let oauth = use_context::<OAuthManager>();
    let passkey = use_context::<PasskeyManager>();
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let is_loading = use_signal(|| false);
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));

    let on_logout = move |_| async move {
        is_loading.set(true);
        // Clear session data (placeholder for OAuth/Passkey cleanup)
        match oauth.clear_session().await {
            Ok(_) => {
                notification.set(Some("Logged out successfully".to_string()));
                use_router().push(Route::Home);
            }
            Err(e) => error.set(Some(e)),
        }
        is_loading.set(false);
    };

    rsx! {
        div {
            class: "logout",
            style: "{global_styles()} .logout { display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 10px; }",
            style: "{animated}",
            h2 { class: "title", "Logout" }
            button { onclick: on_logout, disabled: *is_loading.read(), "Confirm Logout" }
            ErrorDisplay { error: *error.read() }
            Notification { message: *notification.read(), is_success: true }
            if *is_loading.read() {
                Loading { message: "Logging out".to_string() }
            }
        }
    }
}

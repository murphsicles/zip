use dioxus::prelude::*;
use dioxus_motion::use_animated;
use uuid::Uuid;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::errors::ZipError;
use crate::ui::components::{ErrorDisplay, Loading, Notification};
use crate::ui::styles::global_styles;

#[component]
pub fn AuthForm() -> Element {
    let oauth = use_context::<OAuthManager>();
    let passkey = use_context::<PasskeyManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let totp_code = use_signal(|| String::new());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let is_loading = use_signal(|| false);
    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));

    let on_oauth_signup = move |_| async move {
        is_loading.set(true);
        let (url, _) = oauth.start_oauth_flow();
        // Open url in system browser or embedded view
        // Assume callback handled in router
        notification.set(Some("Redirecting to OAuth provider".to_string()));
        is_loading.set(false);
    };

    let on_passkey_login = move |_| async move {
        is_loading.set(true);
        match passkey.start_authentication(*user_id.read(), Some(&totp_code.read())).await {
            Ok((challenge, state)) => {
                // Prompt biometric and complete
                let cred = PublicKeyCredential::default(); // Placeholder
                match passkey.complete_authentication(cred, state) {
                    Ok(_) => {
                        notification.set(Some("Login successful".to_string()));
                        use_router().push(Route::DashboardRoute);
                    }
                    Err(e) => error.set(Some(e)),
                }
            }
            Err(e) => error.set(Some(e)),
        }
        is_loading.set(false);
    };

    rsx! {
        div {
            class: "auth-form",
            style: "{global_styles()} .auth-form { display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 10px; }",
            style: "{animated}",
            h2 { class: "title", "Sign Up / Login" }
            button { onclick: on_oauth_signup, disabled: *is_loading.read(), "Sign Up with OAuth" }
            input {
                r#type: "text",
                placeholder: "2FA Code (if enabled)",
                oninput: move |evt| totp_code.set(evt.value),
                disabled: *is_loading.read()
            }
            button { onclick: on_passkey_login, disabled: *is_loading.read(), "Login with Passkey" }
            ErrorDisplay { error: *error.read() }
            Notification { message: *notification.read(), is_success: true }
            if *is_loading.read() {
                Loading { message: "Processing authentication".to_string() }
            }
        }
    }
}

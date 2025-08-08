use dioxus::prelude::*;
use dioxus_router::prelude::*;
use uuid::Uuid;
use webauthn_rs::prelude::PublicKeyCredential;

use crate::auth::AuthManager;
use crate::errors::ZipError;
use crate::ui::router::Route;

#[component]
pub fn Auth() -> Element {
    let auth = use_context::<AuthManager>();
    let user_id = use_signal(|| Uuid::new_v4());
    let email = use_signal(|| String::new());
    let totp_code = use_signal(|| String::new());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let is_loading = use_signal(|| false);

    let on_oauth_signup = move |_| async move {
        is_loading.set(true);
        match auth.start_oauth(&user_id.read().to_string()).await {
            Ok((url, _)) => {
                notification.set(Some("Redirecting to OAuth provider".to_string()));
            }
            Err(e) => error.set(Some(e)),
        }
        is_loading.set(false);
    };

    let on_passkey_login = move |_| async move {
        is_loading.set(true);
        match auth.start_passkey_authentication(*user_id.read(), Some(&totp_code.read())).await {
            Ok((challenge, state)) => {
                let cred = PublicKeyCredential::default(); // Placeholder
                match auth.complete_passkey_authentication(*user_id.read(), cred, state).await {
                    Ok(_) => {
                        notification.set(Some("Login successful".to_string()));
                        router().push(Route::DashboardRoute);
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
            class: "auth",
            h2 { class: "title", "Sign Up / Login" }
            button { onclick: on_oauth_signup, disabled: *is_loading.read(), "Sign Up with OAuth" }
            input {
                r#type: "text",
                placeholder: "Email",
                oninput: move |evt| email.set(evt.value()),
                disabled: *is_loading.read()
            }
            input {
                r#type: "text",
                placeholder: "2FA Code (if enabled)",
                oninput: move |evt| totp_code.set(evt.value()),
                disabled: *is_loading.read()
            }
            button { onclick: on_passkey_login, disabled: *is_loading.read(), "Login with Passkey" }
            if let Some(err) = error.read().as_ref() {
                div { class: "error", "{err}" }
            }
            if let Some(msg) = notification.read().as_ref() {
                div { class: "notification", "{msg}" }
            }
            if *is_loading.read() {
                div { class: "loading", "Processing authentication" }
            }
        }
    }
}

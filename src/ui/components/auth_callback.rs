use dioxus::prelude::*;
use dioxus_router::prelude::*;
use oauth2::PkceCodeVerifier;
use uuid::Uuid;

use crate::auth::AuthManager;
use crate::errors::ZipError;
use crate::ui::components::{ErrorDisplay, Loading, Notification};
use crate::ui::router::Route;
use crate::ui::styles::global_styles;
use crate::ui::transitions::fade_in;

#[component]
pub fn AuthCallback() -> Element {
    let auth = use_context::<AuthManager>();
    let code = use_signal(|| String::new());
    let pkce_verifier = use_signal(|| PkceCodeVerifier::new(String::new()));
    let csrf_token = use_signal(|| String::new());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let is_loading = use_signal(|| true);

    use_effect(
        to_owned![
            auth,
            code,
            pkce_verifier,
            csrf_token,
            error,
            notification,
            is_loading
        ],
        || async move {
            // Placeholder: Extract code, verifier, csrf from URL query (not supported by dioxus_router)
            // TODO: Implement query parameter parsing (e.g., via window.location.search or external library)
            let code_param = String::new();
            let verifier = PkceCodeVerifier::new(String::new());
            let csrf = String::new();

            code.set(code_param.clone());
            pkce_verifier.set(verifier.clone());
            csrf_token.set(csrf.clone());

            match auth
                .complete_oauth(&Uuid::new_v4().to_string(), code_param, verifier, csrf)
                .await
            {
                Ok(user_id) => {
                    notification.set(Some("Authenticated successfully".to_string()));
                    router().push(Route::DashboardRoute);
                }
                Err(e) => error.set(Some(e)),
            }
            is_loading.set(false);
        },
    );

    fade_in(
        cx,
        rsx! {
            div {
                class: "auth-callback",
                style: "{{{global_styles()}}} .auth-callback {{ display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 10px; }}",
                h2 { class: "title", "Authenticating..." }
                if *is_loading.read() {
                    Loading { message: "Completing authentication".to_string() }
                }
                ErrorDisplay { error: *error.read() }
                Notification { message: *notification.read(), is_success: true }
            }
        },
    )
}

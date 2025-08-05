use dioxus::prelude::*;
use dioxus_router::prelude::*;
use uuid::Uuid;

use crate::auth::OAuthManager;
use crate::errors::ZipError;
use crate::ui::components::{ErrorDisplay, Loading, Notification};
use crate::ui::styles::global_styles;
use crate::ui::transitions::fade_in;

#[component]
pub fn AuthCallback() -> Element {
    let oauth = use_context::<OAuthManager>();
    let code = use_signal(|| String::new());
    let pkce_verifier = use_signal(|| PkceCodeVerifier::new(String::new()));
    let csrf_token = use_signal(|| String::new());
    let error = use_signal(|| None::<ZipError>);
    let notification = use_signal(|| None::<String>);
    let is_loading = use_signal(|| true);

    use_effect(move || async move {
        // Extract code, verifier, csrf from URL query (Dioxus router params)
        let params = use_router().query_params();
        let code_param = params.get("code").unwrap_or("").to_string();
        let verifier = PkceCodeVerifier::new(params.get("verifier").unwrap_or("").to_string());
        let csrf = params.get("csrf").unwrap_or("").to_string();

        code.set(code_param.clone());
        pkce_verifier.set(verifier.clone());
        csrf_token.set(csrf.clone());

        match oauth.complete_oauth_flow(code_param, verifier, csrf).await {
            Ok((user_id, email)) => {
                notification.set(Some(format!("Authenticated as {}", email)));
                use_router().push(Route::DashboardRoute);
            }
            Err(e) => error.set(Some(e)),
        }
        is_loading.set(false);
    });

    fade_in(rsx! {
        div {
            class: "auth-callback",
            style: "{global_styles()} .auth-callback { display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 10px; }",
            h2 { class: "title", "Authenticating..." }
            if *is_loading.read() {
                Loading { message: "Completing authentication".to_string() }
            }
            ErrorDisplay { error: *error.read() }
            Notification { message: *notification.read(), is_success: true }
        }
    })
}

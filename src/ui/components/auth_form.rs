use dioxus::prelude::*;
use dioxus_motion::use_animated;

use crate::auth::{OAuthManager, PasskeyManager};
use crate::errors::ZipError;

#[component]
pub fn AuthForm(cx: Scope) -> Element {
    let oauth = use_context::<OAuthManager>(cx)?;
    let passkey = use_context::<PasskeyManager>(cx)?;

    let animated = use_animated(|style| style.opacity(1.0).duration(0.5));

    let on_oauth = move |_| {
        let (url, csrf) = oauth.start_oauth_flow();
        // Open system browser or embedded view to url
        // Handle callback asynchronously
    };

    let on_passkey = move |_| {
        let user_id = Uuid::new_v4();  // From state
        let (challenge, state) = passkey.start_registration(user_id, "user")?;
        // Prompt biometric and complete
    };

    rsx! {
        div {
            style: "{animated}",
            button { onclick: on_oauth, "Sign Up with OAuth" }
            button { onclick: on_passkey, "Login with Passkey" }
        }
    }
}

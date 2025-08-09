use bincode;
use dioxus::prelude::*;
use dioxus_router::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use sv::private_key::PrivateKey;
use uuid::Uuid;

use crate::auth::AuthManager;
use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::config::EnvConfig;
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::ui::components::{NavBar, ThemeProvider};
use crate::ui::router::Route;
use crate::ui::styles::global_styles;
use crate::ui::theme::Theme;

#[component]
pub fn App() -> Element {
    let config = EnvConfig::load().unwrap_or_default();
    let storage = Arc::new(ZipStorage::new().unwrap_or_default());
    let rustbus = Arc::new(RustBusIntegrator::new().unwrap_or_default());
    let tx_manager = Arc::new(TransactionManager::new(
        Arc::clone(&storage),
        Some(Arc::clone(&rustbus)),
    ));
    let wallet = WalletManager::new(
        Arc::clone(&storage),
        Arc::clone(&tx_manager),
        Some(Arc::clone(&rustbus)),
    )
    .unwrap_or_default();
    let auth = AuthManager::new(Arc::clone(&storage)).unwrap_or_default();
    let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
    let selected_theme = use_signal(|| Theme::Light);

    use_effect(to_owned![storage, selected_theme], || async move {
        // Load theme from storage
        if let Ok(Some(data)) = storage.get_user_data(Uuid::new_v4()) {
            let prefs: HashMap<String, String> = bincode::deserialize(&data).unwrap_or_default();
            selected_theme.set(
                prefs
                    .get("theme")
                    .map(|t| {
                        if t == "dark" {
                            Theme::Dark
                        } else {
                            Theme::Light
                        }
                    })
                    .unwrap_or(Theme::Light),
            );
        }
    });

    rsx! {
        ThemeProvider { theme: *selected_theme.read(),
            div {
                class: "app-container",
                style: "{{{global_styles()}}}",
                ContextProvider {
                    value: (wallet, auth, paymail, tx_manager, rustbus),
                    NavBar {}
                    div { class: "content",
                        Router::<Route> {}
                    }
                }
            }
        }
    }
}

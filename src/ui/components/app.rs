use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::auth::{AuthManager, SessionManager};
use crate::blockchain::{PaymailManager, TransactionManager, WalletManager};
use crate::integrations::RustBusIntegrator;
use crate::storage::ZipStorage;
use crate::ui::components::{NavBar, ThemeProvider};
use crate::ui::styles::global_styles;

#[component]
pub fn App() -> Element {
    let storage = Arc::new(ZipStorage::new().unwrap());
    let rustbus = Arc::new(RustBusIntegrator::new("http://localhost:8080").unwrap());
    let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&storage), Some(Arc::clone(&rustbus))));
    let wallet = WalletManager::new(Arc::clone(&storage), Arc::clone(&tx_manager), Some(Arc::clone(&rustbus))).unwrap();
    let auth = AuthManager::new(Arc::clone(&storage)).unwrap();
    let paymail = PaymailManager::new(PrivateKey::new(), Arc::clone(&storage));
    let session = SessionManager::new(Arc::clone(&storage));

    rsx! {
        ThemeProvider {
            theme: Theme::Light, // Default theme, overridden by settings
            div {
                class: "app-container",
                style: "{global_styles()} .app-container { display: flex; flex-direction: column; min-height: 100vh; } .content { flex: 1; padding: 20px; }",
                ContextProvider {
                    value: (wallet, auth, paymail, tx_manager, rustbus, session),
                    NavBar {}
                    div { class: "content",
                        Router::<Route> {}
                    }
                }
            }
        }
    }
}

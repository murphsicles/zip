use dioxus::prelude::*;

use crate::ui::styles::global_styles;

#[derive(Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Props, PartialEq)]
pub struct ThemeProps {
    #[props(default = Theme::Light)]
    theme: Theme,
    #[props(optional)]
    children: Option<Element>,
}

#[component]
pub fn ThemeProvider(props: ThemeProps) -> Element {
    let theme_styles = match props.theme {
        Theme::Light => {
            r#"
            body { background-color: #fff; color: #333; }
            .navbar { background-color: #4caf50; }
            .nav-link { color: white; }
            .nav-link:hover { background-color: #388e3c; }
            .auth, .auth-callback, .home, .logout, .payment-form, .profile, .settings-page { background-color: #f0f0f0; }
            .balance-main, .title { color: #333; }
            .balance-sub, .info { color: #666; }
            .history-grid .header { background-color: #e0e0e0; }
            .section { border-color: #ddd; }
            .error { background-color: #ffe6e6; color: #d32f2f; border-color: #f44336; }
            .notification.success { background-color: #e6ffed; color: #2e7d32; border-color: #4caf50; }
            .loading { background-color: #f0f0f0; color: #666; }
        "#
        }
        Theme::Dark => {
            r#"
            body { background-color: #1a1a1a; color: #ccc; }
            .navbar { background-color: #2e7d32; }
            .nav-link { color: #e0e0e0; }
            .nav-link:hover { background-color: #1b5e20; }
            .auth, .auth-callback, .home, .logout, .payment-form, .profile, .settings-page { background-color: #2c2c2c; }
            .balance-main, .title { color: #e0e0e0; }
            .balance-sub, .info { color: #999; }
            .history-grid .header { background-color: #3c3c3c; }
            .section { border-color: #555; }
            .error { background-color: #4a1c1c; color: #ff6655; border-color: #b71c1c; }
            .notification.success { background-color: #1c4a1c; color: #4caf50; border-color: #2e7d32; }
            .loading { background-color: #3c3c3c; color: #999; }
        "#
        }
    };

    rsx! {
        div {
            style: "{{{global_styles()}}} {theme_styles}",
            if let Some(children) = props.children {
                {children}
            }
        }
    }
}

use dioxus::prelude::*;

use crate::ui::styles::global_styles;
use crate::ui::theme::Theme;

#[derive(Props, PartialEq)]
pub struct ThemeSwitcherProps {
    #[props(default = Theme::Light)]
    current_theme: Theme,
    on_theme_change: EventHandler<Theme>,
}

#[component]
pub fn ThemeSwitcher(props: ThemeSwitcherProps) -> Element {
    let on_theme_change = move |evt: Event<FormData>| {
        let new_theme = match evt.value().as_str() {
            "dark" => Theme::Dark,
            _ => Theme::Light,
        };
        props.on_theme_change.call(new_theme);
    };

    rsx! {
        div {
            class: "theme-switcher",
            style: "{{{global_styles()}}} .theme-switcher {{ display: flex; align-items: center; gap: 10px; padding: 10px; }}",
            label { "Theme: " }
            select { onchange: on_theme_change,
                option { value: "light", selected: props.current_theme == Theme::Light, "Light" }
                option { value: "dark", selected: props.current_theme == Theme::Dark, "Dark" }
            }
        }
    }
}

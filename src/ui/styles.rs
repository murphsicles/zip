use dioxus::prelude::*;

pub fn global_styles() -> &'static str {
    r#"
        body { font-family: Arial, sans-serif; background-color: #fff; color: #333; margin: 0; padding: 0; }
        .app-container { min-height: 100vh; display: flex; flex-direction: column; }
        .auth { display: flex; flex-direction: column; align-items: center; padding: 20px; gap: 10px; }
        .dashboard { padding: 20px; background-color: #f0f0f0; border-radius: 8px; text-align: center; }
        .balance-main { font-size: 2.5em; font-weight: bold; color: #333; }
        .balance-sub { font-size: 1.2em; color: #666; }
        .payment-form { display: flex; flex-direction: column; gap: 10px; padding: 20px; }
        .swipe-button { width: 200px; height: 50px; background-color: #4caf50; color: white; text-align: center; line-height: 50px; transition: transform 0.3s ease; cursor: pointer; }
        .history-grid { display: grid; grid-template-columns: 100px 120px 140px 200px 140px 200px; gap: 10px; overflow-y: auto; max-height: 80vh; font-size: 14px; padding: 10px; border: 1px solid #ddd; }
        .header { font-weight: bold; background-color: #f0f0f0; padding: 8px; }
        .delta-positive { color: green; }
        .delta-negative { color: red; }
        .txid-link { color: #007bff; text-decoration: none; display: flex; align-items: center; gap: 4px; }
        .txid-link:hover { text-decoration: underline; }
        .settings-page { display: flex; flex-direction: column; gap: 20px; padding: 20px; }
        .section { border: 1px solid #ddd; padding: 15px; border-radius: 8px; }
        .paymail-list { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }
        .navbar { display: flex; justify-content: space-around; padding: 10px; background-color: #4caf50; color: white; }
        .nav-link { color: white; text-decoration: none; padding: 8px 16px; border-radius: 4px; }
        .nav-link:hover { background-color: #388e3c; }
        select, input, button { padding: 8px; border: 1px solid #ddd; border-radius: 4px; }
        toggle { display: inline-block; width: 40px; height: 20px; background-color: #ccc; border-radius: 10px; position: relative; cursor: pointer; }
        toggle::after { content: ''; position: absolute; width: 18px; height: 18px; background-color: white; border-radius: 50%; top: 1px; left: 1px; transition: transform 0.2s; }
        toggle:checked::after { transform: translateX(20px); }
        toggle:checked { background-color: #4caf50; }
        @media (max-width: 600px) {
            .swipe-button { width: 100%; }
            .history-grid { grid-template-columns: 1fr; font-size: 12px; }
            .dashboard { padding: 10px; }
            .balance-main { font-size: 2em; }
            .paymail-list { grid-template-columns: 1fr; }
            .navbar { flex-direction: column; gap: 10px; }
        }
    "#
}

#[component]
pub fn Styles() -> Element {
    rsx! {
        style { "{{{global_styles()}}}" }
    }
}

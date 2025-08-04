use dioxus::prelude::*;
use reqwest::Client;
use rustbus::Client as RustBusClient;
use serde_json::Value;
use std::collections::HashMap;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::blockchain::WalletManager;
use crate::integrations::RustBusIntegrator;
use crate::ui::styles::global_styles;

#[derive(Clone, Debug)]
struct Tx {
    token: String,
    amount_usd: f64,
    current_value_usd: f64,
    delta_percent: f64,
    txid: String,
    timestamp: String,
    from_to: String,
}

#[component]
pub fn History() -> Element {
    let wallet = use_context::<WalletManager>();
    let rustbus = use_context::<RustBusIntegrator>();
    let user_id = use_signal(|| Uuid::new_v4());
    let txs = use_signal(|| vec![]);
    let page = use_signal(|| 0);
    let loading = use_signal(|| false);
    let current_price = use_signal(|| 0.0);
    let historical_prices = use_signal(|| HashMap::new());

    use_effect(move || async move {
        let client = Client::new();
        let resp = client
            .get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin-sv&vs_currencies=usd")
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        let price = resp["bitcoin-sv"]["usd"].as_f64().unwrap_or(0.0);
        current_price.set(price);
    });

    use_effect(move || async move {
        if *loading.read() {
            return;
        }
        loading.set(true);
        let new_txs = rustbus.query_tx_history(*user_id.read()).await.unwrap_or(vec![]); // Paginate with page * 20
        let mut updated_txs = txs.read().clone();
        for txid in new_txs {
            // Fetch tx details from RustBus or WhatsOnChain
            let tx_details = fetch_tx_details(&txid).await;
            let timestamp = tx_details["time"].as_str().unwrap_or("");
            let dt = OffsetDateTime::parse(timestamp, &Rfc3339).unwrap_or_else(|_| OffsetDateTime::now_utc());
            let date_str = dt.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap();

            let hist_price = if let Some(price) = historical_prices.read().get(&date_str) {
                *price
            } else {
                let client = Client::new();
                let resp = client
                    .get(format!("https://api.coingecko.com/api/v3/coins/bitcoin-sv/history?date={}", date_str))
                    .send()
                    .await
                    .unwrap()
                    .json::<Value>()
                    .await
                    .unwrap();
                let price = resp["market_data"]["current_price"]["usd"].as_f64().unwrap_or(0.0);
                historical_prices.write().insert(date_str, price);
                price
            };

            let tx_amount = tx_details["amount"].as_u64().unwrap_or(0) as f64 / 100_000_000.0;
            let amount_usd = tx_amount * hist_price;
            let current_value_usd = tx_amount * *current_price.read();
            let delta_percent = ((current_value_usd - amount_usd) / amount_usd) * 100.0;

            updated_txs.push(Tx {
                token: "BSV".to_string(),
                amount_usd,
                current_value_usd,
                delta_percent,
                txid,
                timestamp: dt.format(&time::format_description::parse("[year]/[month]/[day]:[hour]:[minute]").unwrap()).unwrap(),
                from_to: tx_details["from"].as_str().unwrap_or("Unknown").to_string(),
            });
        }
        txs.set(updated_txs);
        page.set(*page.read() + 1);
        loading.set(false);
    });

    let on_scroll = move |evt: Event<ScrollData>| {
        if evt.scroll_height - evt.scroll_top - evt.client_height < 50 && !*loading.read() {
            spawn(async move { /* Load next page */ });
        }
    };

    rsx! {
        div {
            class: "history-grid",
            style: "{global_styles()} .history-grid { display: grid; grid-template-columns: repeat(6, 1fr); gap: 10px; overflow-y: auto; max-height: 80vh; } .delta-positive { color: green; } .delta-negative { color: red; }",
            onscroll: on_scroll,
            div { "Token" }
            div { "Amount (USD at Tx)" }
            div { "Value (Current USD)" }
            div { "TXID" }
            div { "Timestamp" }
            div { "From/To" }
            for tx in txs.read().iter() {
                div { tx.token.clone() }
                div { "{tx.amount_usd:.2}" }
                div { class: if tx.delta_percent > 0.0 { "delta-positive" } else { "delta-negative" }, "{tx.current_value_usd:.2} ({tx.delta_percent:.2}%)" }
                div { a { href: "https://whatsonchain.com/tx/{tx.txid}", target: "_blank", "{tx.txid}" } }
                div { "{tx.timestamp}" }
                div { "{tx.from_to}" }
            }
        }
    }
}

async fn fetch_tx_details(txid: &str) -> Value {
    let client = Client::new();
    let resp = client
        .get(format!("https://api.whatsonchain.com/v1/bsv/main/tx/{txid}"))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    resp
}

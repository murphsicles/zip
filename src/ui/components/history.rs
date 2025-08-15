use dioxus::prelude::*;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde_json::Value;
use std::collections::HashMap;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::blockchain::WalletManager;
use crate::integrations::rustbus::RustBusIntegrator;
use crate::ui::styles::global_styles;
use crate::ui::transitions::fade_in;

#[derive(Clone, Debug)]
struct Tx {
    token: String,
    amount_usd: Decimal,
    current_value_usd: Decimal,
    delta_percent: Decimal,
    txid: String,
    timestamp: String,
    from_to: String,
}

#[component]
pub fn History(cx: Scope) -> Element {
    let wallet = use_context::<WalletManager>();
    let rustbus = use_context::<RustBusIntegrator>();
    let user_id = use_signal(|| Uuid::new_v4());
    let currency = use_signal(|| "USD".to_string());
    let txs = use_signal(|| vec![]);
    let page = use_signal(|| 0);
    let loading = use_signal(|| false);
    let current_price = use_signal(|| Decimal::ZERO);
    let historical_prices = use_signal(|| HashMap::new());

    use_effect(move || {
        async move {
            let client = Client::new();
            let resp = client
                .get(format!(
                    "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin-sv&vs_currencies={}",
                    currency.read().to_lowercase()
                ))
                .send()
                .await;
            let price = match resp {
                Ok(resp) => match resp.json::<Value>().await {
                    Ok(json) => json["bitcoin-sv"][currency.read().to_lowercase()]
                        .as_f64()
                        .map(|p| Decimal::try_from_f64(p).unwrap_or_default())
                        .unwrap_or_default(),
                    Err(_) => Decimal::ZERO,
                },
                Err(_) => Decimal::ZERO,
            };
            current_price.set(price);
        }
    });

    use_effect(move || {
        async move {
            if *loading.read() {
                return;
            }
            loading.set(true);
            let new_txs = match rustbus.query_tx_history(*user_id.read()).await {
                Ok(txs) => txs,
                Err(_) => vec![],
            };
            let mut updated_txs = txs.read().clone();
            for txid in new_txs.into_iter().skip(*page.read() * 20).take(20) {
                let tx_details = match fetch_tx_details(&txid).await {
                    Ok(details) => details,
                    Err(_) => Value::default(),
                };
                let timestamp = tx_details["time"].as_str().unwrap_or("");
                let dt = OffsetDateTime::parse(timestamp, &Rfc3339)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc());
                let date_str = dt
                    .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                    .unwrap_or_default();
                let hist_price = if let Some(price) = historical_prices.read().get(&date_str) {
                    *price
                } else {
                    let client = Client::new();
                    let resp = client
                        .get(format!(
                            "https://api.coingecko.com/api/v3/coins/bitcoin-sv/history?date={}",
                            date_str
                        ))
                        .send()
                        .await;
                    let price = match resp {
                        Ok(resp) => match resp.json::<Value>().await {
                            Ok(json) => json["market_data"]["current_price"]
                                [currency.read().to_lowercase()]
                                .as_f64()
                                .map(|p| Decimal::try_from_f64(p).unwrap_or_default())
                                .unwrap_or_default(),
                            Err(_) => Decimal::ZERO,
                        },
                        Err(_) => Decimal::ZERO,
                    };
                    historical_prices.write().insert(date_str.clone(), price);
                    price
                };
                let tx_amount = tx_details["amount"].as_u64().unwrap_or(0) as f64 / 100_000_000.0;
                let amount_usd = Decimal::try_from_f64(tx_amount).unwrap_or_default() * hist_price;
                let current_value_usd =
                    Decimal::try_from_f64(tx_amount).unwrap_or_default() * *current_price.read();
                let delta_percent = if amount_usd != Decimal::ZERO {
                    ((current_value_usd - amount_usd) / amount_usd) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };
                updated_txs.push(Tx {
                    token: "BSV".to_string(), // Placeholder for token support
                    amount_usd,
                    current_value_usd,
                    delta_percent,
                    txid,
                    timestamp: dt
                        .format(
                            &time::format_description::parse("[year]/[month]/[day]:[hour]:[minute]").unwrap(),
                        )
                        .unwrap_or_default(),
                    from_to: tx_details["from"].as_str().unwrap_or("Unknown").to_string(),
                });
            }
            txs.set(updated_txs);
            page.set(*page.read() + 1);
            loading.set(false);
        }
    });

    fade_in(
        cx,
        rsx! {
            div {
                class: "history-grid",
                style: "{{{global_styles()}}} .history-grid {{ display: grid; grid-template-columns: 100px 120px 140px 200px 140px 200px; gap: 10px; overflow-y: auto; max-height: 80vh; font-size: 14px; padding: 10px; }} .history-grid > div {{ padding: 8px; border-bottom: 1px solid #ddd; }} .header {{ font-weight: bold; background-color: #f0f0f0; }} .delta-positive {{ color: green; }} .delta-negative {{ color: red; }} .txid-link {{ color: #007bff; text-decoration: none; }} .txid-link:hover {{ text-decoration: underline; }} @media (max-width: 600px) {{ .history-grid {{ grid-template-columns: 1fr; }} .history-grid > div {{ font-size: 12px; }} }}",
                div { class: "header", "Token" }
                div { class: "header", "Amount ({currency})" }
                div { class: "header", "Value ({currency})" }
                div { class: "header", "TXID" }
                div { class: "header", "Timestamp" }
                div { class: "header", "From/To" }
                for tx in txs.read().iter() {
                    div { "{tx.token}" }
                    div { "{tx.amount_usd:.2}" }
                    div { class: if tx.delta_percent > Decimal::ZERO { "delta-positive" } else { "delta-negative" }, "{tx.current_value_usd:.2} ({tx.delta_percent:.2}%)" }
                    div { a { class: "txid-link", href: "https://whatsonchain.com/tx/{tx.txid}", target: "_blank", "🔗 {tx.txid}" } }
                    div { "{tx.timestamp}" }
                    div { "{tx.from_to}" }
                }
                if *loading.read() {
                    div { style: "grid-column: span 6; text-align: center;", "Loading..." }
                }
            }
        },
    )
}

async fn fetch_tx_details(txid: &str) -> Result<Value, reqwest::Error> {
    let client = Client::new();
    client
        .get(format!("https://api.whatsonchain.com/v1/bsv/main/tx/{}", txid))
        .send()
        .await?
        .json::<Value>()
        .await
}

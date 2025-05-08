use eventsource_client::Client as EventSourceClient; // 避免與 reqwest::Client 衝突
use eventsource_client::{ClientBuilder, SSE};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};
use futures::StreamExt;
use serde_json::Value;
use std::error::Error;
use std::fs;

const BASE_URL: &str = "https://hermes.pyth.network";

type SharedPriceMap = Arc<Mutex<Vec<(String, f64)>>>;

/// 訂閱 Pyth 即時價格串流，並將價格回傳給 callback 函數。
///
/// # 參數
/// - `id`: Pyth price feed 的 ID（hex 字串）
/// - `on_price`: 回呼函數，接收實際價格（`f64`）
///
/// # 範例
/// pyth_stream::subscribe_price_stream("0xe62d...", |price| println!("價格: {}", price)).await;
pub async fn get_price_stream_from_pyth<F>(id: &str, mut on_price: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut(f64) + Send + 'static,
{
    let url = format!("{}/v2/updates/price/stream?ids[]={}", BASE_URL, id);

    let mut stream = ClientBuilder::for_url(&url)?.build().stream();

    while let Some(event) = stream.next().await {
        match event {
            Ok(SSE::Event(ev)) => {
                if let Ok(json) = serde_json::from_str::<Value>(&ev.data) {
                    if let Some(parsed_array) = json.get("parsed").and_then(|v| v.as_array()) {
                        for entry in parsed_array {
                            if let Some(price_obj) = entry.get("price") {
                                if let (Some(price_str), Some(expo)) = (
                                    price_obj.get("price").and_then(|p| p.as_str()),
                                    price_obj.get("expo").and_then(|e| e.as_i64()),
                                ) {
                                    if let Ok(price_int) = price_str.parse::<f64>() {
                                        let actual_price = price_int * 10f64.powi(expo as i32);
                                        on_price(actual_price);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(_) => {} // 略過 Ping/Comment
            Err(e) => {
                eprintln!("SSE 錯誤: {}，3 秒後嘗試重連", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                break;
            }
        }
    }

    Ok(())
}

pub async fn get_pyth_feed_id(symbol: &str, category: &str) -> String {
    let target = symbol.to_uppercase();
    let data = fs::read_to_string("data/id.toml").expect("無法讀取 Pyth 配置檔案");
    let pairs: toml::Value = toml::from_str(&data).expect("無法解析 Pyth 配置檔案");
    let feeds = pairs.get(category).expect("無法找到 feeds");
    let feed_id = feeds
        .get(&target)
        .unwrap_or_else(|| panic!("無法找到 feed_id, symbol = {}", symbol));
    let raw = feed_id.as_str().expect("feed_id 應為字串");
    return raw.to_string();
}

pub fn spawn_price_stream(
    symbol: &str,
    category: &str,
    prices: SharedPriceMap,
    tx: watch::Sender<Vec<(String, f64)>>,
) {
    let symbol = symbol.to_string();
    let category = category.to_string();
    tokio::spawn(async move {
        let id = get_pyth_feed_id(&symbol, &category).await;
        let symbol_clone = symbol.clone();
        if let Err(e) = get_price_stream_from_pyth(id.as_str(), move |price| {
            let prices = Arc::clone(&prices);
            let symbol_clone = symbol_clone.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                update_price(&symbol_clone, price, &prices, &tx).await;
            });
        })
        .await
        {
            eprintln!("Error occurred for {}: {}", symbol, e);
        }
    });
}

async fn update_price(
    symbol: &str,
    price: f64,
    prices: &Arc<Mutex<Vec<(String, f64)>>>,
    tx: &watch::Sender<Vec<(String, f64)>>,
) {
    let mut prices = prices.lock().await;
    if let Some(entry) = prices.iter_mut().find(|(s, _)| s == symbol) {
        entry.1 = price;
    } else {
        prices.push((symbol.to_string(), price));
    }
    // 通知 watch 接收者價格已更新
    let _ = tx.send(prices.clone());
}

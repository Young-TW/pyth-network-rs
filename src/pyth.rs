use eventsource_client::Client as EventSourceClient; // 避免與 reqwest::Client 衝突
use eventsource_client::{ClientBuilder, SSE};
use futures::StreamExt;
use serde_json::Value;
use std::error::Error;
use std::fs;

const BASE_URL: &str = "https://hermes.pyth.network";

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
                eprintln!("SSE 錯誤: {}", e);
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

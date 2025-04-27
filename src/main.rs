use pyth::{get_price_stream_from_pyth, get_pyth_feed_id};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

mod pyth;

#[tokio::main]
async fn main() {
    let prices = Arc::new(Mutex::new(vec![]));

    // 啟動價格流
    spawn_price_stream("BTC", "Crypto", Arc::clone(&prices));
    spawn_price_stream("ETH", "Crypto", Arc::clone(&prices));
    spawn_price_stream("SOL", "Crypto", Arc::clone(&prices));

    // 持續印出最新價格
    loop {
        {
            let prices = prices.lock().await;
            println!("\x1B[2J\x1B[1;1H"); // 清除終端畫面
            for (symbol, price) in prices.iter() {
                println!("{}: {:.2}", symbol, price);
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

fn spawn_price_stream(symbol: &str, category: &str, prices: Arc<Mutex<Vec<(String, f64)>>>) {
    let symbol = symbol.to_string();
    let category = category.to_string();
    tokio::spawn(async move {
        let id = get_pyth_feed_id(&symbol, &category).await;
        let symbol_clone = symbol.clone();
        if let Err(e) = get_price_stream_from_pyth(id.as_str(), move |price| {
            update_price(&symbol_clone, price, &prices)
        })
        .await
        {
            eprintln!("Error occurred for {}: {}", symbol, e);
        }
    });
}

fn update_price(symbol: &str, price: f64, prices: &Arc<Mutex<Vec<(String, f64)>>>) {
    let symbol = symbol.to_string(); // Clone symbol to ensure it is owned
    let prices = Arc::clone(prices); // Clone Arc to ensure it is owned
    tokio::spawn(async move {
        let mut prices = prices.lock().await;
        if let Some(entry) = prices.iter_mut().find(|(s, _)| s == &symbol) {
            entry.1 = price;
        } else {
            prices.push((symbol, price));
        }
    });
}
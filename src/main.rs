use pyth::{get_price_stream_from_pyth, get_pyth_feed_id};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

mod pyth;

#[tokio::main]
async fn main() {
    let prices = Arc::new(Mutex::new(vec![]));

    // 啟動 BTC 價格流
    let btc_prices = Arc::clone(&prices);
    tokio::spawn(async move {
        let id = get_pyth_feed_id("BTC", "Crypto").await;
        if let Err(e) = get_price_stream_from_pyth(id.as_str(), move |price| {
            update_price("BTC", price, &btc_prices)
        })
        .await
        {
            eprintln!("Error occurred for BTC: {}", e);
        }
    });

    // 啟動 ETH 價格流
    let eth_prices = Arc::clone(&prices);
    tokio::spawn(async move {
        let id = get_pyth_feed_id("ETH", "Crypto").await;
        if let Err(e) = get_price_stream_from_pyth(id.as_str(), move |price| {
            update_price("ETH", price, &eth_prices)
        })
        .await
        {
            eprintln!("Error occurred for ETH: {}", e);
        }
    });

    // 啟動 SOL 價格流
    let sol_prices = Arc::clone(&prices);
    tokio::spawn(async move {
        let id = get_pyth_feed_id("SOL", "Crypto").await;
        if let Err(e) = get_price_stream_from_pyth(id.as_str(), move |price| {
            update_price("SOL", price, &sol_prices)
        })
        .await
        {
            eprintln!("Error occurred for SOL: {}", e);
        }
    });

    // 持續印出最新價格
    loop {
        {
            let prices = prices.lock().await;
            println!("\x1B[2J\x1B[1;1H"); // 清除終端畫面
            for (symbol, price) in prices.iter() {
                println!("{}: {:.2}", symbol, price);
            }
        }
        sleep(Duration::from_millis(1)).await;
    }
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
use pyth::spawn_price_stream;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

mod pyth;

#[tokio::main]
async fn main() {
    let prices = Arc::new(Mutex::new(vec![]));

    // 啟動價格流
    spawn_price_stream("BTC", "Crypto", Arc::clone(&prices));
    spawn_price_stream("ETH", "Crypto", Arc::clone(&prices));
    spawn_price_stream("SOL", "Crypto", Arc::clone(&prices));
    spawn_price_stream("TSLA", "Stock", Arc::clone(&prices));

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

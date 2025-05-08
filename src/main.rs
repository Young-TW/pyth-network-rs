use pyth::spawn_price_stream;
use tokio::sync::{watch, Mutex};
use std::sync::Arc;

mod pyth;

#[tokio::main]
async fn main() {
    let prices = Arc::new(Mutex::new(vec![]));
    let (tx, mut rx) = watch::channel(vec![]); // 使用 watch 通道來通知價格更新

    // 啟動價格流
    spawn_price_stream("BTC", "Crypto", Arc::clone(&prices), tx.clone());
    spawn_price_stream("ETH", "Crypto", Arc::clone(&prices), tx.clone());
    spawn_price_stream("SOL", "Crypto", Arc::clone(&prices), tx.clone());
    spawn_price_stream("AAVE", "Crypto", Arc::clone(&prices), tx.clone());
    spawn_price_stream("1INCH", "Crypto", Arc::clone(&prices), tx.clone());

    spawn_price_stream("TSLA", "Stock", Arc::clone(&prices), tx.clone());
    spawn_price_stream("AMD", "Stock", Arc::clone(&prices), tx.clone());
    spawn_price_stream("AAPL", "Stock", Arc::clone(&prices), tx.clone());

    spawn_price_stream("USD/TWD", "Forex", Arc::clone(&prices), tx.clone());

    // 被動等待價格更新並打印
    while rx.changed().await.is_ok() {
        let updated_prices = rx.borrow();
        println!("\x1B[2J\x1B[1;1H"); // 清除終端畫面
        for (symbol, price) in updated_prices.iter() {
            println!("{}: {:.2}", symbol, price);
        }
    }
}

use pyth::{get_price_stream_from_pyth, get_pyth_feed_id};

mod pyth;

use tokio::time::{sleep, Duration};


#[tokio::main]
async fn main() {
    let id = get_pyth_feed_id("ETH", "crypto").await;
    if let Err(e) = get_price_stream_from_pyth(id.as_str(), on_price).await {
        eprintln!("Error occurred: {}", e);
    }

    loop {
        sleep(Duration::from_millis(1)).await;
    }
}

fn on_price(price: f64) {
    println!("Received price: {}", price);
}

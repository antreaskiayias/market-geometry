use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RestSnapshot {
    // pub lastUpdateId: u64,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

pub async fn fetch_snapshot(symbol: &str, limit: usize) -> RestSnapshot {
    let url = format!(
        "https://api.binance.com/api/v3/depth?symbol={}&limit={}",
        symbol, limit
    );

    reqwest::get(&url)
        .await
        .expect("snapshot request failed")
        .json::<RestSnapshot>()
        .await
        .expect("snapshot parse failed")
}

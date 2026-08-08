use std::sync::{Arc, Mutex};
use crossbeam_channel::{unbounded, Sender};
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

mod order_book;
use order_book::{OrderBook, Side};

mod point_cloud;
use point_cloud::{snapshot_to_point_cloud, CloudPoint};

mod binance;
use binance::fetch_snapshot;

mod tda;

mod visualizer;
use visualizer::{Visualizer, GuiApp};

#[derive(Debug, serde::Deserialize)]
struct RawBookMessage {
    // e: String,
    // E: u64,
    // s: String,
    // U: u64,
    // u: u64,
    // pu: Option<u64>,
    b: Vec<(String, String)>,
    a: Vec<(String, String)>,
}

fn main() -> eframe::Result<()> {
    let symbols: Vec<String> = std::env::args().skip(1).collect();
    if symbols.is_empty() {
        eprintln!("Usage: market-geometry SYMBOL [SYMBOL...]");
        return Ok(());
    }

    let shared = Arc::new(Mutex::new(Visualizer {
        clouds: std::collections::HashMap::new(),
    }));

    // Tokio + WSS + REST ingestion (unchanged)
    {
        let shared_rt = shared.clone();
        let symbols_rt = symbols.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                for symbol in symbols_rt {
                    let shared_clone = shared_rt.clone();
                    tokio::spawn(async move {
                        run_symbol_ingestion(symbol, shared_clone).await;
                    });
                }
                futures::future::pending::<()>().await;
            });
        });
    }

    // GUI on main thread, using GuiApp
    let options = eframe::NativeOptions::default();
    let shared_gui = shared.clone();

    eframe::run_native(
        "Point Cloud Visualizer",
        options,
        Box::new(move |_| Box::new(GuiApp::new(shared_gui.clone()))),
    )
}

async fn run_symbol_ingestion(symbol: String, app: Arc<Mutex<Visualizer>>) {
    let ws_url = format!(
        "wss://stream.binance.com:9443/ws/{}@depth",
        symbol.to_lowercase()
    );

    let (tx, rx) = unbounded();
    tokio::spawn(run_ws_ingestion(ws_url.clone(), tx));

    // Initial REST snapshot
    let snapshot = fetch_snapshot(&symbol, 50).await;

    let mut book = OrderBook::new();
    for (p, s) in snapshot.bids {
        book.update_level(Side::Bid, p.parse().unwrap(), s.parse().unwrap());
    }
    for (p, s) in snapshot.asks {
        book.update_level(Side::Ask, p.parse().unwrap(), s.parse().unwrap());
    }

    // Streaming updates
    while let Ok(msg) = rx.recv() {
        apply_update(&mut book, &msg);

        let snapshot = book.snapshot_top_n(50);
        let cloud: Vec<CloudPoint> = snapshot_to_point_cloud(&snapshot);

        if let Ok(mut gui) = app.lock() {
            gui.update_cloud(&symbol, cloud);
        }

    }
}

async fn run_ws_ingestion(url: String, tx: Sender<RawBookMessage>) {
    let (ws_stream, _) = connect_async(url).await.expect("WS connect failed");
    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(parsed) = serde_json::from_str::<RawBookMessage>(&text) {
                    let _ = tx.send(parsed);
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("WS error: {:?}", e);
                break;
            }
        }
    }
}

fn apply_update(book: &mut OrderBook, msg: &RawBookMessage) {
    for (price, size) in &msg.b {
        let p = price.parse::<f64>().unwrap();
        let s = size.parse::<f64>().unwrap();
        book.update_level(Side::Bid, p, s);
    }
    for (price, size) in &msg.a {
        let p = price.parse::<f64>().unwrap();
        let s = size.parse::<f64>().unwrap();
        book.update_level(Side::Ask, p, s);
    }
}

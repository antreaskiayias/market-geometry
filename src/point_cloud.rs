use crate::order_book::{Level, Side};

#[derive(Clone)]
pub struct CloudPoint {
    pub x: f64,      // price offset
    pub y: f64,      // normalized size
    pub depth: usize,
    pub side: Side,
    pub ts: u64,
}

pub fn snapshot_to_point_cloud(snapshot: &Vec<Level>) -> Vec<CloudPoint> {
    if snapshot.is_empty() {
        return Vec::new();
    }

    let mid_price = snapshot.iter().map(|l| l.price).sum::<f64>() / snapshot.len() as f64;
    let max_size = snapshot.iter().map(|l| l.size).fold(0.0_f64, f64::max).max(1e-9);

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    snapshot.iter()
        .enumerate()
        .map(|(depth, lvl)| {
            let price_offset = (lvl.price - mid_price) / mid_price * 100.0;
            let size_norm = lvl.size / max_size;

            CloudPoint {
                x: price_offset,
                y: size_norm,
                depth,
                side: lvl.side,
                ts,
            }
        })
        .collect()
}

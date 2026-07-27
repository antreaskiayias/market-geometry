use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy)]
pub struct Level {
    pub price: f64,
    pub size: f64,
    pub side: Side,
}

#[derive(Debug)]
pub struct OrderBook {
    pub bids: BTreeMap<i64, f64>,
    pub asks: BTreeMap<i64, f64>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    // 👇 MUST be called as Self::price_to_key(...)
    fn price_to_key(price: f64) -> i64 {
        (price * 1_000_000.0) as i64
    }

    pub fn update_level(&mut self, side: Side, price: f64, size: f64) {
        // 👇 FIXED
        let key = Self::price_to_key(price);

        let book = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        if size == 0.0 {
            book.remove(&key);
        } else {
            book.insert(key, size);
        }
    }

    pub fn snapshot_top_n(&self, n: usize) -> Vec<Level> {
        let mut levels = Vec::with_capacity(2 * n);

        for (&key, &size) in self.bids.iter().rev().take(n) {
            levels.push(Level {
                price: key as f64 / 1_000_000.0, 
                size,
                side: Side::Bid,
            });
        }

        for (&key, &size) in self.asks.iter().take(n) {
            levels.push(Level {
                price: key as f64 / 1_000_000.0, 
                size,
                side: Side::Ask,
            });
        }

        levels
    }
}
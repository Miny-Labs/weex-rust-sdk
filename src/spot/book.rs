use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<String, f64>, // Price -> Size. Using String for price key to avoid float issues
    pub asks: BTreeMap<String, f64>,
    pub last_update_id: u64,
}

impl OrderBook {
    pub fn new(symbol: &str) -> Self {
        OrderBook {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: 0,
        }
    }

    pub fn update(&mut self, _side: &str, price: &str, _size: &str) {
        // Logic to insert/remove levels
        // if size == 0, remove. else insert.
        // Simplified stub
        let _p = price.to_string();
    }
}

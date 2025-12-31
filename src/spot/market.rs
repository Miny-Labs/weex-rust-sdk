use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ticker {
    pub symbol: String,
    pub last: Decimal,
    #[serde(rename = "best_ask")]
    pub best_ask: Decimal,
    #[serde(rename = "best_bid")]
    pub best_bid: Decimal,
}

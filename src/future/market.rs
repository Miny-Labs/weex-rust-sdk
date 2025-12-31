use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Contract {
    pub symbol: String,
    pub base_coin: String,
    pub quote_coin: String,
    pub min_trade_num: String,
}

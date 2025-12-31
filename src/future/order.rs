use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FutureOrder {
    pub symbol: String,
    pub size: String,
    pub side: String,
    pub leverage: String,
}

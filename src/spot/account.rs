use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    pub currency: Option<String>,
    pub available: Option<Decimal>,
}

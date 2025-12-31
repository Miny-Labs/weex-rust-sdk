use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::types::{Side, OrderType};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub client_oid: String,
    pub size: Decimal,
    #[serde(rename = "type")]
    pub side: Side, 
    pub order_type: OrderType, 
    pub match_price: bool, // Assuming boolean flag based on WEEX docs? Or keep String if API expects number. Keeping minimal change for now, assuming 1=market, 0=limit?
    // Actually, let's keep price as Decimal
    pub price: Option<Decimal>, 
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_oid: String,
}

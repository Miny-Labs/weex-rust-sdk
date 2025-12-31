use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Ticker {
    pub symbol: String,
    pub last: String,
    #[serde(rename = "best_ask")]
    pub best_ask: String,
    #[serde(rename = "best_bid")]
    pub best_bid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    // Determine actual fields from API response
    // Usually currency, limit, available, etc.
    // For now, map generic fields
    pub currency: Option<String>,
    pub available: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub client_oid: String,
    pub size: String,
    #[serde(rename = "type")]
    pub side: String, // "1"=Open Long, etc.
    pub order_type: String, // "0"=Limit
    pub match_price: String,
    pub price: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_oid: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Option<T>,
    // Sometimes response is flat?
    #[serde(flatten)]
    pub flat_data: Option<T>, 
}

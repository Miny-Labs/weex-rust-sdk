use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TriggerOrderRequest {
    pub symbol: String,
    pub side: String,
    pub trigger_price: String,
    pub execute_price: String, // Market or Limit
    pub size: String,
    pub trigger_type: String, // "market_price" or "fill_price"
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlanOrderResponse {
    pub order_id: String,
    pub client_oid: Option<String>,
}

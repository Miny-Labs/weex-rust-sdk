use serde::{Deserialize, Serialize};
use crate::spot::order::PlaceOrderRequest;

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchOrderRequest {
    pub symbol: String,
    pub orders: Vec<PlaceOrderRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchOrderResponse {
    // Assuming API returns a list of results or a summary
    // Need to verify exact WEEX response format, usually it's a list of OrderResponse
    pub data: Vec<crate::spot::order::OrderResponse>,
}

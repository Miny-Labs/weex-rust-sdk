use crate::spot::order::OrderResponse;
use std::collections::HashMap;

pub struct OrderManager {
    // Stub: In real implementation, this would track open orders
    pub active_orders: HashMap<String, OrderResponse>,
}

impl OrderManager {
    pub fn new() -> Self {
        OrderManager {
            active_orders: HashMap::new(),
        }
    }

    pub fn on_order_placed(&mut self, order: OrderResponse) {
        self.active_orders.insert(order.order_id.clone(), order);
    }
}

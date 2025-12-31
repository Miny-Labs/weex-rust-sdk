use async_trait::async_trait;
use crate::spot::market::Ticker;

pub struct Context {
    // Placeholder for shared state (OrderBook, etc.)
    pub exchange_name: String, 
}

#[async_trait]
pub trait Strategy {
    async fn on_tick(&mut self, ticker: Ticker, ctx: &mut Context);
    // async fn on_order_update(&mut self, order: OrderUpdate, ctx: &mut Context);
}

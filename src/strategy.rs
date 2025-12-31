use async_trait::async_trait;
use crate::spot::market::Ticker;

/// Trading context passed to strategies
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub exchange_name: String,
    pub symbol: String,
    pub price: f64,
    pub balance: f64,
    pub position: f64,
}

#[async_trait]
pub trait Strategy {
    async fn on_tick(&mut self, ticker: Ticker, ctx: &mut Context);
    // async fn on_order_update(&mut self, order: OrderUpdate, ctx: &mut Context);
}


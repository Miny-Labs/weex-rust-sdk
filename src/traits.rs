use async_trait::async_trait;
use crate::spot::market::Ticker;
use crate::error::WeexError;

#[async_trait]
pub trait Exchange {
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker, WeexError>;
    async fn get_balance(&self) -> Result<String, WeexError>; 
    // Add place_order signatures here properly typed later
}

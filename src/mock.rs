use async_trait::async_trait;
use crate::traits::Exchange;
use crate::spot::market::Ticker;
use crate::error::WeexError;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone)]
pub struct MockExchange {
    // Simulated state
    pub tickers: Ticker,
}

impl MockExchange {
    pub fn new() -> Self {
        MockExchange {
            tickers: Ticker {
                symbol: "BTCUSDT".to_string(),
                last: Decimal::from_str("100000").unwrap(),
                best_ask: Decimal::from_str("100001").unwrap(),
                best_bid: Decimal::from_str("99999").unwrap(),
            }
        }
    }
}

#[async_trait]
impl Exchange for MockExchange {
    async fn get_ticker(&self, _symbol: &str) -> Result<Ticker, WeexError> {
        Ok(self.tickers.clone()) // Always return mock data
    }
    
    async fn get_balance(&self) -> Result<String, WeexError> {
        Ok("{\"available\": \"1000000\"}".to_string())
    }
}

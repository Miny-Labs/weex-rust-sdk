use crate::ws::client::{WeexWebsocket, WsMessage, WS_PUBLIC_URL};
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use std::collections::HashMap;

/// Real-time market data from WebSocket
#[derive(Debug, Clone)]
pub enum MarketEvent {
    Ticker { symbol: String, price: f64 },
    Depth { symbol: String, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)> },
    Trade { symbol: String, price: f64, size: f64, side: String },
    Kline { symbol: String, open: f64, high: f64, low: f64, close: f64, volume: f64 },
}

/// User account events (from private WS)
#[derive(Debug, Clone)]
pub enum AccountEvent {
    OrderUpdate { order_id: String, status: String, filled_size: f64 },
    PositionUpdate { symbol: String, size: f64, entry_price: f64, pnl: f64 },
    BalanceUpdate { coin: String, available: f64, frozen: f64 },
}

/// Event handler that distributes WS messages to subscribers
pub struct EventHandler {
    market_tx: broadcast::Sender<MarketEvent>,
    account_tx: broadcast::Sender<AccountEvent>,
}

impl EventHandler {
    pub fn new() -> (Self, broadcast::Receiver<MarketEvent>, broadcast::Receiver<AccountEvent>) {
        let (market_tx, market_rx) = broadcast::channel(1000);
        let (account_tx, account_rx) = broadcast::channel(100);
        
        (Self { market_tx, account_tx }, market_rx, account_rx)
    }

    /// Subscribe to market events
    pub fn subscribe_market(&self) -> broadcast::Receiver<MarketEvent> {
        self.market_tx.subscribe()
    }

    /// Subscribe to account events
    pub fn subscribe_account(&self) -> broadcast::Receiver<AccountEvent> {
        self.account_tx.subscribe()
    }

    /// Process raw WS message and emit typed events
    pub fn handle_message(&self, msg: &WsMessage) {
        // Parse based on channel type
        if let Some(ref data) = msg.data {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    // Try to parse as ticker
                    if let (Some(symbol), Some(last)) = (
                        item.get("instId").and_then(|v| v.as_str()),
                        item.get("last").and_then(|v| v.as_str()),
                    ) {
                        if let Ok(price) = last.parse::<f64>() {
                            let _ = self.market_tx.send(MarketEvent::Ticker {
                                symbol: symbol.to_string(),
                                price,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Run the public WebSocket event loop
pub async fn run_public_ws_loop(
    symbols: Vec<String>,
    handler: EventHandler,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut ws = WeexWebsocket::public();
    let (mut write, read) = ws.connect_with_reconnect().await?;
    
    use futures::SinkExt;
    use tokio_tungstenite::tungstenite::protocol::Message;
    
    // Subscribe to tickers for all symbols
    for symbol in &symbols {
        let sub_msg = WeexWebsocket::build_subscribe_msg("ticker", symbol);
        write.send(Message::Text(sub_msg)).await?;
    }
    
    // Create channel for WS messages
    let (msg_tx, mut msg_rx) = mpsc::channel::<WsMessage>(1000);
    
    // Spawn heartbeat handler
    tokio::spawn(crate::ws::client::handle_heartbeat(write, read, msg_tx));
    
    // Process messages
    while let Some(msg) = msg_rx.recv().await {
        handler.handle_message(&msg);
    }
    
    Ok(())
}

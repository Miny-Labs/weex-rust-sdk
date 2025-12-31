use futures::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;
use std::time::Duration;
use serde::{Serialize, Deserialize};

pub const WS_PUBLIC_URL: &str = "wss://ws-spot.weex.com/v2/ws/public";
pub const WS_PRIVATE_URL: &str = "wss://ws-spot.weex.com/v2/ws/private";

#[derive(Debug, Clone, Serialize)]
pub struct SubscribeRequest {
    pub op: String,
    pub args: Vec<SubscribeArg>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscribeArg {
    #[serde(rename = "instType")]
    pub inst_type: String,
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WsMessage {
    pub event: Option<String>,
    pub arg: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
    pub action: Option<String>,
}

pub struct WeexWebsocket {
    url: String,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    reconnect_delay_ms: u64,
}

impl WeexWebsocket {
    pub fn new(url: &str) -> Self {
        WeexWebsocket {
            url: url.to_string(),
            reconnect_attempts: 0,
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 1000,
        }
    }

    pub fn public() -> Self {
        Self::new(WS_PUBLIC_URL)
    }

    pub fn private() -> Self {
        Self::new(WS_PRIVATE_URL)
    }

    /// Connect with automatic reconnection
    pub async fn connect_with_reconnect(&mut self) -> Result<(
        SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
    ), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            match self.connect().await {
                Ok(stream) => {
                    self.reconnect_attempts = 0;
                    return Ok(stream.split());
                }
                Err(e) => {
                    self.reconnect_attempts += 1;
                    if self.reconnect_attempts >= self.max_reconnect_attempts {
                        return Err(e);
                    }
                    let delay = self.reconnect_delay_ms * (1 << self.reconnect_attempts.min(6));
                    tracing::warn!("WS connect failed, retry {} in {}ms: {}", self.reconnect_attempts, delay, e);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    /// Basic connect
    pub async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error + Send + Sync>> {
        let (ws_stream, _) = connect_async(Url::parse(&self.url)?).await?;
        tracing::info!("WebSocket connected to {}", self.url);
        Ok(ws_stream)
    }

    /// Subscribe to a channel
    pub fn build_subscribe_msg(channel: &str, inst_id: &str) -> String {
        let req = SubscribeRequest {
            op: "subscribe".to_string(),
            args: vec![SubscribeArg {
                inst_type: "sp".to_string(), // spot
                channel: channel.to_string(),
                inst_id: inst_id.to_string(),
            }],
        };
        serde_json::to_string(&req).unwrap_or_default()
    }

    /// Build unsubscribe message
    pub fn build_unsubscribe_msg(channel: &str, inst_id: &str) -> String {
        let req = SubscribeRequest {
            op: "unsubscribe".to_string(),
            args: vec![SubscribeArg {
                inst_type: "sp".to_string(),
                channel: channel.to_string(),
                inst_id: inst_id.to_string(),
            }],
        };
        serde_json::to_string(&req).unwrap_or_default()
    }
}

/// Heartbeat handler - responds to ping with pong
pub async fn handle_heartbeat(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    msg_tx: mpsc::Sender<WsMessage>,
) {
    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(Message::Ping(data)) => {
                tracing::debug!("Received ping, sending pong");
                if let Err(e) = write.send(Message::Pong(data)).await {
                    tracing::error!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Text(text)) => {
                // Check if it's a ping message in text format
                if text.contains("\"ping\"") {
                    let pong = text.replace("ping", "pong");
                    if let Err(e) = write.send(Message::Text(pong)).await {
                        tracing::error!("Failed to send text pong: {}", e);
                        break;
                    }
                } else {
                    // Parse and forward the message
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        let _ = msg_tx.send(ws_msg).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket closed by server");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Convenience: Run a WS subscription loop with auto-reconnect
pub async fn run_subscription_loop(
    channel: &str,
    inst_id: &str,
    msg_tx: mpsc::Sender<WsMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut ws = WeexWebsocket::public();
    
    loop {
        let (mut write, read) = ws.connect_with_reconnect().await?;
        
        // Subscribe
        let sub_msg = WeexWebsocket::build_subscribe_msg(channel, inst_id);
        write.send(Message::Text(sub_msg)).await?;
        tracing::info!("Subscribed to {}:{}", channel, inst_id);
        
        // Handle messages with heartbeat
        handle_heartbeat(write, read, msg_tx.clone()).await;
        
        // If we exit the handler, attempt reconnect
        tracing::warn!("Connection lost, attempting reconnect...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

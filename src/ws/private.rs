use crate::WeexClient;
use crate::error::WeexError;
use crate::ws::client::{WeexWebsocket, WS_PRIVATE_URL, WsMessage, handle_heartbeat};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures::SinkExt;

/// UserStream manages authenticated WebSocket connection
pub struct UserStream {
    client: WeexClient,
    listen_key: Option<String>,
}

impl UserStream {
    pub fn new(client: WeexClient) -> Self {
        UserStream { client, listen_key: None }
    }

    /// Get or create listenKey
    pub async fn get_listen_key(&mut self) -> Result<String, WeexError> {
        let path = "/api/v2/spot/public/listenKey";
        let url = format!("{}{}", self.client.base_url, path);
        let timestamp = self.client.get_timestamp();
        let headers = self.client.build_headers("POST", path, "", "", &timestamp)?;

        let resp = self.client.client.post(&url).headers(headers).send().await?;
        let text = resp.text().await?;
        
        // Parse response to extract listenKey
        // WEEX returns: {"code":"00000","data":{"listenKey":"xxx"}}
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(key) = json.get("data").and_then(|d| d.get("listenKey")).and_then(|k| k.as_str()) {
                self.listen_key = Some(key.to_string());
                return Ok(key.to_string());
            }
        }
        
        Err(WeexError::Api { 
            code: "LISTEN_KEY_ERROR".to_string(), 
            msg: format!("Failed to get listenKey: {}", text) 
        })
    }

    /// Keep listenKey alive (call every 30 min)
    pub async fn keep_alive(&self) -> Result<(), WeexError> {
        if let Some(ref key) = self.listen_key {
            let path = "/api/v2/spot/public/listenKey";
            let body = serde_json::json!({ "listenKey": key }).to_string();
            let url = format!("{}{}", self.client.base_url, path);
            let timestamp = self.client.get_timestamp();
            let headers = self.client.build_headers("PUT", path, "", &body, &timestamp)?;

            let _ = self.client.client.put(&url).headers(headers).body(body).send().await?;
        }
        Ok(())
    }

    /// Start the private WebSocket stream with auto keep-alive
    pub async fn start(&mut self, msg_tx: mpsc::Sender<WsMessage>) -> Result<(), WeexError> {
        let listen_key = self.get_listen_key().await?;
        let ws_url = format!("{}?listenKey={}", WS_PRIVATE_URL, listen_key);
        
        // Spawn keep-alive task
        let client_clone = self.client.clone();
        let key_clone = listen_key.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1800)); // 30 min
            loop {
                ticker.tick().await;
                let path = "/api/v2/spot/public/listenKey";
                let body = serde_json::json!({ "listenKey": key_clone }).to_string();
                let url = format!("{}{}", client_clone.base_url, path);
                if let Ok(timestamp) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    let ts = timestamp.as_millis().to_string();
                    if let Ok(headers) = client_clone.build_headers("PUT", path, "", &body, &ts) {
                        let _ = client_clone.client.put(&url).headers(headers).body(body.clone()).send().await;
                        tracing::debug!("ListenKey keep-alive sent");
                    }
                }
            }
        });

        // Connect and handle messages
        let mut ws = WeexWebsocket::new(&ws_url);
        loop {
            match ws.connect_with_reconnect().await {
                Ok((mut write, read)) => {
                    // Subscribe to user channels
                    let login_msg = serde_json::json!({
                        "op": "login",
                        "args": [{
                            "apiKey": self.client.api_key,
                            "passphrase": self.client.passphrase,
                            "timestamp": self.client.get_timestamp(),
                            "sign": crate::util::generate_signature(&self.client.secret_key, &self.client.get_timestamp())
                        }]
                    }).to_string();
                    
                    if let Err(e) = write.send(Message::Text(login_msg)).await {
                        tracing::error!("Failed to send login: {}", e);
                        continue;
                    }
                    
                    // Subscribe to order and account channels
                    let sub_msg = serde_json::json!({
                        "op": "subscribe",
                        "args": [
                            { "instType": "sp", "channel": "orders", "instId": "default" },
                            { "instType": "sp", "channel": "account", "coin": "default" }
                        ]
                    }).to_string();
                    
                    if let Err(e) = write.send(Message::Text(sub_msg)).await {
                        tracing::error!("Failed to subscribe: {}", e);
                        continue;
                    }
                    
                    handle_heartbeat(write, read, msg_tx.clone()).await;
                    tracing::warn!("Private WS disconnected, reconnecting...");
                }
                Err(e) => {
                    tracing::error!("Failed to connect private WS: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}

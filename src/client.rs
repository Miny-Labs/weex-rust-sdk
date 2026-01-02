use crate::spot::market::Ticker;
use crate::spot::batch::BatchOrderRequest;
use crate::util::generate_signature;
use crate::error::WeexError;
use crate::builder::WeexClientBuilder;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::Deserialize;

// Helper for parsing response logic
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    pub code: Option<String>,
    pub msg: Option<String>,
    pub data: Option<T>,
    #[serde(flatten)]
    pub flat_data: Option<T>, 
}

#[derive(Clone)]
pub struct WeexClient {
    pub base_url: String,
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
    pub client: Client,
}

impl WeexClient {
    pub fn builder() -> WeexClientBuilder {
        WeexClientBuilder::new()
    }

    pub fn get_timestamp(&self) -> String {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_millis(0)); // Fail safe
        since_the_epoch.as_millis().to_string()
    }

    pub fn build_headers(&self, method: &str, request_path: &str, query_string: &str, body: &str, timestamp: &str) -> Result<HeaderMap, WeexError> {
        let mut headers = HeaderMap::new();
        // Safe header creation
        headers.insert("ACCESS-KEY", HeaderValue::from_str(&self.api_key).map_err(|_| WeexError::Unknown("Invalid Header Value".into()))?);
        headers.insert("ACCESS-PASSPHRASE", HeaderValue::from_str(&self.passphrase).map_err(|_| WeexError::Unknown("Invalid Header Value".into()))?);
        headers.insert("ACCESS-TIMESTAMP", HeaderValue::from_str(timestamp).map_err(|_| WeexError::Unknown("Invalid Header Value".into()))?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let message = format!("{}{}{}{}{}", timestamp, method.to_uppercase(), request_path, query_string, body);
        let signature = generate_signature(&self.secret_key, &message);
        headers.insert("ACCESS-SIGN", HeaderValue::from_str(&signature).map_err(|_| WeexError::Signing("Invalid Signature Header".into()))?);

        Ok(headers)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_ticker(&self, symbol: &str) -> Result<Ticker, WeexError> {
        let path = "/capi/v2/market/ticker";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;

        let resp = self.client.get(&url).headers(headers).send().await?;
        let text = resp.text().await?;

        // Logic to try flat vs wrapped parsing
        if let Ok(t) = serde_json::from_str::<Ticker>(&text) {
            return Ok(t);
        }
        let wrapper: ApiResponse<Ticker> = serde_json::from_str(&text)?;
        wrapper.data.or(wrapper.flat_data).ok_or_else(|| WeexError::Api { 
            code: wrapper.code.unwrap_or_default(), 
            msg: wrapper.msg.unwrap_or_default() 
        })
    }

    /// Get ticker as raw JSON string (useful for CLI)
    pub async fn get_ticker_raw(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/market/ticker";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;

        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }
    
    pub async fn get_balance(&self) -> Result<String, WeexError> {
         let path = "/capi/v2/account/balance";
         let url = format!("{}{}", self.base_url, path);
         let timestamp = self.get_timestamp();
         let headers = self.build_headers("GET", path, "", "", &timestamp)?;
         
         let resp = self.client.get(&url).headers(headers).send().await?;
         Ok(resp.text().await?)
    }

    pub async fn set_leverage(&self, symbol: &str, leverage: i32, side: crate::types::Side) -> Result<String, WeexError> {
        let path = "/capi/v2/account/leverage";
        let body = serde_json::json!({
            "symbol": symbol,
            "leverage": leverage,
            "side": side
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;

        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn set_margin_mode(&self, symbol: &str, margin_mode: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/account/setMarginMode";
        let body = serde_json::json!({
            "symbol": symbol,
            "marginMode": margin_mode
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;

        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn post_batch_orders(&self, req: BatchOrderRequest) -> Result<String, WeexError> {
        let path = "/api/v2/trade/batch-orders"; 
        let body = serde_json::to_string(&req)?;
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;

        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    // ==================== V5: CORE TRADING ====================

    /// Place a single order (Spot)
    #[tracing::instrument(skip(self))]
    pub async fn place_order(&self, req: &crate::spot::order::PlaceOrderRequest) -> Result<crate::spot::order::OrderResponse, WeexError> {
        let path = "/api/v2/trade/orders";
        let body = serde_json::to_string(req)?;
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;

        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        let text = resp.text().await?;
        
        let wrapper: ApiResponse<crate::spot::order::OrderResponse> = serde_json::from_str(&text)?;
        wrapper.data.ok_or_else(|| WeexError::Api {
            code: wrapper.code.unwrap_or_default(),
            msg: wrapper.msg.unwrap_or_else(|| text),
        })
    }

    /// Place a futures/contract order
    #[tracing::instrument(skip(self))]
    pub async fn place_futures_order(
        &self,
        symbol: &str,
        size: &str,
        side: crate::types::Side,
        order_type: crate::types::OrderType,
        price: Option<&str>,
        client_oid: Option<&str>,
    ) -> Result<String, WeexError> {
        let path = "/capi/v2/order/placeOrder";
        let mut body_map = serde_json::json!({
            "symbol": symbol,
            "size": size,
            "side": side,
            "orderType": order_type,
            "marginCoin": "USDT"
        });
        
        if let Some(p) = price {
            body_map["price"] = serde_json::Value::String(p.to_string());
        }
        if let Some(oid) = client_oid {
            body_map["clientOid"] = serde_json::Value::String(oid.to_string());
        }
        
        let body = body_map.to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;

        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Cancel a single order
    #[tracing::instrument(skip(self))]
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<String, WeexError> {
        let path = "/api/v2/trade/cancel-order";
        let body = serde_json::json!({
            "symbol": symbol,
            "orderId": order_id
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;

        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Get current open orders
    #[tracing::instrument(skip(self))]
    pub async fn get_open_orders(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/api/v2/trade/open-orders";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;

        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get futures position
    #[tracing::instrument(skip(self))]
    pub async fn get_position(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/account/position/singlePosition";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;

        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    // ==================== V5: MARKET DATA ====================

    /// Get Kline/Candlestick data
    #[tracing::instrument(skip(self))]
    pub async fn get_klines(&self, symbol: &str, interval: &str, limit: u32) -> Result<String, WeexError> {
        let path = "/capi/v2/market/candles";
        let qs = format!("?symbol={}&granularity={}&limit={}", symbol, interval, limit);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;

        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get orderbook depth snapshot
    /// depth_type: "step0" (default), "step1", "step2", etc.
    #[tracing::instrument(skip(self))]
    pub async fn get_depth(&self, symbol: &str, depth_type: Option<&str>) -> Result<String, WeexError> {
        let path = "/capi/v2/market/depth";
        let qs = format!("?symbol={}&type={}", symbol, depth_type.unwrap_or("step0"));
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;

        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get funding rate (Futures)
    #[tracing::instrument(skip(self))]
    pub async fn get_funding_rate(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/market/fundingRate";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;

        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    // ==================== AI WARS: MARKET DATA (PUBLIC) ====================

    /// Get server time
    pub async fn get_server_time(&self) -> Result<String, WeexError> {
        let path = "/capi/v2/market/time";
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    /// Get futures contract information
    pub async fn get_contracts(&self, symbol: Option<&str>) -> Result<String, WeexError> {
        let path = "/capi/v2/market/contracts";
        let qs = symbol.map(|s| format!("?symbol={}", s)).unwrap_or_default();
        let url = format!("{}{}{}", self.base_url, path, qs);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    /// Get all tickers
    pub async fn get_all_tickers(&self) -> Result<String, WeexError> {
        let path = "/capi/v2/market/tickers";
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    /// Get recent trades
    pub async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<String, WeexError> {
        let path = "/capi/v2/market/trades";
        let qs = format!("?symbol={}&limit={}", symbol, limit.unwrap_or(100));
        let url = format!("{}{}{}", self.base_url, path, qs);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    /// Get cryptocurrency index
    pub async fn get_index(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/market/index";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    /// Get open interest
    pub async fn get_open_interest(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/market/openInterest";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    /// Get next funding time
    pub async fn get_funding_time(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/market/fundingTime";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    /// Get historical funding rates
    pub async fn get_history_funding_rate(&self, symbol: &str, page_size: Option<u32>) -> Result<String, WeexError> {
        let path = "/capi/v2/market/historyFundingRate";
        let qs = format!("?symbol={}&pageSize={}", symbol, page_size.unwrap_or(20));
        let url = format!("{}{}{}", self.base_url, path, qs);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.text().await?)
    }

    // ==================== AI WARS: ACCOUNT (PRIVATE) ====================

    /// Get account assets
    #[tracing::instrument(skip(self))]
    pub async fn get_assets(&self) -> Result<String, WeexError> {
        let path = "/capi/v2/account/assets";
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, "", "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get account bills/ledger
    #[tracing::instrument(skip(self))]
    pub async fn get_bills(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/account/bills";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get futures settings for a symbol
    #[tracing::instrument(skip(self))]
    pub async fn get_settings(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/account/settings";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Adjust position margin
    #[tracing::instrument(skip(self))]
    pub async fn adjust_margin(&self, symbol: &str, amount: &str, side: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/account/adjustPositionMargin";
        let body = serde_json::json!({
            "symbol": symbol,
            "amount": amount,
            "side": side
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Set auto margin top-up
    #[tracing::instrument(skip(self))]
    pub async fn set_auto_margin(&self, symbol: &str, side: &str, auto_margin: bool) -> Result<String, WeexError> {
        let path = "/capi/v2/account/autoMargin";
        let body = serde_json::json!({
            "symbol": symbol,
            "side": side,
            "autoMargin": if auto_margin { "on" } else { "off" }
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Get all positions
    #[tracing::instrument(skip(self))]
    pub async fn get_all_positions(&self) -> Result<String, WeexError> {
        let path = "/capi/v2/account/position/allPosition";
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, "", "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Modify position mode (one-way/hedge)
    #[tracing::instrument(skip(self))]
    pub async fn modify_pos_mode(&self, symbol: &str, pos_mode: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/account/modifyPosMode";
        let body = serde_json::json!({
            "symbol": symbol,
            "posMode": pos_mode
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    // ==================== AI WARS: TRADE (PRIVATE) ====================

    /// Get order details
    #[tracing::instrument(skip(self))]
    pub async fn get_order_detail(&self, symbol: &str, order_id: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/detail";
        let qs = format!("?symbol={}&orderId={}", symbol, order_id);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get order history
    #[tracing::instrument(skip(self))]
    pub async fn get_order_history(&self, symbol: &str, page_size: Option<u32>) -> Result<String, WeexError> {
        let path = "/capi/v2/order/history";
        let qs = format!("?symbol={}&pageSize={}", symbol, page_size.unwrap_or(20));
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get current/open orders
    #[tracing::instrument(skip(self))]
    pub async fn get_current_orders(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/current";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get trade fills
    #[tracing::instrument(skip(self))]
    pub async fn get_fills(&self, symbol: &str, order_id: Option<&str>) -> Result<String, WeexError> {
        let path = "/capi/v2/order/fills";
        let qs = match order_id {
            Some(oid) => format!("?symbol={}&orderId={}", symbol, oid),
            None => format!("?symbol={}", symbol),
        };
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Cancel futures order (correct path)
    #[tracing::instrument(skip(self))]
    pub async fn cancel_futures_order(&self, symbol: &str, order_id: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/cancelOrder";
        let body = serde_json::json!({
            "symbol": symbol,
            "orderId": order_id
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Place trigger order
    #[tracing::instrument(skip(self))]
    pub async fn place_trigger_order(
        &self,
        symbol: &str,
        size: &str,
        side: &str,
        trigger_price: &str,
        execute_price: Option<&str>,
    ) -> Result<String, WeexError> {
        let path = "/capi/v2/order/placeTriggerOrder";
        let mut body_map = serde_json::json!({
            "symbol": symbol,
            "size": size,
            "side": side,
            "triggerPrice": trigger_price,
            "triggerType": "fill_price"
        });
        if let Some(ep) = execute_price {
            body_map["executePrice"] = serde_json::Value::String(ep.to_string());
        }
        let body = body_map.to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Cancel trigger order
    #[tracing::instrument(skip(self))]
    pub async fn cancel_trigger_order(&self, symbol: &str, order_id: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/cancelTriggerOrder";
        let body = serde_json::json!({
            "symbol": symbol,
            "orderId": order_id
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Get current plan orders
    #[tracing::instrument(skip(self))]
    pub async fn get_current_plan(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/currentPlan";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Get history plan orders
    #[tracing::instrument(skip(self))]
    pub async fn get_history_plan(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/historyPlan";
        let qs = format!("?symbol={}", symbol);
        let url = format!("{}{}{}", self.base_url, path, qs);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("GET", path, &qs, "", &timestamp)?;
        let resp = self.client.get(&url).headers(headers).send().await?;
        Ok(resp.text().await?)
    }

    /// Close all positions
    #[tracing::instrument(skip(self))]
    pub async fn close_all_positions(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/closeAllPositions";
        let body = serde_json::json!({
            "symbol": symbol
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Cancel all orders
    #[tracing::instrument(skip(self))]
    pub async fn cancel_all_orders(&self, symbol: &str) -> Result<String, WeexError> {
        let path = "/capi/v2/order/cancelAllOrders";
        let body = serde_json::json!({
            "symbol": symbol
        }).to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Place TP/SL order
    #[tracing::instrument(skip(self))]
    pub async fn place_tpsl(
        &self,
        symbol: &str,
        side: &str,
        tp_price: Option<&str>,
        sl_price: Option<&str>,
    ) -> Result<String, WeexError> {
        let path = "/capi/v2/order/placeTPSL";
        let mut body_map = serde_json::json!({
            "symbol": symbol,
            "side": side
        });
        if let Some(tp) = tp_price {
            body_map["presetTakeProfitPrice"] = serde_json::Value::String(tp.to_string());
        }
        if let Some(sl) = sl_price {
            body_map["presetStopLossPrice"] = serde_json::Value::String(sl.to_string());
        }
        let body = body_map.to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    /// Modify TP/SL order
    #[tracing::instrument(skip(self))]
    pub async fn modify_tpsl(
        &self,
        symbol: &str,
        side: &str,
        tp_price: Option<&str>,
        sl_price: Option<&str>,
    ) -> Result<String, WeexError> {
        let path = "/capi/v2/order/modifyTPSL";
        let mut body_map = serde_json::json!({
            "symbol": symbol,
            "side": side
        });
        if let Some(tp) = tp_price {
            body_map["presetTakeProfitPrice"] = serde_json::Value::String(tp.to_string());
        }
        if let Some(sl) = sl_price {
            body_map["presetStopLossPrice"] = serde_json::Value::String(sl.to_string());
        }
        let body = body_map.to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }

    // ==================== AI WARS: AI LOG (CRITICAL) ====================

    /// Upload AI log for competition compliance
    #[tracing::instrument(skip(self, input, output))]
    pub async fn upload_ai_log(
        &self,
        order_id: Option<i64>,
        stage: &str,
        model: &str,
        input: serde_json::Value,
        output: serde_json::Value,
        explanation: &str,
    ) -> Result<String, WeexError> {
        let path = "/capi/v2/order/uploadAiLog";
        let mut body_map = serde_json::json!({
            "stage": stage,
            "model": model,
            "input": input,
            "output": output,
            "explanation": explanation
        });
        if let Some(oid) = order_id {
            body_map["orderId"] = serde_json::Value::Number(oid.into());
        }
        let body = body_map.to_string();
        let url = format!("{}{}", self.base_url, path);
        let timestamp = self.get_timestamp();
        let headers = self.build_headers("POST", path, "", &body, &timestamp)?;
        let resp = self.client.post(&url).headers(headers).body(body).send().await?;
        Ok(resp.text().await?)
    }
}

use crate::traits::Exchange;
use async_trait::async_trait;

#[async_trait]
impl Exchange for WeexClient {
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker, WeexError> {
        self.get_ticker(symbol).await
    }
    async fn get_balance(&self) -> Result<String, WeexError> {
        self.get_balance().await
    }
}



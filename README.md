# WEEX Rust SDK

[![Crates.io](https://img.shields.io/crates/v/weex_rust_sdk)](https://crates.io/crates/weex_rust_sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Professional-grade async Rust SDK for [WEEX Exchange](https://www.weex.com).

## Features

- ✅ **Async/Await** - Built on Tokio for high-performance concurrent trading
- ✅ **Type-Safe** - Decimal precision, strict enums, no stringly-typed APIs
- ✅ **Full Trading** - Place, cancel, batch orders + position management
- ✅ **Market Data** - Ticker, Klines, Orderbook, Funding Rate
- ✅ **WebSocket** - Real-time streams with auto-reconnect
- ✅ **Rate Limiting** - Built-in token bucket to prevent bans
- ✅ **Retry Logic** - Exponential backoff for network resilience

## Installation

```toml
[dependencies]
weex_rust_sdk = "0.5"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use weex_rust_sdk::WeexClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = WeexClient::builder()
        .base_url("https://api.weex.com")
        .api_key("YOUR_API_KEY")
        .secret_key("YOUR_SECRET_KEY")
        .passphrase("YOUR_PASSPHRASE")
        .build()?;

    // Get BTC ticker
    let ticker = client.get_ticker("BTCUSDT").await?;
    println!("BTC: {}", ticker.last);

    // Get klines
    let klines = client.get_klines("BTCUSDT", "1h", 100).await?;

    // Place order
    use weex_rust_sdk::spot::order::PlaceOrderRequest;
    use weex_rust_sdk::types::{Side, OrderType};
    use rust_decimal::Decimal;

    let order = PlaceOrderRequest {
        symbol: "BTCUSDT".to_string(),
        client_oid: "my_order_001".to_string(),
        size: Decimal::from(1),
        side: Side::Buy,
        order_type: OrderType::Limit,
        match_price: false,
        price: Some(Decimal::from(50000)),
    };
    let response = client.place_order(&order).await?;
    println!("Order ID: {}", response.order_id);

    Ok(())
}
```

## WebSocket Streaming

```rust
use weex_rust_sdk::ws::client::{WeexWebsocket, run_subscription_loop};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);
    
    tokio::spawn(run_subscription_loop("books", "BTCUSDT", tx));
    
    while let Some(msg) = rx.recv().await {
        println!("{:?}", msg);
    }
}
```

## API Coverage

| Category | Methods |
|:---------|:--------|
| **Trading** | `place_order`, `cancel_order`, `get_open_orders`, `post_batch_orders` |
| **Market** | `get_ticker`, `get_klines`, `get_depth`, `get_funding_rate` |
| **Account** | `get_balance`, `get_position`, `set_leverage`, `set_margin_mode` |
| **WebSocket** | Public streams, Private streams with auth |

## Rate Limiting

```rust
use weex_rust_sdk::rate_limiter::default_weex_limiter;

let limiter = default_weex_limiter(); // 10 req/sec
limiter.acquire().await;
client.get_ticker("BTCUSDT").await?;
```

## License

MIT

# WEEX Rust SDK

[![Crates.io](https://img.shields.io/crates/v/weex_rust_sdk)](https://crates.io/crates/weex_rust_sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Professional-grade async Rust SDK for [WEEX Exchange](https://www.weex.com)** - Built for HFT bots, AI trading strategies, and production systems.

## Features

### Core Trading
- ✅ **Place/Cancel Orders** - Spot & Futures with type-safe enums
- ✅ **Batch Orders** - Execute multiple orders atomically
- ✅ **Position Management** - Real-time position tracking
- ✅ **Leverage/Margin Control** - Configure risk parameters

### Market Data
- ✅ **Real-time Ticker** - Live price feeds
- ✅ **Klines/Candlesticks** - OHLCV data for TA
- ✅ **Orderbook Depth** - Level 2 market data
- ✅ **Funding Rates** - Futures funding info

### Production Infrastructure
- ✅ **Rate Limiter** - Token bucket (10 req/sec)
- ✅ **Retry Middleware** - Exponential backoff
- ✅ **WebSocket** - Auto-reconnect with heartbeat
- ✅ **Position Sizing** - Fixed %, Kelly Criterion
- ✅ **State Persistence** - Trade logging, PnL tracking
- ✅ **Telegram Alerts** - Trade notifications

## Installation

```toml
[dependencies]
weex_rust_sdk = "0.5"
tokio = { version = "1", features = ["full"] }
rust_decimal = "1.33"
```

## Quick Start

```rust
use weex_rust_sdk::WeexClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = WeexClient::builder()
        .base_url("https://api-contract.weex.com")
        .api_key("YOUR_API_KEY")
        .secret_key("YOUR_SECRET_KEY")
        .passphrase("YOUR_PASSPHRASE")
        .build()?;

    // Get BTC price
    let ticker = client.get_ticker("cmt_btcusdt").await?;
    println!("BTC: ${}", ticker.last);

    // Place futures order
    use weex_rust_sdk::types::{Side, OrderType};
    let response = client.place_futures_order(
        "cmt_btcusdt",
        "0.001",
        Side::Buy,
        OrderType::Limit,
        Some("50000"),
        Some("my_order_1"),
    ).await?;

    Ok(())
}
```

## Risk Management

```rust
use weex_rust_sdk::{RiskConfig, PositionSizer};
use rust_decimal::Decimal;

let sizer = PositionSizer::new(RiskConfig::default());

// Fixed percentage (2% risk per trade)
let size = sizer.fixed_percentage(
    Decimal::from(10000),  // $10k account
    Decimal::from(90000),  // $90k BTC
    Decimal::from_str("0.02").unwrap(), // 2% stop loss
);

// Kelly Criterion
let kelly_size = sizer.kelly_criterion(
    Decimal::from(10000),
    Decimal::from_str("0.55").unwrap(), // 55% win rate
    Decimal::from(200),  // avg win
    Decimal::from(100),  // avg loss
);
```

## State Persistence

```rust
use weex_rust_sdk::{StateManager, TradeRecord};

let state = StateManager::new("./bot_data");

// Log trade
state.log_trade(&TradeRecord { /* ... */ })?;

// Get statistics
let stats = state.calculate_stats()?;
println!("Win Rate: {:.1}%", stats.win_rate * 100.0);
```

## Telegram Alerts

```rust
use weex_rust_sdk::{TelegramAlerter, TelegramConfig};

let alerter = TelegramAlerter::new(TelegramConfig::new(
    "YOUR_BOT_TOKEN",
    "YOUR_CHAT_ID",
));

alerter.notify_trade("BTCUSDT", "BUY", 0.001, 90000.0).await?;
```

## API Coverage

| Category | Methods |
|:---------|:--------|
| **Trading** | `place_order`, `place_futures_order`, `cancel_order`, `post_batch_orders` |
| **Market** | `get_ticker`, `get_klines`, `get_depth`, `get_funding_rate` |
| **Account** | `get_balance`, `get_position`, `get_open_orders`, `set_leverage`, `set_margin_mode` |
| **WebSocket** | Public streams, Private streams with auth |
| **Bot Infra** | Rate limiter, Retry, Position sizing, State, Alerts |

## Architecture

```
weex_rust_sdk/
├── src/
│   ├── client.rs       # WeexClient (main entry point)
│   ├── types.rs        # Side, OrderType enums
│   ├── risk.rs         # Position sizing
│   ├── engine.rs       # Strategy orchestration
│   ├── state.rs        # Trade persistence
│   ├── alerts.rs       # Telegram notifications
│   ├── rate_limiter.rs # Token bucket
│   ├── retry.rs        # Exponential backoff
│   └── ws/             # WebSocket handlers
```

## Examples

See [`examples/`](./examples/) for:
- `full_test.rs` - API endpoint verification
- `v6_integration_test.rs` - Production bot features
- `market_maker.rs` - Strategy template

## License

MIT

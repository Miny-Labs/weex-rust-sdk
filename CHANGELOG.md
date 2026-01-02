# Changelog

All notable changes to WEEX Rust SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-01-02

### Added
- 🤖 AI Log upload endpoint for AI Wars competition
- 📊 30+ new API endpoints for full AI Wars coverage
- 📈 Market endpoints: contracts, all tickers, trades, index, open interest, funding time, history funding
- 💼 Account endpoints: assets, bills, settings, adjust margin, auto margin, all positions, position mode
- ⚡ Trade endpoints: order detail, history, current, fills, trigger orders, plan orders, close all, cancel all, TP/SL
- 🏷️ New types: `MarginMode`, `PositionSide`, `TriggerType`, `AILogStage`

### Fixed
- `get_klines` endpoint path corrected to `/capi/v2/market/candles`
- `get_depth` endpoint uses `type` parameter instead of `limit`

### Changed
- Updated author to "Akash Mondal"
- Enhanced package description for AI Wars

## [0.5.0] - 2025-12-31

### Added
- Core trading functionality
- WebSocket support
- Rate limiting middleware
- Retry logic with exponential backoff
- Strategy and Engine modules
- State persistence
- Telegram alerts
- Published to crates.io

## [0.4.0] - 2025-12-30

### Added
- Futures trading support
- Position management
- Leverage configuration

## [0.3.0] - 2025-12-29

### Added
- Spot trading endpoints
- Order management
- Account balance queries

## [0.2.0] - 2025-12-28

### Added
- HMAC-SHA256 authentication
- Builder pattern for client
- Error handling with `WeexError`

## [0.1.0] - 2025-12-27

### Added
- Initial release
- Basic HTTP client
- Market data endpoints

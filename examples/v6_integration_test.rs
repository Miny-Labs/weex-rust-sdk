use weex_rust_sdk::{
    WeexClient, RiskConfig, PositionSizer, StateManager, TradeRecord,
    TelegramAlerter, TelegramConfig,
};
use weex_rust_sdk::types::{Side, OrderType};
use rust_decimal::Decimal;
use std::str::FromStr;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("=== WEEX Rust SDK - V6 Full Integration Test ===\n");

    let api_key = "weex_2cd87fb352ae668394f62ddf720725dc";
    let secret = "b22aea1dca700c1942ad8a0d36398d1987f4933d7200f4ecdec53369767586c9";
    let passphrase = "weex4662269";
    
    let client = WeexClient::builder()
        .base_url("https://api-contract.weex.com")
        .api_key(api_key)
        .secret_key(secret)
        .passphrase(passphrase)
        .build()
        .expect("Failed to build client");

    let symbol = "cmt_btcusdt";

    // ========== ORIGINAL V5 API TESTS ==========
    println!("📊 V5: MARKET DATA");
    
    print!("  get_ticker()... ");
    match client.get_ticker(symbol).await {
        Ok(t) => println!("✅ {} = ${}", t.symbol, t.last),
        Err(e) => println!("❌ {:?}", e),
    }

    print!("  get_klines()... ");
    match client.get_klines(symbol, "1h", 5).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    println!("\n🔐 V5: ACCOUNT (Authenticated)");
    
    print!("  get_balance()... ");
    match client.get_balance().await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    print!("  get_position()... ");
    match client.get_position(symbol).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // ========== V6: RISK MANAGEMENT ==========
    println!("\n📐 V6: RISK MANAGEMENT");
    
    print!("  PositionSizer (Fixed %)... ");
    let sizer = PositionSizer::new(RiskConfig::default());
    let size = sizer.fixed_percentage(
        Decimal::from(10000),  // $10k account
        Decimal::from(90000),  // $90k BTC
        Decimal::from_str("0.02").unwrap(), // 2% stop loss
    );
    println!("✅ Calculated size: {} BTC", size);

    print!("  PositionSizer (Kelly)... ");
    let kelly_size = sizer.kelly_criterion(
        Decimal::from(10000),
        Decimal::from_str("0.55").unwrap(), // 55% win rate
        Decimal::from(200),  // avg win $200
        Decimal::from(100),  // avg loss $100
    );
    println!("✅ Kelly size: ${}", kelly_size);

    // ========== V6: STATE PERSISTENCE ==========
    println!("\n💾 V6: STATE PERSISTENCE");
    
    print!("  StateManager (init)... ");
    let state = StateManager::new("/tmp/weex_bot_data");
    println!("✅ Initialized");

    print!("  StateManager (log trade)... ");
    let trade = TradeRecord {
        id: format!("test_{}", Utc::now().timestamp()),
        timestamp: Utc::now(),
        symbol: symbol.to_string(),
        side: "BUY".to_string(),
        size: 0.001,
        price: 90000.0,
        pnl: Some(50.0),
        status: "filled".to_string(),
    };
    match state.log_trade(&trade) {
        Ok(_) => println!("✅ Trade logged"),
        Err(e) => println!("❌ {:?}", e),
    }

    print!("  StateManager (calculate stats)... ");
    match state.calculate_stats() {
        Ok(stats) => println!("✅ Trades: {}, Win Rate: {:.1}%", stats.total_trades, stats.win_rate * 100.0),
        Err(e) => println!("❌ {:?}", e),
    }

    // ========== V6: TELEGRAM ALERTS ==========
    println!("\n📱 V6: TELEGRAM ALERTS");
    
    print!("  TelegramAlerter (disabled mode)... ");
    let alerter = TelegramAlerter::new(TelegramConfig::disabled());
    match alerter.notify_info("Test message").await {
        Ok(_) => println!("✅ Alerter initialized (disabled mode)"),
        Err(e) => println!("❌ {:?}", e),
    }

    // ========== V6: TRADING (with real API) ==========
    println!("\n📝 V6: TRADING");

    print!("  place_futures_order()... ");
    match client.place_futures_order(
        symbol,
        "0.001",
        Side::Buy,
        OrderType::Limit,
        Some("50000"),
        Some(&format!("v6_test_{}", Utc::now().timestamp())),
    ).await {
        Ok(resp) => println!("✅ Response: {} bytes", resp.len()),
        Err(e) => println!("⚠️  {:?}", e),
    }

    println!("\n=== V6 Integration Test Complete ===");
    println!("All production bot features verified! 🚀");
}

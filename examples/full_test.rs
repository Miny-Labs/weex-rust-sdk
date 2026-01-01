use weex_rust_sdk::WeexClient;
use weex_rust_sdk::types::{Side, OrderType};
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    println!("=== WEEX Rust SDK - Full Feature Test ===\n");

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

    // ========== MARKET DATA ==========
    println!("📊 MARKET DATA");
    
    // 1. Ticker
    print!("  get_ticker()... ");
    match client.get_ticker(symbol).await {
        Ok(t) => println!("✅ {} = ${}", t.symbol, t.last),
        Err(e) => println!("❌ {:?}", e),
    }

    // 2. Klines
    print!("  get_klines()... ");
    match client.get_klines(symbol, "1h", 5).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // 3. Orderbook Depth
    print!("  get_depth()... ");
    match client.get_depth(symbol, None).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // 4. Funding Rate
    print!("  get_funding_rate()... ");
    match client.get_funding_rate(symbol).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // ========== ACCOUNT (Private/Auth) ==========
    println!("\n🔐 ACCOUNT (Authenticated)");

    // 5. Balance
    print!("  get_balance()... ");
    match client.get_balance().await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // 6. Position
    print!("  get_position()... ");
    match client.get_position(symbol).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // 7. Open Orders
    print!("  get_open_orders()... ");
    match client.get_open_orders(symbol).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // ========== TRADING CONFIG ==========
    println!("\n⚙️  TRADING CONFIG");

    // 8. Set Leverage
    print!("  set_leverage()... ");
    match client.set_leverage(symbol, 10, Side::Buy).await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // 9. Set Margin Mode
    print!("  set_margin_mode()... ");
    match client.set_margin_mode(symbol, "crossed").await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("❌ {:?}", e),
    }

    // ========== ORDER PLACEMENT (Dry Run - will fail if no balance) ==========
    println!("\n📝 ORDER PLACEMENT (may fail if no balance)");

    // 10. Place Futures Order
    print!("  place_futures_order()... ");
    match client.place_futures_order(
        symbol,
        "0.001",
        Side::Buy,
        OrderType::Limit,
        Some("50000"),
        Some(&format!("test_{}", chrono::Utc::now().timestamp())),
    ).await {
        Ok(resp) => println!("✅ Response: {} bytes", resp.len()),
        Err(e) => println!("⚠️  {:?}", e),
    }

    // 11. Cancel Order (will fail if no order exists)
    print!("  cancel_order()... ");
    match client.cancel_order(symbol, "fake_order_id").await {
        Ok(data) => println!("✅ Got {} bytes", data.len()),
        Err(e) => println!("⚠️  {:?}", e),
    }

    println!("\n=== Tests Complete ===");
}

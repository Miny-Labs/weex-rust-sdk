/**
 * weex-cli - Command Line Interface for WEEX Rust SDK
 * 
 * This binary wraps the weex_rust_sdk library to provide
 * a CLI interface that can be called from other languages (TypeScript, Python).
 */

use weex_rust_sdk::{WeexClient, types::{Side, OrderType}};
use std::env;
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: weex-cli <command> [options]");
        eprintln!("Commands: ticker, depth, candles, funding, assets, positions, order, ai-log");
        std::process::exit(1);
    }

    // Load credentials from environment
    let api_key = match env::var("WEEX_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            println!(r#"{{"success": false, "error": "WEEX_API_KEY not set"}}"#);
            std::process::exit(1);
        }
    };
    let secret_key = match env::var("WEEX_SECRET_KEY") {
        Ok(k) => k,
        Err(_) => {
            println!(r#"{{"success": false, "error": "WEEX_SECRET_KEY not set"}}"#);
            std::process::exit(1);
        }
    };
    let passphrase = match env::var("WEEX_PASSPHRASE") {
        Ok(k) => k,
        Err(_) => {
            println!(r#"{{"success": false, "error": "WEEX_PASSPHRASE not set"}}"#);
            std::process::exit(1);
        }
    };
    let base_url = env::var("WEEX_BASE_URL").unwrap_or_else(|_| "https://api-contract.weex.com".to_string());

    let client = match WeexClient::builder()
        .base_url(&base_url)
        .api_key(&api_key)
        .secret_key(&secret_key)
        .passphrase(&passphrase)
        .build() {
            Ok(c) => c,
            Err(e) => {
                println!(r#"{{"success": false, "error": "Failed to build client: {:?}"}}"#, e);
                std::process::exit(1);
            }
        };

    let command = &args[1];
    let result = match command.as_str() {
        "ticker" => {
            let symbol = get_arg(&args, "--symbol").unwrap_or("cmt_btcusdt".to_string());
            match client.get_ticker_raw(&symbol).await {
                Ok(raw) => {
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": raw})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "depth" => {
            let symbol = get_arg(&args, "--symbol").unwrap_or("cmt_btcusdt".to_string());
            match client.get_depth(&symbol, None).await {
                Ok(raw) => {
                    // Parse raw JSON string
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": raw})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "candles" => {
            let symbol = get_arg(&args, "--symbol").unwrap_or("cmt_btcusdt".to_string());
            let granularity = get_arg(&args, "--granularity").unwrap_or("1H".to_string());
            let limit = get_arg(&args, "--limit").unwrap_or("50".to_string()).parse().unwrap_or(50);
            match client.get_klines(&symbol, &granularity, limit).await {
                Ok(raw) => {
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": raw})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "funding" => {
            let symbol = get_arg(&args, "--symbol").unwrap_or("cmt_btcusdt".to_string());
            match client.get_funding_rate(&symbol).await {
                Ok(raw) => {
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": raw})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "assets" => {
            match client.get_assets().await {
                Ok(raw) => {
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": raw})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "positions" => {
            match client.get_all_positions().await {
                Ok(raw) => {
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": raw})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },

        "order-history" => {
            let symbol = get_arg(&args, "--symbol").unwrap_or("cmt_btcusdt".to_string());
            match client.get_order_history(&symbol, None).await {
                Ok(raw) => {
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": raw})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "order" => {
            let symbol = get_arg(&args, "--symbol").unwrap_or_else(|| {
                println!(r#"{{"success": false, "error": "--symbol required"}}"#);
                std::process::exit(1);
            });
            let side_str = get_arg(&args, "--side").unwrap_or_else(|| {
                println!(r#"{{"success": false, "error": "--side required (buy/sell)"}}"#);
                std::process::exit(1);
            });
            let size_str = get_arg(&args, "--size").unwrap_or_else(|| {
                println!(r#"{{"success": false, "error": "--size required"}}"#);
                std::process::exit(1);
            });
            
            let side = if side_str == "buy" { Side::Buy } else { Side::Sell };
            
            match client.place_futures_order(&symbol, &size_str, side, OrderType::Market, None, None).await {
                Ok(order_id) => json!({
                    "success": true,
                    "data": {
                        "order_id": order_id
                    }
                }),
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "ai-log" => {
            let stage = get_arg(&args, "--stage").unwrap_or_else(|| {
                println!(r#"{{"success": false, "error": "--stage required"}}"#);
                std::process::exit(1);
            });
            let model = get_arg(&args, "--model").unwrap_or_else(|| {
                println!(r#"{{"success": false, "error": "--model required"}}"#);
                std::process::exit(1);
            });
            let input_str = get_arg(&args, "--input").unwrap_or("{}".to_string());
            let output_str = get_arg(&args, "--output").unwrap_or("{}".to_string());
            let explanation = get_arg(&args, "--explanation").unwrap_or("AI decision".to_string());
            let order_id = get_arg(&args, "--order-id").and_then(|s| s.parse::<i64>().ok());
            
            let input: Value = serde_json::from_str(&input_str).unwrap_or(json!({}));
            let output: Value = serde_json::from_str(&output_str).unwrap_or(json!({}));
            
            match client.upload_ai_log(order_id, &stage, &model, input, output, &explanation).await {
                Ok(response) => {
                    match serde_json::from_str::<Value>(&response) {
                        Ok(v) => json!({"success": true, "data": v}),
                        Err(_) => json!({"success": true, "data": response})
                    }
                },
                Err(e) => json!({"success": false, "error": format!("{:?}", e)})
            }
        },
        
        "help" | "--help" | "-h" => {
            json!({
                "name": "weex-cli",
                "version": "0.6.2",
                "description": "CLI for WEEX Rust SDK",
                "commands": {
                    "ticker": "--symbol <symbol>",
                    "depth": "--symbol <symbol>",
                    "candles": "--symbol <s> --granularity <1H> --limit <50>",
                    "funding": "--symbol <symbol>",
                    "assets": "(no args)",
                    "positions": "(no args)",
                    "order-history": "--symbol <symbol>",
                    "order": "--symbol <s> --side <buy/sell> --size <n>",
                    "ai-log": "--stage <s> --model <m> --input <json> --output <json> --explanation <text>"
                }
            })
        },
        
        _ => {
            json!({"success": false, "error": format!("Unknown command: {}. Use 'help'.", command)})
        }
    };

    println!("{}", serde_json::to_string(&result).unwrap());
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

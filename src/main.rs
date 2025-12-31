use weex_rust_sdk::WeexClient;
use std::env;
use std::time::Instant;
use futures::future::join_all;
use tokio;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let benchmark_mode = args.contains(&"--benchmark".to_string());

    if benchmark_mode {
        run_benchmark().await;
        return;
    }

    println!("Starting WEEX Rust SDK Example (Async)...");
    // Normal single execution (V4 Builder)
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

    match client.get_ticker("cmt_btcusdt").await {
         Ok(t) => println!("Ticker: {} Price: {}", t.symbol, t.last),
         Err(e) => eprintln!("Error: {:?}", e),
    }
}

async fn run_benchmark() {
    println!("Starting Rust HFT Benchmark (50 Concurrent Requests)...");
    let api_key = "weex_2cd87fb352ae668394f62ddf720725dc";
    let secret = "b22aea1dca700c1942ad8a0d36398d1987f4933d7200f4ecdec53369767586c9";
    let passphrase = "weex4662269";
    // Initialize Client (V4 Builder)
    let client = WeexClient::builder()
        .base_url("https://api-contract.weex.com")
        .api_key(&api_key)
        .secret_key(&secret)
        .passphrase(&passphrase)
        .build()
        .expect("Failed to build client");
    let iterations = 50;
    
    let start = Instant::now();
    let mut tasks = Vec::new();

    for _ in 0..iterations {
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            let req_start = Instant::now();
            let res = c.get_ticker("cmt_btcusdt").await;
            (res.is_ok(), req_start.elapsed().as_millis())
        }));
    }

    let results = join_all(tasks).await;
    let total_duration = start.elapsed();

    let mut latencies = Vec::new();
    let mut success_count = 0;

    for res in results {
        if let Ok((is_ok, latency)) = res {
            if is_ok {
                success_count += 1;
                latencies.push(latency);
            }
        }
    }

    if latencies.is_empty() {
        println!("No successful requests");
        return;
    }

    latencies.sort();
    let min = latencies.first().unwrap();
    let max = latencies.last().unwrap();
    let sum: u128 = latencies.iter().sum();
    let avg = sum as f64 / latencies.len() as f64;
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];

    println!("\nRust HFT Results (Concurrent):");
    println!("Total Wall Time: {}ms (for {} reqs)", total_duration.as_millis(), iterations);
    println!("Success: {}/{}", success_count, iterations);
    println!("Min: {}ms", min);
    println!("Max: {}ms", max);
    println!("Avg: {:.2}ms", avg);
    println!("P95: {}ms", p95);
    println!("Throughput: {:.2} req/sec", (success_count as f64 / total_duration.as_secs_f64()));
}

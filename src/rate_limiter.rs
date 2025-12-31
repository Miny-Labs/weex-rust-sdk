use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Simple Token Bucket Rate Limiter
/// Prevents exceeding WEEX's 10 req/sec limit
pub struct RateLimiter {
    tokens: AtomicU64,
    max_tokens: u64,
    refill_rate: u64, // tokens per second
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(max_tokens: u64, refill_rate: u64) -> Self {
        RateLimiter {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    /// Wait until a token is available, then consume it
    pub async fn acquire(&self) {
        loop {
            // Refill tokens based on elapsed time
            {
                let mut last = self.last_refill.lock().await;
                let elapsed = last.elapsed().as_secs();
                if elapsed > 0 {
                    let new_tokens = elapsed * self.refill_rate;
                    let current = self.tokens.load(Ordering::SeqCst);
                    let refilled = std::cmp::min(current + new_tokens, self.max_tokens);
                    self.tokens.store(refilled, Ordering::SeqCst);
                    *last = Instant::now();
                }
            }

            // Try to consume a token
            let current = self.tokens.load(Ordering::SeqCst);
            if current > 0 {
                self.tokens.fetch_sub(1, Ordering::SeqCst);
                return;
            }

            // No tokens, wait a bit
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

/// Default rate limiter for WEEX (10 req/sec)
pub fn default_weex_limiter() -> RateLimiter {
    RateLimiter::new(10, 10)
}

use std::time::Duration;
use crate::error::WeexError;

/// Retry configuration
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

/// Execute an async operation with exponential backoff retry
pub async fn with_retry<T, F, Fut>(config: &RetryConfig, mut operation: F) -> Result<T, WeexError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, WeexError>>,
{
    let mut attempts = 0;
    let mut delay = config.base_delay_ms;

    loop {
        attempts += 1;
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempts >= config.max_attempts {
                    return Err(e);
                }
                
                // Check if error is retryable
                if !is_retryable(&e) {
                    return Err(e);
                }

                // Exponential backoff with jitter
                let jitter = rand::random::<u64>() % 50;
                tokio::time::sleep(Duration::from_millis(delay + jitter)).await;
                delay = std::cmp::min(delay * 2, config.max_delay_ms);
            }
        }
    }
}

/// Determine if an error is retryable
fn is_retryable(error: &WeexError) -> bool {
    match error {
        WeexError::Http(_) => true, // Network errors are retryable
        WeexError::Api { code, .. } => {
            // Rate limit (429) or server error (5xx) are retryable
            code == "429" || code.starts_with("5")
        }
        _ => false,
    }
}

use crate::WeexClient;
use crate::error::WeexError;
use reqwest::Client;
use std::time::Duration;

#[derive(Default)]
pub struct WeexClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    secret_key: Option<String>,
    passphrase: Option<String>,
    timeout: Option<Duration>,
}

impl WeexClientBuilder {
    pub fn new() -> Self {
        WeexClientBuilder::default()
    }

    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }

    pub fn api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    pub fn secret_key(mut self, secret: &str) -> Self {
        self.secret_key = Some(secret.to_string());
        self
    }

    pub fn passphrase(mut self, phrase: &str) -> Self {
        self.passphrase = Some(phrase.to_string());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<WeexClient, WeexError> {
        let base_url = self.base_url.unwrap_or_else(|| "https://api.weex.com".to_string());
        let api_key = self.api_key.ok_or_else(|| WeexError::Unknown("API Key required".into()))?;
        let secret_key = self.secret_key.ok_or_else(|| WeexError::Unknown("Secret Key required".into()))?;
        let passphrase = self.passphrase.ok_or_else(|| WeexError::Unknown("Passphrase required".into()))?;

        let timeout = self.timeout.unwrap_or(Duration::from_secs(10));
        
        let client = Client::builder()
            .timeout(timeout)
            .user_agent("WeexRustSDK/0.4.0 (Professional)")
            .build()
            .map_err(WeexError::Http)?;

        Ok(WeexClient {
            base_url,
            api_key,
            secret_key,
            passphrase,
            client,
        })
    }
}

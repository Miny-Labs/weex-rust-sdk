use reqwest::Client;
use serde::Serialize;

/// Telegram bot configuration
#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
    pub enabled: bool,
}

impl TelegramConfig {
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        TelegramConfig {
            bot_token: bot_token.to_string(),
            chat_id: chat_id.to_string(),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        TelegramConfig {
            bot_token: String::new(),
            chat_id: String::new(),
            enabled: false,
        }
    }
}

/// Alert types
#[derive(Debug, Clone)]
pub enum AlertType {
    Trade,
    Position,
    Error,
    Info,
}

impl AlertType {
    fn emoji(&self) -> &str {
        match self {
            AlertType::Trade => "💰",
            AlertType::Position => "📊",
            AlertType::Error => "🚨",
            AlertType::Info => "ℹ️",
        }
    }
}

/// Telegram alerter
pub struct TelegramAlerter {
    config: TelegramConfig,
    client: Client,
}

impl TelegramAlerter {
    pub fn new(config: TelegramConfig) -> Self {
        TelegramAlerter {
            config,
            client: Client::new(),
        }
    }

    /// Send a message to Telegram
    pub async fn send(&self, alert_type: AlertType, message: &str) -> Result<(), reqwest::Error> {
        if !self.config.enabled {
            return Ok(());
        }

        let text = format!("{} {}", alert_type.emoji(), message);
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let params = [
            ("chat_id", self.config.chat_id.as_str()),
            ("text", text.as_str()),
            ("parse_mode", "HTML"),
        ];

        self.client.post(&url).form(&params).send().await?;
        Ok(())
    }

    /// Send trade notification
    pub async fn notify_trade(
        &self,
        symbol: &str,
        side: &str,
        size: f64,
        price: f64,
    ) -> Result<(), reqwest::Error> {
        let msg = format!(
            "<b>{}</b> {}\nSize: {:.4}\nPrice: ${:.2}",
            side.to_uppercase(),
            symbol,
            size,
            price
        );
        self.send(AlertType::Trade, &msg).await
    }

    /// Send position update
    pub async fn notify_position(
        &self,
        symbol: &str,
        size: f64,
        pnl: f64,
    ) -> Result<(), reqwest::Error> {
        let pnl_emoji = if pnl >= 0.0 { "🟢" } else { "🔴" };
        let msg = format!(
            "{} <b>{}</b>\nSize: {:.4}\nPnL: ${:.2}",
            pnl_emoji, symbol, size, pnl
        );
        self.send(AlertType::Position, &msg).await
    }

    /// Send error alert
    pub async fn notify_error(&self, error: &str) -> Result<(), reqwest::Error> {
        self.send(AlertType::Error, error).await
    }

    /// Send info message
    pub async fn notify_info(&self, message: &str) -> Result<(), reqwest::Error> {
        self.send(AlertType::Info, message).await
    }
}

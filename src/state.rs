use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use chrono::{DateTime, Utc};

/// Trade record for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub side: String,
    pub size: f64,
    pub price: f64,
    pub pnl: Option<f64>,
    pub status: String,
}

/// Session state for recovery
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    pub session_id: String,
    pub started_at: Option<DateTime<Utc>>,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub total_pnl: f64,
    pub open_positions: Vec<PositionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRecord {
    pub symbol: String,
    pub side: String,
    pub size: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
}

/// State manager for persistence
pub struct StateManager {
    trades_file: String,
    session_file: String,
}

impl StateManager {
    pub fn new(data_dir: &str) -> Self {
        // Create data directory if it doesn't exist
        let _ = std::fs::create_dir_all(data_dir);
        
        StateManager {
            trades_file: format!("{}/trades.json", data_dir),
            session_file: format!("{}/session.json", data_dir),
        }
    }

    /// Log a trade
    pub fn log_trade(&self, trade: &TradeRecord) -> Result<(), std::io::Error> {
        let mut trades = self.load_trades().unwrap_or_default();
        trades.push(trade.clone());
        self.save_trades(&trades)
    }

    /// Load all trades
    pub fn load_trades(&self) -> Result<Vec<TradeRecord>, std::io::Error> {
        if !Path::new(&self.trades_file).exists() {
            return Ok(Vec::new());
        }
        
        let file = File::open(&self.trades_file)?;
        let reader = BufReader::new(file);
        let trades: Vec<TradeRecord> = serde_json::from_reader(reader)
            .unwrap_or_default();
        Ok(trades)
    }

    /// Save trades
    fn save_trades(&self, trades: &[TradeRecord]) -> Result<(), std::io::Error> {
        let file = File::create(&self.trades_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, trades)?;
        Ok(())
    }

    /// Save session state
    pub fn save_session(&self, state: &SessionState) -> Result<(), std::io::Error> {
        let file = File::create(&self.session_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, state)?;
        Ok(())
    }

    /// Load session state
    pub fn load_session(&self) -> Result<SessionState, std::io::Error> {
        if !Path::new(&self.session_file).exists() {
            return Ok(SessionState::default());
        }
        
        let file = File::open(&self.session_file)?;
        let reader = BufReader::new(file);
        let state: SessionState = serde_json::from_reader(reader)
            .unwrap_or_default();
        Ok(state)
    }

    /// Calculate PnL statistics
    pub fn calculate_stats(&self) -> Result<PnlStats, std::io::Error> {
        let trades = self.load_trades()?;
        
        let mut stats = PnlStats::default();
        
        for trade in &trades {
            stats.total_trades += 1;
            if let Some(pnl) = trade.pnl {
                stats.total_pnl += pnl;
                if pnl > 0.0 {
                    stats.winning_trades += 1;
                    stats.gross_profit += pnl;
                } else {
                    stats.gross_loss += pnl.abs();
                }
            }
        }
        
        if stats.total_trades > 0 {
            stats.win_rate = stats.winning_trades as f64 / stats.total_trades as f64;
        }
        
        if stats.gross_loss > 0.0 {
            stats.profit_factor = stats.gross_profit / stats.gross_loss;
        }
        
        Ok(stats)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PnlStats {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub gross_profit: f64,
    pub gross_loss: f64,
    pub profit_factor: f64,
}

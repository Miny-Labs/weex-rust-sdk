use rust_decimal::Decimal;
use std::str::FromStr;

/// Risk management configuration
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// Maximum risk per trade as percentage of account (e.g., 0.02 = 2%)
    pub max_risk_per_trade: Decimal,
    /// Maximum position size in base currency
    pub max_position_size: Decimal,
    /// Maximum leverage allowed
    pub max_leverage: u32,
    /// Maximum number of concurrent positions
    pub max_positions: usize,
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            max_risk_per_trade: Decimal::from_str("0.02").unwrap(), // 2%
            max_position_size: Decimal::from_str("1.0").unwrap(),   // 1 BTC
            max_leverage: 10,
            max_positions: 3,
        }
    }
}

/// Position sizing calculator
pub struct PositionSizer {
    config: RiskConfig,
}

impl PositionSizer {
    pub fn new(config: RiskConfig) -> Self {
        PositionSizer { config }
    }

    /// Calculate position size based on fixed percentage risk
    /// 
    /// Formula: size = (account_balance * risk_percent) / (entry_price * stop_loss_percent)
    pub fn fixed_percentage(
        &self,
        account_balance: Decimal,
        entry_price: Decimal,
        stop_loss_percent: Decimal,
    ) -> Decimal {
        let risk_amount = account_balance * self.config.max_risk_per_trade;
        let stop_loss_amount = entry_price * stop_loss_percent;
        
        if stop_loss_amount.is_zero() {
            return Decimal::ZERO;
        }
        
        let size = risk_amount / stop_loss_amount;
        
        // Apply maximum position size limit
        if size > self.config.max_position_size {
            self.config.max_position_size
        } else {
            size
        }
    }

    /// Kelly Criterion position sizing
    /// 
    /// Formula: f* = (bp - q) / b
    /// where: b = win/loss ratio, p = win probability, q = loss probability
    pub fn kelly_criterion(
        &self,
        account_balance: Decimal,
        win_rate: Decimal,        // e.g., 0.55 = 55%
        avg_win: Decimal,         // average winning trade size
        avg_loss: Decimal,        // average losing trade size
    ) -> Decimal {
        if avg_loss.is_zero() {
            return Decimal::ZERO;
        }
        
        let b = avg_win / avg_loss;  // Win/loss ratio
        let p = win_rate;
        let q = Decimal::ONE - win_rate;
        
        // Kelly fraction: f* = (bp - q) / b
        let kelly_fraction = (b * p - q) / b;
        
        // Use half-Kelly for safety
        let safe_fraction = kelly_fraction / Decimal::from(2);
        
        if safe_fraction <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        let size = account_balance * safe_fraction;
        
        // Apply maximum limit
        if size > self.config.max_position_size {
            self.config.max_position_size
        } else {
            size
        }
    }

    /// Check if we can open a new position given current positions count
    pub fn can_open_position(&self, current_positions: usize) -> bool {
        current_positions < self.config.max_positions
    }

    /// Validate leverage is within limits
    pub fn validate_leverage(&self, leverage: u32) -> u32 {
        if leverage > self.config.max_leverage {
            self.config.max_leverage
        } else {
            leverage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_percentage() {
        let sizer = PositionSizer::new(RiskConfig::default());
        let size = sizer.fixed_percentage(
            Decimal::from(10000),  // $10k account
            Decimal::from(50000),  // $50k BTC
            Decimal::from_str("0.02").unwrap(), // 2% stop loss
        );
        // Risk = $10k * 2% = $200
        // Stop loss = $50k * 2% = $1000
        // Size = $200 / $1000 = 0.2 BTC
        assert_eq!(size, Decimal::from_str("0.2").unwrap());
    }
}

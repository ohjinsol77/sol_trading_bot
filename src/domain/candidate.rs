use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCandidate {
    pub trade_date: NaiveDate,
    pub stock_code: String,
    pub stock_name: String,
    pub supply_score: Decimal,
    pub liquidity_score: Decimal,
    pub trend_score: Decimal,
    pub risk_score: Decimal,
    pub final_score: Decimal,
    pub inst_buy_days: i32,
    pub foreign_buy_days: i32,
    pub inst_net_buy_amt_5d: i64,
    pub foreign_net_buy_amt_5d: i64,
    pub total_net_buy_amt_5d: i64,
    pub market_cap: Option<i64>,
    pub avg_trade_amount_20d: Option<i64>,
    pub prev_close: Decimal,
    pub prev_high: Decimal,
    pub prev_low: Decimal,
    pub ma5: Option<Decimal>,
    pub ma20: Option<Decimal>,
    pub breakout_price: Option<Decimal>,
    pub pullback_price: Option<Decimal>,
    pub stop_loss_price: Option<Decimal>,
    pub target_price: Option<Decimal>,
    pub strategy_type: StrategyType,
    pub status: CandidateStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrategyType {
    Opening,
    Pullback,
    Breakout,
}

impl fmt::Display for StrategyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyType::Opening => write!(f, "OPENING"),
            StrategyType::Pullback => write!(f, "PULLBACK"),
            StrategyType::Breakout => write!(f, "BREAKOUT"),
        }
    }
}

impl std::str::FromStr for StrategyType {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "OPENING" => Ok(Self::Opening),
            "PULLBACK" => Ok(Self::Pullback),
            "BREAKOUT" => Ok(Self::Breakout),
            _ => anyhow::bail!("unsupported strategy type: {value}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateStatus {
    Ready,
    Triggered,
    Ordered,
    Filled,
    Skipped,
    Expired,
}

impl fmt::Display for CandidateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CandidateStatus::Ready => write!(f, "READY"),
            CandidateStatus::Triggered => write!(f, "TRIGGERED"),
            CandidateStatus::Ordered => write!(f, "ORDERED"),
            CandidateStatus::Filled => write!(f, "FILLED"),
            CandidateStatus::Skipped => write!(f, "SKIPPED"),
            CandidateStatus::Expired => write!(f, "EXPIRED"),
        }
    }
}

impl std::str::FromStr for CandidateStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "READY" => Ok(Self::Ready),
            "TRIGGERED" => Ok(Self::Triggered),
            "ORDERED" => Ok(Self::Ordered),
            "FILLED" => Ok(Self::Filled),
            "SKIPPED" => Ok(Self::Skipped),
            "EXPIRED" => Ok(Self::Expired),
            _ => anyhow::bail!("unsupported candidate status: {value}"),
        }
    }
}

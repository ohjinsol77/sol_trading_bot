use chrono::{NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::StrategyType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub signal_date: NaiveDate,
    pub signal_time: NaiveTime,
    pub stock_code: String,
    pub stock_name: String,
    pub strategy_type: StrategyType,
    pub signal_price: Decimal,
    pub final_score: Decimal,
    pub reason: String,
}

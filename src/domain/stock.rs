use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBasic {
    pub stock_code: String,
    pub stock_name: String,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub is_preferred: bool,
    pub is_etf: bool,
    pub is_etn: bool,
    pub is_spac: bool,
    pub is_trading_halted: bool,
    pub is_administrative_issue: bool,
    pub is_warning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialRatio {
    pub stock_code: String,
    pub debt_ratio: Option<Decimal>,
    pub operating_profit: Option<i64>,
    pub quarterly_operating_profit: Option<i64>,
    pub capital_impairment: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMetricDaily {
    pub trade_date: chrono::NaiveDate,
    pub stock_code: String,
    pub market_cap: Option<i64>,
    pub credit_balance: Option<i64>,
    pub short_selling_amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentQuote {
    pub stock_code: String,
    pub stock_name: String,
    pub current_price: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub prev_close: Decimal,
    pub accumulated_trade_amount: i64,
    pub expected_trade_amount_ratio: Decimal,
    pub execution_strength: Decimal,
    pub bid_price: Decimal,
    pub ask_price: Decimal,
    pub vwap: Option<Decimal>,
    pub is_vi_expected: bool,
}

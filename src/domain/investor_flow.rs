use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestorFlowDaily {
    pub trade_date: NaiveDate,
    pub stock_code: String,
    pub institution_net_buy_amt: i64,
    pub foreign_net_buy_amt: i64,
    pub individual_net_buy_amt: i64,
    pub institution_net_buy_qty: Option<i64>,
    pub foreign_net_buy_qty: Option<i64>,
    pub individual_net_buy_qty: Option<i64>,
}

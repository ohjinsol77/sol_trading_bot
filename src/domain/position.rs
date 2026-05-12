use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub stock_code: String,
    pub stock_name: String,
    pub entry_date: NaiveDate,
    pub entry_price: Decimal,
    pub qty: i64,
    pub highest_price_after_buy: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellPlan {
    pub stop_loss_price: Decimal,
    pub target_price_1: Decimal,
    pub target_price_2: Decimal,
    pub trailing_stop_price: Decimal,
}

pub fn build_sell_plan(buy_price: Decimal, highest_price_after_buy: Decimal) -> SellPlan {
    SellPlan {
        stop_loss_price: buy_price * Decimal::new(98, 2),
        target_price_1: buy_price * Decimal::new(103, 2),
        target_price_2: buy_price * Decimal::new(105, 2),
        trailing_stop_price: highest_price_after_buy * Decimal::new(98, 2),
    }
}

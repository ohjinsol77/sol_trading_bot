use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub stock_code: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Decimal>,
    pub qty: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub kis_order_no: Option<String>,
    pub status: OrderStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyOrderRequest {
    pub trade_date: NaiveDate,
    pub stock_code: String,
    pub stock_name: String,
    pub price: Decimal,
    pub qty: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellOrderRequest {
    pub trade_date: NaiveDate,
    pub stock_code: String,
    pub stock_name: String,
    pub price: Decimal,
    pub qty: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExecutionResult {
    pub order_id: Option<i64>,
    pub kis_order_no: Option<String>,
    pub status: OrderStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "MARKET"),
            OrderType::Limit => write!(f, "LIMIT"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Requested,
    DryRun,
    Submitted,
    Filled,
    Rejected,
    Cancelled,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Requested => write!(f, "REQUESTED"),
            OrderStatus::DryRun => write!(f, "DRY_RUN"),
            OrderStatus::Submitted => write!(f, "SUBMITTED"),
            OrderStatus::Filled => write!(f, "FILLED"),
            OrderStatus::Rejected => write!(f, "REJECTED"),
            OrderStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

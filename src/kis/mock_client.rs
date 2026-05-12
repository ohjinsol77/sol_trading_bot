use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use rust_decimal::Decimal;

use crate::domain::*;

use super::client::KisClient;

#[derive(Debug, Default)]
pub struct MockKisClient;

impl MockKisClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl KisClient for MockKisClient {
    async fn issue_access_token(&self) -> anyhow::Result<String> {
        Ok("mock-token".to_string())
    }

    async fn fetch_stock_universe(&self) -> anyhow::Result<Vec<StockBasic>> {
        Ok(mock_stocks())
    }

    async fn fetch_investor_flow_daily(
        &self,
        stock_code: &str,
        _from: NaiveDate,
        to: NaiveDate,
    ) -> anyhow::Result<Vec<InvestorFlowDaily>> {
        Ok((0..5)
            .rev()
            .map(|offset| {
                let trade_date = to - Duration::days(offset);
                let (inst, foreign) = match stock_code {
                    "005930" => (8_400_000_000, 6_200_000_000),
                    "000660" => (7_600_000_000, 2_400_000_000),
                    "035420" => (2_200_000_000, 1_400_000_000),
                    "035720" => {
                        if offset < 2 {
                            (900_000_000, -300_000_000)
                        } else {
                            (-700_000_000, 500_000_000)
                        }
                    }
                    "005380" => (1_600_000_000, 1_100_000_000),
                    _ => (0, 0),
                };
                InvestorFlowDaily {
                    trade_date,
                    stock_code: stock_code.to_string(),
                    institution_net_buy_amt: inst,
                    foreign_net_buy_amt: foreign,
                    individual_net_buy_amt: -(inst + foreign),
                    institution_net_buy_qty: Some(inst / 100_000),
                    foreign_net_buy_qty: Some(foreign / 100_000),
                    individual_net_buy_qty: Some(-(inst + foreign) / 100_000),
                }
            })
            .collect())
    }

    async fn fetch_ohlcv_daily(
        &self,
        stock_code: &str,
        _from: NaiveDate,
        to: NaiveDate,
    ) -> anyhow::Result<Vec<OhlcvDaily>> {
        let base = match stock_code {
            "005930" => 78_000,
            "000660" => 176_000,
            "035420" => 192_000,
            "035720" => 52_000,
            "005380" => 241_000,
            _ => 10_000,
        };
        let trade_amount = match stock_code {
            "035720" => 2_000_000_000,
            "035420" => 8_000_000_000,
            _ => 20_000_000_000,
        };
        Ok((0..20)
            .rev()
            .map(|offset| {
                let idx = 19 - offset;
                let close = Decimal::from(base + idx * 180);
                let open = close - Decimal::from(250);
                let high = close + Decimal::from(700);
                let low = close - Decimal::from(900);
                OhlcvDaily {
                    trade_date: to - Duration::days(offset as i64),
                    stock_code: stock_code.to_string(),
                    open_price: open,
                    high_price: high,
                    low_price: low,
                    close_price: close,
                    volume: 1_000_000 + idx as i64 * 20_000,
                    trade_amount: trade_amount + idx as i64 * 120_000_000,
                }
            })
            .collect())
    }

    async fn fetch_stock_basic(&self, stock_code: &str) -> anyhow::Result<StockBasic> {
        mock_stocks()
            .into_iter()
            .find(|stock| stock.stock_code == stock_code)
            .ok_or_else(|| anyhow::anyhow!("mock stock not found: {stock_code}"))
    }

    async fn fetch_financial_ratio(&self, stock_code: &str) -> anyhow::Result<Option<FinancialRatio>> {
        let ratio = FinancialRatio {
            stock_code: stock_code.to_string(),
            debt_ratio: Some(match stock_code {
                "035720" => Decimal::from(320),
                _ => Decimal::from(80),
            }),
            operating_profit: Some(match stock_code {
                "035720" => -100_000_000_000,
                _ => 1_000_000_000_000,
            }),
            quarterly_operating_profit: Some(match stock_code {
                "035720" => -10_000_000_000,
                _ => 100_000_000_000,
            }),
            capital_impairment: Some(false),
        };
        Ok(Some(ratio))
    }

    async fn fetch_market_cap(&self, stock_code: &str, _date: NaiveDate) -> anyhow::Result<Option<i64>> {
        Ok(Some(match stock_code {
            "005930" => 470_000_000_000_000,
            "000660" => 120_000_000_000_000,
            "035420" => 31_000_000_000_000,
            "035720" => 21_000_000_000_000,
            "005380" => 49_000_000_000_000,
            _ => 50_000_000_000,
        }))
    }

    async fn fetch_current_quote(&self, stock_code: &str) -> anyhow::Result<CurrentQuote> {
        let (name, prev_close, current, prev_high, strength, ratio, vwap) = match stock_code {
            "005930" => ("삼성전자", 81_200, 82_250, 82_000, 124, 160, 81_900),
            "000660" => ("SK하이닉스", 179_400, 178_900, 181_000, 106, 90, 178_500),
            "035420" => ("NAVER", 195_000, 194_700, 197_000, 101, 80, 194_400),
            "005380" => ("현대차", 244_000, 245_500, 247_000, 112, 120, 245_200),
            _ => ("UNKNOWN", 10_000, 10_000, 10_200, 80, 50, 10_000),
        };
        Ok(CurrentQuote {
            stock_code: stock_code.to_string(),
            stock_name: name.to_string(),
            current_price: Decimal::from(current),
            open_price: Decimal::from(prev_close),
            high_price: Decimal::from(current + 300),
            low_price: Decimal::from(current - 900),
            prev_close: Decimal::from(prev_close),
            accumulated_trade_amount: 30_000_000_000,
            expected_trade_amount_ratio: Decimal::from(ratio),
            execution_strength: Decimal::from(strength),
            bid_price: Decimal::from(current - 40),
            ask_price: Decimal::from(current + 40),
            vwap: Some(Decimal::from(vwap)),
            is_vi_expected: false,
        })
    }

    async fn submit_order(&self, _request: OrderRequest) -> anyhow::Result<OrderResponse> {
        Ok(OrderResponse {
            kis_order_no: None,
            status: OrderStatus::DryRun,
            message: "mock order accepted as dry-run".to_string(),
        })
    }
}

fn mock_stocks() -> Vec<StockBasic> {
    vec![
        stock("005930", "삼성전자", "KOSPI", "반도체"),
        stock("000660", "SK하이닉스", "KOSPI", "반도체"),
        stock("035420", "NAVER", "KOSPI", "인터넷"),
        stock("035720", "카카오", "KOSPI", "인터넷"),
        stock("005380", "현대차", "KOSPI", "자동차"),
    ]
}

fn stock(code: &str, name: &str, market: &str, sector: &str) -> StockBasic {
    StockBasic {
        stock_code: code.to_string(),
        stock_name: name.to_string(),
        market: Some(market.to_string()),
        sector: Some(sector.to_string()),
        is_preferred: false,
        is_etf: false,
        is_etn: false,
        is_spac: false,
        is_trading_halted: false,
        is_administrative_issue: false,
        is_warning: false,
    }
}

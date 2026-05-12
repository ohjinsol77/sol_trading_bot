use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::*;

#[async_trait]
pub trait Repository: Send + Sync {
    async fn save_stock(&self, stock: &StockBasic) -> anyhow::Result<()>;
    async fn save_ohlcv_daily(&self, rows: &[OhlcvDaily]) -> anyhow::Result<()>;
    async fn save_investor_flow_daily(&self, rows: &[InvestorFlowDaily]) -> anyhow::Result<()>;
    async fn save_financial_ratio(&self, ratio: &FinancialRatio) -> anyhow::Result<()>;
    async fn save_market_metric(&self, metric: &MarketMetricDaily) -> anyhow::Result<()>;
    async fn save_candidate(&self, candidate: &TradeCandidate) -> anyhow::Result<()>;
    async fn save_signal(&self, signal: &TradeSignal) -> anyhow::Result<i64>;
    async fn create_dry_run_order(&self, request: &BuyOrderRequest) -> anyhow::Result<i64>;
    async fn list_stocks(&self) -> anyhow::Result<Vec<StockBasic>>;
    async fn list_ohlcv(&self, stock_code: &str, to: NaiveDate, limit: i64) -> anyhow::Result<Vec<OhlcvDaily>>;
    async fn list_flows(&self, stock_code: &str, to: NaiveDate, limit: i64) -> anyhow::Result<Vec<InvestorFlowDaily>>;
    async fn get_financial_ratio(&self, stock_code: &str) -> anyhow::Result<Option<FinancialRatio>>;
    async fn get_market_cap(&self, stock_code: &str, date: NaiveDate) -> anyhow::Result<Option<i64>>;
    async fn list_ready_candidates(&self, trade_date: NaiveDate) -> anyhow::Result<Vec<TradeCandidate>>;
    async fn list_candidates(&self, trade_date: NaiveDate) -> anyhow::Result<Vec<TradeCandidate>>;
    async fn count_signals(&self, signal_date: NaiveDate) -> anyhow::Result<i64>;
    async fn has_signal_for_stock(&self, signal_date: NaiveDate, stock_code: &str) -> anyhow::Result<bool>;
    async fn update_candidate_status(
        &self,
        trade_date: NaiveDate,
        stock_code: &str,
        strategy_type: StrategyType,
        status: CandidateStatus,
    ) -> anyhow::Result<()>;
}

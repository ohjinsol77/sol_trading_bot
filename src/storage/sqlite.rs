use std::{fs, path::Path, str::FromStr};

use anyhow::{Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};

use crate::domain::*;

#[derive(Debug, Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn connect(database_url: &str) -> Result<Self> {
        ensure_sqlite_parent(database_url)?;
        let options = SqliteConnectOptions::from_str(database_url)
            .with_context(|| format!("invalid sqlite url: {database_url}"))?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("connect sqlite")?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        let migration = include_str!("../../migrations/001_init.sql");
        for statement in migration.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn save_stock(&self, stock: &StockBasic) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stock_master (
                stock_code, stock_name, market, sector, is_preferred, is_etf, is_etn,
                is_spac, is_trading_halted, is_administrative_issue, is_warning, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(stock_code) DO UPDATE SET
                stock_name = excluded.stock_name,
                market = excluded.market,
                sector = excluded.sector,
                is_preferred = excluded.is_preferred,
                is_etf = excluded.is_etf,
                is_etn = excluded.is_etn,
                is_spac = excluded.is_spac,
                is_trading_halted = excluded.is_trading_halted,
                is_administrative_issue = excluded.is_administrative_issue,
                is_warning = excluded.is_warning,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&stock.stock_code)
        .bind(&stock.stock_name)
        .bind(&stock.market)
        .bind(&stock.sector)
        .bind(stock.is_preferred as i64)
        .bind(stock.is_etf as i64)
        .bind(stock.is_etn as i64)
        .bind(stock.is_spac as i64)
        .bind(stock.is_trading_halted as i64)
        .bind(stock.is_administrative_issue as i64)
        .bind(stock.is_warning as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_ohlcv_daily(&self, rows: &[OhlcvDaily]) -> Result<()> {
        for row in rows {
            sqlx::query(
                r#"
                INSERT INTO stock_ohlcv_daily (
                    trade_date, stock_code, open_price, high_price, low_price,
                    close_price, volume, trade_amount
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(trade_date, stock_code) DO UPDATE SET
                    open_price = excluded.open_price,
                    high_price = excluded.high_price,
                    low_price = excluded.low_price,
                    close_price = excluded.close_price,
                    volume = excluded.volume,
                    trade_amount = excluded.trade_amount
                "#,
            )
            .bind(row.trade_date.to_string())
            .bind(&row.stock_code)
            .bind(row.open_price.to_string())
            .bind(row.high_price.to_string())
            .bind(row.low_price.to_string())
            .bind(row.close_price.to_string())
            .bind(row.volume)
            .bind(row.trade_amount)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn save_investor_flow_daily(&self, rows: &[InvestorFlowDaily]) -> Result<()> {
        for row in rows {
            sqlx::query(
                r#"
                INSERT INTO stock_investor_flow_daily (
                    trade_date, stock_code, institution_net_buy_amt, foreign_net_buy_amt,
                    individual_net_buy_amt, institution_net_buy_qty, foreign_net_buy_qty,
                    individual_net_buy_qty
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(trade_date, stock_code) DO UPDATE SET
                    institution_net_buy_amt = excluded.institution_net_buy_amt,
                    foreign_net_buy_amt = excluded.foreign_net_buy_amt,
                    individual_net_buy_amt = excluded.individual_net_buy_amt,
                    institution_net_buy_qty = excluded.institution_net_buy_qty,
                    foreign_net_buy_qty = excluded.foreign_net_buy_qty,
                    individual_net_buy_qty = excluded.individual_net_buy_qty
                "#,
            )
            .bind(row.trade_date.to_string())
            .bind(&row.stock_code)
            .bind(row.institution_net_buy_amt)
            .bind(row.foreign_net_buy_amt)
            .bind(row.individual_net_buy_amt)
            .bind(row.institution_net_buy_qty)
            .bind(row.foreign_net_buy_qty)
            .bind(row.individual_net_buy_qty)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn save_financial_ratio(&self, ratio: &FinancialRatio) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stock_financial_ratio (
                stock_code, debt_ratio, operating_profit, quarterly_operating_profit,
                capital_impairment, updated_at
            )
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(stock_code) DO UPDATE SET
                debt_ratio = excluded.debt_ratio,
                operating_profit = excluded.operating_profit,
                quarterly_operating_profit = excluded.quarterly_operating_profit,
                capital_impairment = excluded.capital_impairment,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&ratio.stock_code)
        .bind(ratio.debt_ratio.map(|v| v.to_string()))
        .bind(ratio.operating_profit)
        .bind(ratio.quarterly_operating_profit)
        .bind(ratio.capital_impairment.map(|v| v as i64))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_market_metric(&self, metric: &MarketMetricDaily) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stock_market_metric_daily (
                trade_date, stock_code, market_cap, credit_balance, short_selling_amount
            )
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(trade_date, stock_code) DO UPDATE SET
                market_cap = excluded.market_cap,
                credit_balance = excluded.credit_balance,
                short_selling_amount = excluded.short_selling_amount
            "#,
        )
        .bind(metric.trade_date.to_string())
        .bind(&metric.stock_code)
        .bind(metric.market_cap)
        .bind(metric.credit_balance)
        .bind(metric.short_selling_amount)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_candidate(&self, candidate: &TradeCandidate) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO trade_candidate_daily (
                trade_date, stock_code, stock_name, supply_score, liquidity_score,
                trend_score, risk_score, final_score, inst_buy_days, foreign_buy_days,
                inst_net_buy_amt_5d, foreign_net_buy_amt_5d, total_net_buy_amt_5d,
                market_cap, avg_trade_amount_20d, prev_close, prev_high, prev_low,
                ma5, ma20, breakout_price, pullback_price, stop_loss_price, target_price,
                strategy_type, status
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(trade_date, stock_code, strategy_type) DO UPDATE SET
                stock_name = excluded.stock_name,
                supply_score = excluded.supply_score,
                liquidity_score = excluded.liquidity_score,
                trend_score = excluded.trend_score,
                risk_score = excluded.risk_score,
                final_score = excluded.final_score,
                status = excluded.status
            "#,
        )
        .bind(candidate.trade_date.to_string())
        .bind(&candidate.stock_code)
        .bind(&candidate.stock_name)
        .bind(candidate.supply_score.to_string())
        .bind(candidate.liquidity_score.to_string())
        .bind(candidate.trend_score.to_string())
        .bind(candidate.risk_score.to_string())
        .bind(candidate.final_score.to_string())
        .bind(candidate.inst_buy_days)
        .bind(candidate.foreign_buy_days)
        .bind(candidate.inst_net_buy_amt_5d)
        .bind(candidate.foreign_net_buy_amt_5d)
        .bind(candidate.total_net_buy_amt_5d)
        .bind(candidate.market_cap)
        .bind(candidate.avg_trade_amount_20d)
        .bind(candidate.prev_close.to_string())
        .bind(candidate.prev_high.to_string())
        .bind(candidate.prev_low.to_string())
        .bind(candidate.ma5.map(|v| v.to_string()))
        .bind(candidate.ma20.map(|v| v.to_string()))
        .bind(candidate.breakout_price.map(|v| v.to_string()))
        .bind(candidate.pullback_price.map(|v| v.to_string()))
        .bind(candidate.stop_loss_price.map(|v| v.to_string()))
        .bind(candidate.target_price.map(|v| v.to_string()))
        .bind(candidate.strategy_type.to_string())
        .bind(candidate.status.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_signal(&self, signal: &TradeSignal) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO trade_signal (
                signal_date, signal_time, stock_code, stock_name, strategy_type,
                signal_price, final_score, reason
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(signal.signal_date.to_string())
        .bind(signal.signal_time.to_string())
        .bind(&signal.stock_code)
        .bind(&signal.stock_name)
        .bind(signal.strategy_type.to_string())
        .bind(signal.signal_price.to_string())
        .bind(signal.final_score.to_string())
        .bind(&signal.reason)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn create_dry_run_order(&self, request: &BuyOrderRequest) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO trade_order (
                trade_date, stock_code, stock_name, side, order_type,
                order_price, order_qty, status, reason, updated_at
            )
            VALUES (?, ?, ?, 'BUY', 'LIMIT', ?, ?, 'DRY_RUN', ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(request.trade_date.to_string())
        .bind(&request.stock_code)
        .bind(&request.stock_name)
        .bind(request.price.to_string())
        .bind(request.qty)
        .bind(&request.reason)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn list_stocks(&self) -> Result<Vec<StockBasic>> {
        let rows = sqlx::query("SELECT * FROM stock_master ORDER BY stock_code")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(stock_from_row).collect()
    }

    pub async fn list_ohlcv(&self, stock_code: &str, to: NaiveDate, limit: i64) -> Result<Vec<OhlcvDaily>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM stock_ohlcv_daily
            WHERE stock_code = ? AND trade_date <= ?
            ORDER BY trade_date DESC
            LIMIT ?
            "#,
        )
        .bind(stock_code)
        .bind(to.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        let mut rows = rows
            .into_iter()
            .map(ohlcv_from_row)
            .collect::<Result<Vec<_>>>()?;
        rows.sort_by_key(|row| row.trade_date);
        Ok(rows)
    }

    pub async fn list_flows(&self, stock_code: &str, to: NaiveDate, limit: i64) -> Result<Vec<InvestorFlowDaily>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM stock_investor_flow_daily
            WHERE stock_code = ? AND trade_date <= ?
            ORDER BY trade_date DESC
            LIMIT ?
            "#,
        )
        .bind(stock_code)
        .bind(to.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        let mut rows = rows
            .into_iter()
            .map(flow_from_row)
            .collect::<Result<Vec<_>>>()?;
        rows.sort_by_key(|row| row.trade_date);
        Ok(rows)
    }

    pub async fn get_financial_ratio(&self, stock_code: &str) -> Result<Option<FinancialRatio>> {
        let row = sqlx::query("SELECT * FROM stock_financial_ratio WHERE stock_code = ?")
            .bind(stock_code)
            .fetch_optional(&self.pool)
            .await?;
        row.map(financial_from_row).transpose()
    }

    pub async fn get_market_cap(&self, stock_code: &str, date: NaiveDate) -> Result<Option<i64>> {
        let row = sqlx::query(
            "SELECT market_cap FROM stock_market_metric_daily WHERE stock_code = ? AND trade_date = ?",
        )
        .bind(stock_code)
        .bind(date.to_string())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.and_then(|r| r.try_get::<Option<i64>, _>("market_cap").ok().flatten()))
    }

    pub async fn list_ready_candidates(&self, trade_date: NaiveDate) -> Result<Vec<TradeCandidate>> {
        self.list_candidates_by_status(trade_date, Some(CandidateStatus::Ready)).await
    }

    pub async fn list_candidates(&self, trade_date: NaiveDate) -> Result<Vec<TradeCandidate>> {
        self.list_candidates_by_status(trade_date, None).await
    }

    async fn list_candidates_by_status(
        &self,
        trade_date: NaiveDate,
        status: Option<CandidateStatus>,
    ) -> Result<Vec<TradeCandidate>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                r#"
                SELECT * FROM trade_candidate_daily
                WHERE trade_date = ? AND status = ?
                ORDER BY CAST(final_score AS REAL) DESC
                "#,
            )
            .bind(trade_date.to_string())
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT * FROM trade_candidate_daily
                WHERE trade_date = ?
                ORDER BY CAST(final_score AS REAL) DESC
                "#,
            )
            .bind(trade_date.to_string())
            .fetch_all(&self.pool)
            .await?
        };
        rows.into_iter().map(candidate_from_row).collect()
    }

    pub async fn count_signals(&self, signal_date: NaiveDate) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) AS cnt FROM trade_signal WHERE signal_date = ?")
            .bind(signal_date.to_string())
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get("cnt")?)
    }

    pub async fn has_signal_for_stock(&self, signal_date: NaiveDate, stock_code: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS cnt FROM trade_signal WHERE signal_date = ? AND stock_code = ?",
        )
        .bind(signal_date.to_string())
        .bind(stock_code)
        .fetch_one(&self.pool)
        .await?;
        let count: i64 = row.try_get("cnt")?;
        Ok(count > 0)
    }

    pub async fn update_candidate_status(
        &self,
        trade_date: NaiveDate,
        stock_code: &str,
        strategy_type: StrategyType,
        status: CandidateStatus,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE trade_candidate_daily
            SET status = ?
            WHERE trade_date = ? AND stock_code = ? AND strategy_type = ?
            "#,
        )
        .bind(status.to_string())
        .bind(trade_date.to_string())
        .bind(stock_code)
        .bind(strategy_type.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn ensure_sqlite_parent(database_url: &str) -> Result<()> {
    let path = database_url.strip_prefix("sqlite://").unwrap_or(database_url);
    if path == ":memory:" {
        return Ok(());
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn parse_date(value: String) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(&value, "%Y-%m-%d")?)
}

fn parse_decimal(value: String) -> Result<Decimal> {
    Decimal::from_str(&value).with_context(|| format!("parse decimal: {value}"))
}

fn parse_opt_decimal(value: Option<String>) -> Result<Option<Decimal>> {
    value.map(parse_decimal).transpose()
}

fn stock_from_row(row: sqlx::sqlite::SqliteRow) -> Result<StockBasic> {
    Ok(StockBasic {
        stock_code: row.try_get("stock_code")?,
        stock_name: row.try_get("stock_name")?,
        market: row.try_get("market")?,
        sector: row.try_get("sector")?,
        is_preferred: row.try_get::<i64, _>("is_preferred")? != 0,
        is_etf: row.try_get::<i64, _>("is_etf")? != 0,
        is_etn: row.try_get::<i64, _>("is_etn")? != 0,
        is_spac: row.try_get::<i64, _>("is_spac")? != 0,
        is_trading_halted: row.try_get::<i64, _>("is_trading_halted")? != 0,
        is_administrative_issue: row.try_get::<i64, _>("is_administrative_issue")? != 0,
        is_warning: row.try_get::<i64, _>("is_warning")? != 0,
    })
}

fn ohlcv_from_row(row: sqlx::sqlite::SqliteRow) -> Result<OhlcvDaily> {
    Ok(OhlcvDaily {
        trade_date: parse_date(row.try_get("trade_date")?)?,
        stock_code: row.try_get("stock_code")?,
        open_price: parse_decimal(row.try_get("open_price")?)?,
        high_price: parse_decimal(row.try_get("high_price")?)?,
        low_price: parse_decimal(row.try_get("low_price")?)?,
        close_price: parse_decimal(row.try_get("close_price")?)?,
        volume: row.try_get("volume")?,
        trade_amount: row.try_get("trade_amount")?,
    })
}

fn flow_from_row(row: sqlx::sqlite::SqliteRow) -> Result<InvestorFlowDaily> {
    Ok(InvestorFlowDaily {
        trade_date: parse_date(row.try_get("trade_date")?)?,
        stock_code: row.try_get("stock_code")?,
        institution_net_buy_amt: row.try_get("institution_net_buy_amt")?,
        foreign_net_buy_amt: row.try_get("foreign_net_buy_amt")?,
        individual_net_buy_amt: row.try_get("individual_net_buy_amt")?,
        institution_net_buy_qty: row.try_get("institution_net_buy_qty")?,
        foreign_net_buy_qty: row.try_get("foreign_net_buy_qty")?,
        individual_net_buy_qty: row.try_get("individual_net_buy_qty")?,
    })
}

fn financial_from_row(row: sqlx::sqlite::SqliteRow) -> Result<FinancialRatio> {
    Ok(FinancialRatio {
        stock_code: row.try_get("stock_code")?,
        debt_ratio: parse_opt_decimal(row.try_get("debt_ratio")?)?,
        operating_profit: row.try_get("operating_profit")?,
        quarterly_operating_profit: row.try_get("quarterly_operating_profit")?,
        capital_impairment: row
            .try_get::<Option<i64>, _>("capital_impairment")?
            .map(|value| value != 0),
    })
}

fn candidate_from_row(row: sqlx::sqlite::SqliteRow) -> Result<TradeCandidate> {
    let strategy_type: String = row.try_get("strategy_type")?;
    let status: String = row.try_get("status")?;
    Ok(TradeCandidate {
        trade_date: parse_date(row.try_get("trade_date")?)?,
        stock_code: row.try_get("stock_code")?,
        stock_name: row.try_get("stock_name")?,
        supply_score: parse_decimal(row.try_get("supply_score")?)?,
        liquidity_score: parse_decimal(row.try_get("liquidity_score")?)?,
        trend_score: parse_decimal(row.try_get("trend_score")?)?,
        risk_score: parse_decimal(row.try_get("risk_score")?)?,
        final_score: parse_decimal(row.try_get("final_score")?)?,
        inst_buy_days: row.try_get("inst_buy_days")?,
        foreign_buy_days: row.try_get("foreign_buy_days")?,
        inst_net_buy_amt_5d: row.try_get("inst_net_buy_amt_5d")?,
        foreign_net_buy_amt_5d: row.try_get("foreign_net_buy_amt_5d")?,
        total_net_buy_amt_5d: row.try_get("total_net_buy_amt_5d")?,
        market_cap: row.try_get("market_cap")?,
        avg_trade_amount_20d: row.try_get("avg_trade_amount_20d")?,
        prev_close: parse_decimal(row.try_get("prev_close")?)?,
        prev_high: parse_decimal(row.try_get("prev_high")?)?,
        prev_low: parse_decimal(row.try_get("prev_low")?)?,
        ma5: parse_opt_decimal(row.try_get("ma5")?)?,
        ma20: parse_opt_decimal(row.try_get("ma20")?)?,
        breakout_price: parse_opt_decimal(row.try_get("breakout_price")?)?,
        pullback_price: parse_opt_decimal(row.try_get("pullback_price")?)?,
        stop_loss_price: parse_opt_decimal(row.try_get("stop_loss_price")?)?,
        target_price: parse_opt_decimal(row.try_get("target_price")?)?,
        strategy_type: strategy_type.parse()?,
        status: status.parse()?,
    })
}

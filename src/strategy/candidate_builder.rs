use chrono::NaiveDate;
use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::{
    config::AppConfig,
    domain::{CandidateStatus, StrategyType, TradeCandidate},
    storage::SqliteRepository,
};

use super::{filters, scoring};

pub async fn build_candidates(
    date: NaiveDate,
    config: &AppConfig,
    repository: &SqliteRepository,
) -> anyhow::Result<Vec<TradeCandidate>> {
    let stocks = repository.list_stocks().await?;
    let mut candidates = Vec::new();

    for stock in stocks {
        let ohlcv = repository.list_ohlcv(&stock.stock_code, date, 20).await?;
        let flows = repository
            .list_flows(&stock.stock_code, date, config.strategy.lookback_days as i64)
            .await?;
        let financial = repository.get_financial_ratio(&stock.stock_code).await?;
        let market_cap = repository.get_market_cap(&stock.stock_code, date).await?;

        if let Err(reason) = filters::passes_candidate_filters(
            &stock,
            &flows,
            &ohlcv,
            financial.as_ref(),
            market_cap,
            &config.strategy,
        ) {
            warn!(stock_code = %stock.stock_code, reason, "candidate skipped");
            continue;
        }

        let inst_buy_days = scoring::count_institution_buy_days(&flows);
        let foreign_buy_days = scoring::count_foreign_buy_days(&flows);
        let inst_sum = scoring::sum_institution_net_buy(&flows);
        let foreign_sum = scoring::sum_foreign_net_buy(&flows);
        let total_sum = inst_sum + foreign_sum;
        let ma5 = scoring::moving_average(&ohlcv, 5);
        let ma20 = scoring::moving_average(&ohlcv, 20);
        let Some(latest) = ohlcv.last() else {
            warn!(stock_code = %stock.stock_code, "candidate skipped after empty ohlcv");
            continue;
        };
        let previous = ohlcv.get(ohlcv.len().saturating_sub(2));

        let supply_score = scoring::supply_score(inst_buy_days, foreign_buy_days, total_sum, market_cap);
        let liquidity_score = scoring::liquidity_score(&ohlcv);
        let trend_score = scoring::trend_score(&ohlcv, ma5, ma20);
        let risk_score = scoring::risk_score(financial.as_ref(), Some(latest), previous);
        let final_score = scoring::final_score(supply_score, liquidity_score, trend_score, risk_score).round_dp(2);

        let avg_trade_amount_20d = scoring::average_trade_amount(&ohlcv, 20);
        let pullback_price = ma5.map(|ma| max_decimal(ma, latest.close_price * Decimal::new(985, 3)));
        let breakout_price = Some(latest.high_price * Decimal::new(1001, 3));
        let stop_loss_price = Some(latest.close_price * Decimal::new(98, 2));
        let target_price = Some(latest.close_price * Decimal::new(103, 2));

        let mut built = Vec::new();
        if config.strategy.enable_breakout_strategy {
            built.push(candidate(
                date,
                &stock,
                supply_score,
                liquidity_score,
                trend_score,
                risk_score,
                final_score,
                inst_buy_days,
                foreign_buy_days,
                inst_sum,
                foreign_sum,
                total_sum,
                market_cap,
                avg_trade_amount_20d,
                latest,
                ma5,
                ma20,
                breakout_price,
                pullback_price,
                stop_loss_price,
                target_price,
                StrategyType::Breakout,
            ));
        }
        if config.strategy.enable_pullback_strategy {
            built.push(candidate(
                date,
                &stock,
                supply_score,
                liquidity_score,
                trend_score,
                risk_score,
                final_score,
                inst_buy_days,
                foreign_buy_days,
                inst_sum,
                foreign_sum,
                total_sum,
                market_cap,
                avg_trade_amount_20d,
                latest,
                ma5,
                ma20,
                breakout_price,
                pullback_price,
                stop_loss_price,
                target_price,
                StrategyType::Pullback,
            ));
        }
        if config.strategy.enable_opening_strategy {
            built.push(candidate(
                date,
                &stock,
                supply_score,
                liquidity_score,
                trend_score,
                risk_score,
                final_score,
                inst_buy_days,
                foreign_buy_days,
                inst_sum,
                foreign_sum,
                total_sum,
                market_cap,
                avg_trade_amount_20d,
                latest,
                ma5,
                ma20,
                breakout_price,
                pullback_price,
                stop_loss_price,
                target_price,
                StrategyType::Opening,
            ));
        }

        for candidate in built {
            repository.save_candidate(&candidate).await?;
            info!(stock_code = %candidate.stock_code, strategy = %candidate.strategy_type, final_score = %candidate.final_score, "candidate built");
            candidates.push(candidate);
        }
    }

    candidates.sort_by(|a, b| b.final_score.cmp(&a.final_score));
    Ok(candidates)
}

#[allow(clippy::too_many_arguments)]
fn candidate(
    date: NaiveDate,
    stock: &crate::domain::StockBasic,
    supply_score: Decimal,
    liquidity_score: Decimal,
    trend_score: Decimal,
    risk_score: Decimal,
    final_score: Decimal,
    inst_buy_days: i32,
    foreign_buy_days: i32,
    inst_sum: i64,
    foreign_sum: i64,
    total_sum: i64,
    market_cap: Option<i64>,
    avg_trade_amount_20d: Option<i64>,
    latest: &crate::domain::OhlcvDaily,
    ma5: Option<Decimal>,
    ma20: Option<Decimal>,
    breakout_price: Option<Decimal>,
    pullback_price: Option<Decimal>,
    stop_loss_price: Option<Decimal>,
    target_price: Option<Decimal>,
    strategy_type: StrategyType,
) -> TradeCandidate {
    TradeCandidate {
        trade_date: date,
        stock_code: stock.stock_code.clone(),
        stock_name: stock.stock_name.clone(),
        supply_score,
        liquidity_score,
        trend_score,
        risk_score,
        final_score,
        inst_buy_days,
        foreign_buy_days,
        inst_net_buy_amt_5d: inst_sum,
        foreign_net_buy_amt_5d: foreign_sum,
        total_net_buy_amt_5d: total_sum,
        market_cap,
        avg_trade_amount_20d,
        prev_close: latest.close_price,
        prev_high: latest.high_price,
        prev_low: latest.low_price,
        ma5,
        ma20,
        breakout_price,
        pullback_price,
        stop_loss_price,
        target_price,
        strategy_type,
        status: CandidateStatus::Ready,
    }
}

fn max_decimal(left: Decimal, right: Decimal) -> Decimal {
    if left >= right {
        left
    } else {
        right
    }
}

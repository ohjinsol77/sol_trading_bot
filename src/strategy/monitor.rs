use chrono::{Duration, Local, NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::{
    config::AppConfig,
    discord::{format_signal_report, DiscordClient},
    domain::{BuyOrderRequest, CandidateStatus, CurrentQuote, StrategyType, TradeSignal},
    kis::KisClient,
    order::{DryRunOrderExecutor, OrderExecutor},
    storage::SqliteRepository,
};

use super::risk;

pub async fn monitor(
    date: NaiveDate,
    config: &AppConfig,
    repository: &SqliteRepository,
    kis_client: &dyn KisClient,
    discord: &DiscordClient,
) -> anyhow::Result<()> {
    let candidate_date = date - Duration::days(1);
    let candidates = repository.list_ready_candidates(candidate_date).await?;
    let executor = DryRunOrderExecutor::new(repository.clone(), discord.clone());

    for candidate in candidates {
        let quote = match kis_client.fetch_current_quote(&candidate.stock_code).await {
            Ok(quote) => quote,
            Err(err) => {
                warn!(stock_code = %candidate.stock_code, error = ?err, "quote fetch failed");
                continue;
            }
        };
        let now = Local::now().time();
        let Some(reasons) = evaluate_signal(&candidate, &quote, now, config) else {
            continue;
        };

        let daily_count = repository.count_signals(date).await?;
        let already = repository
            .has_signal_for_stock(date, &candidate.stock_code)
            .await?;
        if let Err(reason) = risk::risk_allows_new_signal(&quote, &config.risk, daily_count, already, now) {
            warn!(stock_code = %candidate.stock_code, reason, "risk blocked signal");
            continue;
        }

        let signal = TradeSignal {
            signal_date: date,
            signal_time: now,
            stock_code: candidate.stock_code.clone(),
            stock_name: candidate.stock_name.clone(),
            strategy_type: candidate.strategy_type,
            signal_price: quote.current_price,
            final_score: candidate.final_score,
            reason: reasons.join("\n"),
        };
        repository.save_signal(&signal).await?;
        repository
            .update_candidate_status(
                candidate.trade_date,
                &candidate.stock_code,
                candidate.strategy_type,
                CandidateStatus::Triggered,
            )
            .await?;

        let buy_request = BuyOrderRequest {
            trade_date: date,
            stock_code: candidate.stock_code.clone(),
            stock_name: candidate.stock_name.clone(),
            price: quote.current_price,
            qty: 1,
            reason: reasons.join(", "),
        };
        executor.submit_buy_order(buy_request).await?;

        let timestamp = format!("{} {}", date, now.format("%H:%M:%S"));
        let message = format_signal_report(
            &timestamp,
            &candidate.stock_code,
            &candidate.stock_name,
            candidate.strategy_type,
            quote.current_price,
            candidate.prev_high,
            candidate.final_score,
            &reasons,
        );
        discord.send_text(&message).await?;
        info!(stock_code = %candidate.stock_code, "signal triggered");
    }

    Ok(())
}

fn evaluate_signal(
    candidate: &crate::domain::TradeCandidate,
    quote: &CurrentQuote,
    now: NaiveTime,
    config: &AppConfig,
) -> Option<Vec<String>> {
    let active_window = now >= NaiveTime::from_hms_opt(9, 10, 0)?
        && now <= NaiveTime::from_hms_opt(15, 10, 0)?;
    if !active_window {
        return None;
    }

    match candidate.strategy_type {
        StrategyType::Opening if config.strategy.enable_opening_strategy => opening_signal(candidate, quote, now),
        StrategyType::Pullback if config.strategy.enable_pullback_strategy => pullback_signal(candidate, quote),
        StrategyType::Breakout if config.strategy.enable_breakout_strategy => breakout_signal(candidate, quote),
        _ => None,
    }
}

fn opening_signal(
    candidate: &crate::domain::TradeCandidate,
    quote: &CurrentQuote,
    now: NaiveTime,
) -> Option<Vec<String>> {
    if now < NaiveTime::from_hms_opt(9, 3, 0)? || candidate.final_score < Decimal::from(85) {
        return None;
    }
    let gap = (quote.open_price - candidate.prev_close) / candidate.prev_close;
    if gap < Decimal::new(-1, 2) || gap > Decimal::new(3, 2) {
        return None;
    }
    if quote.current_price <= quote.open_price || quote.execution_strength < Decimal::from(110) {
        return None;
    }
    Some(vec![
        "시초가 조건 통과".to_string(),
        format!("체결강도 {}", quote.execution_strength.round_dp(0)),
    ])
}

fn pullback_signal(candidate: &crate::domain::TradeCandidate, quote: &CurrentQuote) -> Option<Vec<String>> {
    if candidate.final_score < Decimal::from(70) {
        return None;
    }
    let close_band = (quote.current_price - candidate.prev_close) / candidate.prev_close;
    let ma5 = candidate.ma5?;
    let ma20 = candidate.ma20?;
    if close_band < Decimal::new(-2, 2) || close_band > Decimal::new(2, 2) {
        return None;
    }
    if quote.current_price < ma5 * Decimal::new(995, 3) || quote.current_price < ma20 {
        return None;
    }
    if quote.execution_strength < Decimal::from(100) || quote.expected_trade_amount_ratio < Decimal::from(70) {
        return None;
    }
    Some(vec![
        "눌림목 가격 구간 진입".to_string(),
        format!("체결강도 {}", quote.execution_strength.round_dp(0)),
        format!("거래대금 예상 대비 {}%", quote.expected_trade_amount_ratio.round_dp(0)),
        format!("호가 스프레드 {}%", (risk::spread_rate(quote) * Decimal::from(100)).round_dp(2)),
    ])
}

fn breakout_signal(candidate: &crate::domain::TradeCandidate, quote: &CurrentQuote) -> Option<Vec<String>> {
    if candidate.final_score < Decimal::from(80) {
        return None;
    }
    if quote.current_price < candidate.prev_high * Decimal::new(1001, 3) {
        return None;
    }
    if quote.execution_strength < Decimal::from(120) || quote.expected_trade_amount_ratio < Decimal::from(150) {
        return None;
    }
    if quote.vwap.is_some_and(|vwap| quote.current_price < vwap) {
        return None;
    }
    Some(vec![
        "전일고가 돌파".to_string(),
        format!("체결강도 {}", quote.execution_strength.round_dp(0)),
        format!("거래대금 예상 대비 {}%", quote.expected_trade_amount_ratio.round_dp(0)),
        format!("호가 스프레드 {}%", (risk::spread_rate(quote) * Decimal::from(100)).round_dp(2)),
    ])
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::domain::{CandidateStatus, TradeCandidate};

    fn candidate(strategy_type: StrategyType) -> TradeCandidate {
        TradeCandidate {
            trade_date: NaiveDate::from_ymd_opt(2026, 5, 12).unwrap(),
            stock_code: "005930".to_string(),
            stock_name: "삼성전자".to_string(),
            supply_score: Decimal::from(90),
            liquidity_score: Decimal::from(90),
            trend_score: Decimal::from(90),
            risk_score: Decimal::from(90),
            final_score: Decimal::from(90),
            inst_buy_days: 5,
            foreign_buy_days: 5,
            inst_net_buy_amt_5d: 1,
            foreign_net_buy_amt_5d: 1,
            total_net_buy_amt_5d: 2,
            market_cap: Some(1),
            avg_trade_amount_20d: Some(1),
            prev_close: Decimal::from(100),
            prev_high: Decimal::from(101),
            prev_low: Decimal::from(99),
            ma5: Some(Decimal::from(99)),
            ma20: Some(Decimal::from(98)),
            breakout_price: Some(Decimal::new(101101, 3)),
            pullback_price: Some(Decimal::from(99)),
            stop_loss_price: Some(Decimal::from(98)),
            target_price: Some(Decimal::from(103)),
            strategy_type,
            status: CandidateStatus::Ready,
        }
    }

    fn quote(price: i64) -> CurrentQuote {
        CurrentQuote {
            stock_code: "005930".to_string(),
            stock_name: "삼성전자".to_string(),
            current_price: Decimal::from(price),
            open_price: Decimal::from(100),
            high_price: Decimal::from(price),
            low_price: Decimal::from(99),
            prev_close: Decimal::from(100),
            accumulated_trade_amount: 1,
            expected_trade_amount_ratio: Decimal::from(160),
            execution_strength: Decimal::from(124),
            bid_price: Decimal::new(price * 100 - 10, 2),
            ask_price: Decimal::new(price * 100 + 10, 2),
            vwap: Some(Decimal::from(100)),
            is_vi_expected: false,
        }
    }

    #[test]
    fn detects_pullback_and_breakout() {
        assert!(pullback_signal(&candidate(StrategyType::Pullback), &quote(100)).is_some());
        assert!(breakout_signal(&candidate(StrategyType::Breakout), &quote(102)).is_some());
    }
}

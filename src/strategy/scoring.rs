use rust_decimal::Decimal;

use crate::domain::{FinancialRatio, InvestorFlowDaily, OhlcvDaily};

pub fn count_institution_buy_days(flows: &[InvestorFlowDaily]) -> i32 {
    flows
        .iter()
        .filter(|flow| flow.institution_net_buy_amt > 0)
        .count() as i32
}

pub fn count_foreign_buy_days(flows: &[InvestorFlowDaily]) -> i32 {
    flows
        .iter()
        .filter(|flow| flow.foreign_net_buy_amt > 0)
        .count() as i32
}

pub fn sum_institution_net_buy(flows: &[InvestorFlowDaily]) -> i64 {
    flows.iter().map(|flow| flow.institution_net_buy_amt).sum()
}

pub fn sum_foreign_net_buy(flows: &[InvestorFlowDaily]) -> i64 {
    flows.iter().map(|flow| flow.foreign_net_buy_amt).sum()
}

pub fn moving_average(ohlcv: &[OhlcvDaily], window: usize) -> Option<Decimal> {
    if ohlcv.len() < window {
        return None;
    }
    let slice = &ohlcv[ohlcv.len() - window..];
    let sum = slice
        .iter()
        .fold(Decimal::ZERO, |acc, row| acc + row.close_price);
    Some(sum / Decimal::from(window as i64))
}

pub fn average_trade_amount(ohlcv: &[OhlcvDaily], window: usize) -> Option<i64> {
    if ohlcv.len() < window {
        return None;
    }
    let slice = &ohlcv[ohlcv.len() - window..];
    Some(slice.iter().map(|row| row.trade_amount).sum::<i64>() / window as i64)
}

pub fn rise_rate(ohlcv: &[OhlcvDaily], window: usize) -> Option<Decimal> {
    if ohlcv.len() < window || window < 2 {
        return None;
    }
    let start = ohlcv[ohlcv.len() - window].close_price;
    let end = ohlcv.last()?.close_price;
    if start.is_zero() {
        return None;
    }
    Some((end - start) / start)
}

pub fn prev_day_rise_rate(ohlcv: &[OhlcvDaily]) -> Option<Decimal> {
    if ohlcv.len() < 2 {
        return None;
    }
    let prev = ohlcv[ohlcv.len() - 2].close_price;
    let latest = ohlcv.last()?.close_price;
    if prev.is_zero() {
        return None;
    }
    Some((latest - prev) / prev)
}

pub fn supply_score(
    inst_buy_days: i32,
    foreign_buy_days: i32,
    total_net_buy_amt_5d: i64,
    market_cap: Option<i64>,
) -> Decimal {
    let mut score = Decimal::ZERO;
    score += match inst_buy_days {
        5 => Decimal::from(25),
        4 => Decimal::from(20),
        3 => Decimal::from(15),
        _ => Decimal::ZERO,
    };
    score += match foreign_buy_days {
        5 => Decimal::from(25),
        4 => Decimal::from(20),
        3 => Decimal::from(15),
        _ => Decimal::ZERO,
    };
    score += if total_net_buy_amt_5d >= 10_000_000_000 {
        Decimal::from(20)
    } else if total_net_buy_amt_5d >= 5_000_000_000 {
        Decimal::from(15)
    } else if total_net_buy_amt_5d >= 3_000_000_000 {
        Decimal::from(10)
    } else if total_net_buy_amt_5d >= 1_000_000_000 {
        Decimal::from(5)
    } else {
        Decimal::ZERO
    };

    if let Some(market_cap) = market_cap.filter(|value| *value > 0) {
        let ratio = Decimal::from(total_net_buy_amt_5d) / Decimal::from(market_cap);
        score += if ratio >= Decimal::new(10, 3) {
            Decimal::from(20)
        } else if ratio >= Decimal::new(5, 3) {
            Decimal::from(15)
        } else if ratio >= Decimal::new(2, 3) {
            Decimal::from(10)
        } else if ratio >= Decimal::new(1, 3) {
            Decimal::from(5)
        } else {
            Decimal::ZERO
        };
    }
    clamp_score(score)
}

pub fn liquidity_score(ohlcv: &[OhlcvDaily]) -> Decimal {
    let avg20 = average_trade_amount(ohlcv, 20).unwrap_or_default();
    let avg5 = average_trade_amount(ohlcv, 5).unwrap_or_default();
    let mut score = Decimal::ZERO;
    score += tier_score(avg20, &[(50_000_000_000, 40), (20_000_000_000, 30), (10_000_000_000, 20), (5_000_000_000, 10)]);
    score += tier_score(avg5, &[(50_000_000_000, 30), (20_000_000_000, 20), (10_000_000_000, 15), (5_000_000_000, 10)]);
    if avg20 > 0 {
        let ratio = Decimal::from(avg5) / Decimal::from(avg20);
        score += if ratio >= Decimal::from(2) {
            Decimal::from(30)
        } else if ratio >= Decimal::new(15, 1) {
            Decimal::from(20)
        } else if ratio >= Decimal::new(12, 1) {
            Decimal::from(10)
        } else {
            Decimal::ZERO
        };
    }
    clamp_score(score)
}

pub fn trend_score(ohlcv: &[OhlcvDaily], ma5: Option<Decimal>, ma20: Option<Decimal>) -> Decimal {
    let Some(latest) = ohlcv.last() else {
        return Decimal::ZERO;
    };
    let mut score = Decimal::ZERO;
    if ma5.is_some_and(|ma| latest.close_price > ma) {
        score += Decimal::from(20);
    }
    if ma20.is_some_and(|ma| latest.close_price > ma) {
        score += Decimal::from(20);
    }
    if let (Some(ma5), Some(ma20)) = (ma5, ma20) {
        if ma5 > ma20 {
            score += Decimal::from(20);
        }
    }
    if let Some(rate) = rise_rate(ohlcv, 5) {
        if rate >= Decimal::ZERO && rate <= Decimal::new(15, 2) {
            score += Decimal::from(20);
        }
        if rate > Decimal::new(25, 2) {
            score -= Decimal::from(40);
        }
    }
    if let Some(high20) = ohlcv.iter().map(|row| row.high_price).max() {
        if high20 > Decimal::ZERO && latest.close_price >= high20 * Decimal::new(90, 2) {
            score += Decimal::from(20);
        }
    }
    clamp_score(score)
}

pub fn risk_score(financial: Option<&FinancialRatio>, latest: Option<&OhlcvDaily>, prev: Option<&OhlcvDaily>) -> Decimal {
    let mut score = Decimal::from(100);
    if let Some(financial) = financial {
        if financial.debt_ratio.is_some_and(|ratio| ratio > Decimal::from(300)) {
            score -= Decimal::from(30);
        }
        if financial.operating_profit.is_some_and(|profit| profit < 0) {
            score -= Decimal::from(20);
        }
        if financial.quarterly_operating_profit.is_some_and(|profit| profit < 0) {
            score -= Decimal::from(10);
        }
        if financial.capital_impairment == Some(true) {
            score -= Decimal::from(50);
        }
    }
    if let Some(latest) = latest {
        let upper_wick = latest.high_price - latest.close_price;
        let range = latest.high_price - latest.low_price;
        if range > Decimal::ZERO && upper_wick / range > Decimal::new(45, 2) {
            score -= Decimal::from(15);
        }
    }
    if let (Some(latest), Some(prev)) = (latest, prev) {
        if latest.volume > prev.volume * 3 && latest.close_price < latest.open_price {
            score -= Decimal::from(20);
        }
    }
    clamp_score(score)
}

pub fn final_score(
    supply_score: Decimal,
    liquidity_score: Decimal,
    trend_score: Decimal,
    risk_score: Decimal,
) -> Decimal {
    clamp_score(
        supply_score * Decimal::new(45, 2)
            + liquidity_score * Decimal::new(20, 2)
            + trend_score * Decimal::new(20, 2)
            + risk_score * Decimal::new(15, 2),
    )
}

fn tier_score(value: i64, tiers: &[(i64, i64)]) -> Decimal {
    tiers
        .iter()
        .find_map(|(threshold, score)| (value >= *threshold).then_some(Decimal::from(*score)))
        .unwrap_or(Decimal::ZERO)
}

fn clamp_score(score: Decimal) -> Decimal {
    if score < Decimal::ZERO {
        Decimal::ZERO
    } else if score > Decimal::from(100) {
        Decimal::from(100)
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn flow(inst: i64, foreign: i64) -> InvestorFlowDaily {
        InvestorFlowDaily {
            trade_date: NaiveDate::from_ymd_opt(2026, 5, 12).unwrap(),
            stock_code: "005930".to_string(),
            institution_net_buy_amt: inst,
            foreign_net_buy_amt: foreign,
            individual_net_buy_amt: -(inst + foreign),
            institution_net_buy_qty: None,
            foreign_net_buy_qty: None,
            individual_net_buy_qty: None,
        }
    }

    fn candle(close: i64, amount: i64) -> OhlcvDaily {
        OhlcvDaily {
            trade_date: NaiveDate::from_ymd_opt(2026, 5, 12).unwrap(),
            stock_code: "005930".to_string(),
            open_price: Decimal::from(close - 10),
            high_price: Decimal::from(close + 20),
            low_price: Decimal::from(close - 20),
            close_price: Decimal::from(close),
            volume: 1000,
            trade_amount: amount,
        }
    }

    #[test]
    fn counts_buy_days_and_sums() {
        let flows = vec![flow(1, 1), flow(1, -1), flow(-1, 1), flow(1, 1), flow(1, 1)];
        assert_eq!(count_institution_buy_days(&flows), 4);
        assert_eq!(count_foreign_buy_days(&flows), 4);
        assert_eq!(sum_institution_net_buy(&flows), 3);
    }

    #[test]
    fn calculates_moving_average_and_trade_amount() {
        let rows = (1..=20).map(|idx| candle(idx, idx * 100)).collect::<Vec<_>>();
        assert_eq!(moving_average(&rows, 5), Some(Decimal::from(18)));
        assert_eq!(moving_average(&rows, 20), Some(Decimal::new(105, 1)));
        assert_eq!(average_trade_amount(&rows, 20), Some(1050));
    }

    #[test]
    fn calculates_scores() {
        let rows = (1..=20)
            .map(|idx| candle(10_000 + idx * 10, 30_000_000_000))
            .collect::<Vec<_>>();
        assert!(supply_score(5, 5, 12_000_000_000, Some(1_000_000_000_000)) > Decimal::from(80));
        assert!(liquidity_score(&rows) >= Decimal::from(60));
        assert!(trend_score(&rows, moving_average(&rows, 5), moving_average(&rows, 20)) > Decimal::from(60));
        assert!(risk_score(None, rows.last(), rows.get(18)) > Decimal::from(70));
        assert!(final_score(Decimal::from(90), Decimal::from(70), Decimal::from(80), Decimal::from(100)) > Decimal::from(80));
    }
}

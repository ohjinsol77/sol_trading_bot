use chrono::NaiveTime;
use rust_decimal::Decimal;

use crate::{config::RiskSection, domain::CurrentQuote};

pub fn is_after_no_new_buy(now: NaiveTime, risk: &RiskSection) -> anyhow::Result<bool> {
    let cutoff = NaiveTime::parse_from_str(&risk.no_new_buy_after, "%H:%M:%S")?;
    Ok(now >= cutoff)
}

pub fn spread_rate(quote: &CurrentQuote) -> Decimal {
    if quote.current_price.is_zero() {
        return Decimal::from(999_999);
    }
    let spread = if quote.ask_price >= quote.bid_price {
        quote.ask_price - quote.bid_price
    } else {
        quote.bid_price - quote.ask_price
    };
    spread / quote.current_price
}

pub fn intraday_rise_rate(quote: &CurrentQuote) -> Decimal {
    if quote.prev_close.is_zero() {
        return Decimal::ZERO;
    }
    (quote.current_price - quote.prev_close) / quote.prev_close
}

pub fn risk_allows_new_signal(
    quote: &CurrentQuote,
    risk: &RiskSection,
    daily_signal_count: i64,
    already_signaled: bool,
    now: NaiveTime,
) -> Result<(), String> {
    if daily_signal_count >= risk.max_daily_buy_count {
        return Err("하루 최대 신규 매수 신호 수 초과".to_string());
    }
    if already_signaled {
        return Err("동일 종목 하루 중복 신호 방지".to_string());
    }
    if is_after_no_new_buy(now, risk).map_err(|err| err.to_string())? {
        return Err("15:10 이후 신규 매수 금지".to_string());
    }
    if spread_rate(quote) > risk.max_spread_rate {
        return Err("호가 스프레드 초과".to_string());
    }
    if intraday_rise_rate(quote) > risk.max_intraday_rise_rate {
        return Err("당일 상승률 제한 초과".to_string());
    }
    if quote.is_vi_expected {
        return Err("VI 직전 또는 VI 발동 추정 종목 제외".to_string());
    }
    Ok(())
}

pub fn daily_loss_guard(daily_loss_rate: Option<Decimal>, max_daily_loss_rate: Decimal) -> Result<(), String> {
    if daily_loss_rate.is_some_and(|rate| rate <= max_daily_loss_rate) {
        return Err("전체 계좌 기준 일손실 제한 도달".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quote() -> CurrentQuote {
        CurrentQuote {
            stock_code: "005930".to_string(),
            stock_name: "삼성전자".to_string(),
            current_price: Decimal::from(100),
            open_price: Decimal::from(99),
            high_price: Decimal::from(101),
            low_price: Decimal::from(98),
            prev_close: Decimal::from(99),
            accumulated_trade_amount: 1,
            expected_trade_amount_ratio: Decimal::from(100),
            execution_strength: Decimal::from(120),
            bid_price: Decimal::new(999, 1),
            ask_price: Decimal::new(1001, 1),
            vwap: Some(Decimal::from(99)),
            is_vi_expected: false,
        }
    }

    #[test]
    fn blocks_daily_signal_limit_and_duplicate() {
        let risk = crate::config::AppConfig::default().risk;
        let now = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        assert!(risk_allows_new_signal(&quote(), &risk, 3, false, now).is_err());
        assert!(risk_allows_new_signal(&quote(), &risk, 0, true, now).is_err());
    }
}

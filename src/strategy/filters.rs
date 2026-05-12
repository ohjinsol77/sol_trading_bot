use rust_decimal::Decimal;

use crate::{
    config::StrategySection,
    domain::{FinancialRatio, InvestorFlowDaily, OhlcvDaily, StockBasic},
    strategy::scoring,
};

pub fn stock_is_excluded(stock: &StockBasic) -> Option<&'static str> {
    if stock.is_administrative_issue {
        return Some("관리종목 제외");
    }
    if stock.is_trading_halted {
        return Some("거래정지 제외");
    }
    if stock.is_warning {
        return Some("투자경고/위험 제외");
    }
    if stock.is_preferred {
        return Some("우선주 제외");
    }
    if stock.is_etf {
        return Some("ETF 제외");
    }
    if stock.is_etn {
        return Some("ETN 제외");
    }
    if stock.is_spac {
        return Some("스팩 제외");
    }
    None
}

pub fn passes_candidate_filters(
    stock: &StockBasic,
    flows: &[InvestorFlowDaily],
    ohlcv: &[OhlcvDaily],
    financial: Option<&FinancialRatio>,
    market_cap: Option<i64>,
    strategy: &StrategySection,
) -> Result<(), String> {
    if let Some(reason) = stock_is_excluded(stock) {
        return Err(reason.to_string());
    }
    if flows.len() < strategy.lookback_days {
        return Err("수급 데이터 부족".to_string());
    }
    if ohlcv.len() < 20 {
        return Err("OHLCV 데이터 부족".to_string());
    }

    let inst_days = scoring::count_institution_buy_days(flows);
    let foreign_days = scoring::count_foreign_buy_days(flows);
    if inst_days < strategy.min_institution_buy_days {
        return Err("기관 연속 순매수 일수 부족".to_string());
    }
    if foreign_days < strategy.min_foreign_buy_days {
        return Err("외국인 연속 순매수 일수 부족".to_string());
    }

    let inst_sum = scoring::sum_institution_net_buy(flows);
    let foreign_sum = scoring::sum_foreign_net_buy(flows);
    if inst_sum <= 0 || foreign_sum <= 0 {
        return Err("기관/외국인 누적 순매수 금액 부족".to_string());
    }
    if inst_sum + foreign_sum < strategy.min_total_net_buy_amt_5d {
        return Err("5일 누적 순매수 금액 부족".to_string());
    }
    if market_cap.unwrap_or_default() < strategy.min_market_cap {
        return Err("시가총액 부족".to_string());
    }
    if scoring::average_trade_amount(ohlcv, 20).unwrap_or_default() < strategy.min_avg_trade_amount_20d {
        return Err("20일 평균 거래대금 부족".to_string());
    }
    if scoring::rise_rate(ohlcv, 5).unwrap_or(Decimal::ZERO) > strategy.max_rise_rate_5d {
        return Err("최근 5일 상승률 과다".to_string());
    }
    if scoring::prev_day_rise_rate(ohlcv).unwrap_or(Decimal::ZERO) > strategy.max_prev_day_rise_rate {
        return Err("전일 상승률 과다".to_string());
    }
    if financial.and_then(|f| f.capital_impairment) == Some(true) {
        return Err("자본잠식 제외".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn excludes_flagged_stock() {
        let mut stock = StockBasic {
            stock_code: "X".to_string(),
            stock_name: "X".to_string(),
            market: None,
            sector: None,
            is_preferred: false,
            is_etf: true,
            is_etn: false,
            is_spac: false,
            is_trading_halted: false,
            is_administrative_issue: false,
            is_warning: false,
        };
        assert!(stock_is_excluded(&stock).is_some());
        stock.is_etf = false;
        assert!(stock_is_excluded(&stock).is_none());
    }

    #[test]
    fn passes_basic_candidate_filters() {
        let stock = StockBasic {
            stock_code: "005930".to_string(),
            stock_name: "삼성전자".to_string(),
            market: None,
            sector: None,
            is_preferred: false,
            is_etf: false,
            is_etn: false,
            is_spac: false,
            is_trading_halted: false,
            is_administrative_issue: false,
            is_warning: false,
        };
        let flows = (0..5)
            .map(|_| InvestorFlowDaily {
                trade_date: NaiveDate::from_ymd_opt(2026, 5, 12).unwrap(),
                stock_code: stock.stock_code.clone(),
                institution_net_buy_amt: 1_000_000_000,
                foreign_net_buy_amt: 1_000_000_000,
                individual_net_buy_amt: -2_000_000_000,
                institution_net_buy_qty: None,
                foreign_net_buy_qty: None,
                individual_net_buy_qty: None,
            })
            .collect::<Vec<_>>();
        let rows = (0..20)
            .map(|idx| OhlcvDaily {
                trade_date: NaiveDate::from_ymd_opt(2026, 5, 12).unwrap(),
                stock_code: stock.stock_code.clone(),
                open_price: Decimal::from(10_000 + idx),
                high_price: Decimal::from(10_050 + idx),
                low_price: Decimal::from(9_950 + idx),
                close_price: Decimal::from(10_000 + idx),
                volume: 1000,
                trade_amount: 10_000_000_000,
            })
            .collect::<Vec<_>>();
        let strategy = crate::config::AppConfig::default().strategy;
        assert!(passes_candidate_filters(&stock, &flows, &rows, None, Some(1_000_000_000_000), &strategy).is_ok());
    }
}

pub mod client;
pub mod models;

pub use client::DiscordClient;

use chrono::NaiveDate;

use crate::domain::{StrategyType, TradeCandidate};

pub fn format_candidate_report(date: NaiveDate, candidates: &[TradeCandidate]) -> String {
    let mut lines = vec![
        "[후보 생성 완료]".to_string(),
        format!("기준일: {date}"),
        format!("후보 수: {}개", candidates.len()),
        String::new(),
    ];

    for (idx, candidate) in candidates.iter().take(10).enumerate() {
        lines.push(format!(
            "{}. {} {}",
            idx + 1,
            candidate.stock_code,
            candidate.stock_name
        ));
        lines.push(format!("   점수: {}", candidate.final_score.round_dp(1)));
        lines.push(format!(
            "   기관 5일 순매수: {}억",
            candidate.inst_net_buy_amt_5d / 100_000_000
        ));
        lines.push(format!(
            "   외국인 5일 순매수: {}억",
            candidate.foreign_net_buy_amt_5d / 100_000_000
        ));
        lines.push(format!("   전략: {}", candidate.strategy_type));
        lines.push(String::new());
    }

    if candidates.is_empty() {
        lines.push("조건을 통과한 후보가 없습니다.".to_string());
    }

    lines.join("\n")
}

pub fn format_signal_report(
    signal_time: &str,
    stock_code: &str,
    stock_name: &str,
    strategy_type: StrategyType,
    current_price: rust_decimal::Decimal,
    prev_high: rust_decimal::Decimal,
    final_score: rust_decimal::Decimal,
    reasons: &[String],
) -> String {
    let mut lines = vec![
        "[DRY_RUN 매수 신호]".to_string(),
        format!("시간: {signal_time}"),
        format!("종목: {stock_code} {stock_name}"),
        format!("전략: {strategy_type}"),
        format!("현재가: {}", current_price.round_dp(0)),
        format!("전일고가: {}", prev_high.round_dp(0)),
        format!("final_score: {}", final_score.round_dp(1)),
        String::new(),
        "사유:".to_string(),
    ];
    lines.extend(reasons.iter().map(|reason| format!("- {reason}")));
    lines.push(String::new());
    lines.push("상태:".to_string());
    lines.push("실제 주문 없음. DRY_RUN 기록만 저장.".to_string());
    lines.join("\n")
}

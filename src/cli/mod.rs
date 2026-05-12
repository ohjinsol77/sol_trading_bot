use anyhow::Context;
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "kis-supply-bot")]
#[command(about = "KIS 기관/외국인 수급 기반 DRY_RUN 자동매매 봇")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    CollectDaily {
        #[arg(long)]
        date: Option<NaiveDate>,
    },
    BuildCandidates {
        #[arg(long)]
        date: Option<NaiveDate>,
    },
    Monitor {
        #[arg(long)]
        date: Option<NaiveDate>,
    },
    Report {
        #[arg(long)]
        date: Option<NaiveDate>,
    },
    Backtest {
        #[arg(long)]
        from: NaiveDate,
        #[arg(long)]
        to: NaiveDate,
    },
    HealthCheck,
}

pub fn resolve_date(date: Option<NaiveDate>) -> anyhow::Result<NaiveDate> {
    Ok(match date {
        Some(date) => date,
        None => Local::now()
            .date_naive()
            .checked_sub_days(chrono::Days::new(0))
            .context("failed to resolve local date")?,
    })
}

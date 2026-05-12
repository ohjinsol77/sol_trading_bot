use chrono::{Duration, NaiveDate};
use tracing::{info, warn};

use crate::{domain::MarketMetricDaily, kis::KisClient, storage::SqliteRepository};

pub async fn collect_daily(
    date: NaiveDate,
    repository: &SqliteRepository,
    kis_client: &dyn KisClient,
) -> anyhow::Result<()> {
    let universe = kis_client.fetch_stock_universe().await?;
    let from = date - Duration::days(30);
    let mut success = 0;
    let mut failed = 0;

    for stock in universe {
        info!(stock_code = %stock.stock_code, "collecting data per stock");
        if let Err(err) = collect_one(date, from, repository, kis_client, &stock).await {
            failed += 1;
            warn!(stock_code = %stock.stock_code, error = ?err, "stock collection failed; skipped");
            continue;
        }
        success += 1;
    }

    info!(success, failed, "Data collection completed with partial failure tolerance");
    Ok(())
}

async fn collect_one(
    date: NaiveDate,
    from: NaiveDate,
    repository: &SqliteRepository,
    kis_client: &dyn KisClient,
    stock: &crate::domain::StockBasic,
) -> anyhow::Result<()> {
    repository.save_stock(stock).await?;

    let ohlcv = kis_client
        .fetch_ohlcv_daily(&stock.stock_code, from, date)
        .await?;
    repository.save_ohlcv_daily(&ohlcv).await?;

    let flows = kis_client
        .fetch_investor_flow_daily(&stock.stock_code, from, date)
        .await?;
    repository.save_investor_flow_daily(&flows).await?;

    if let Some(financial) = kis_client.fetch_financial_ratio(&stock.stock_code).await? {
        repository.save_financial_ratio(&financial).await?;
    }

    let market_cap = kis_client.fetch_market_cap(&stock.stock_code, date).await?;
    repository
        .save_market_metric(&MarketMetricDaily {
            trade_date: date,
            stock_code: stock.stock_code.clone(),
            market_cap,
            credit_balance: None,
            short_selling_amount: None,
        })
        .await?;

    Ok(())
}

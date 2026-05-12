use async_trait::async_trait;
use tracing::info;

use crate::{
    discord::DiscordClient,
    domain::{BuyOrderRequest, OrderExecutionResult, OrderStatus, SellOrderRequest},
    storage::SqliteRepository,
};

#[async_trait]
pub trait OrderExecutor: Send + Sync {
    async fn submit_buy_order(&self, request: BuyOrderRequest) -> anyhow::Result<OrderExecutionResult>;
    async fn submit_sell_order(&self, request: SellOrderRequest) -> anyhow::Result<OrderExecutionResult>;
}

#[derive(Debug, Clone)]
pub struct DryRunOrderExecutor {
    repository: SqliteRepository,
    discord: DiscordClient,
}

impl DryRunOrderExecutor {
    pub fn new(repository: SqliteRepository, discord: DiscordClient) -> Self {
        Self { repository, discord }
    }
}

#[async_trait]
impl OrderExecutor for DryRunOrderExecutor {
    async fn submit_buy_order(&self, request: BuyOrderRequest) -> anyhow::Result<OrderExecutionResult> {
        let order_id = self.repository.create_dry_run_order(&request).await?;
        info!(order_id, stock_code = %request.stock_code, "order dry-run created");
        self.discord
            .send_text(&format!(
                "[DRY_RUN 주문 기록]\n종목: {} {}\n가격: {}\n수량: {}\n상태: 실제 주문 없음",
                request.stock_code, request.stock_name, request.price, request.qty
            ))
            .await?;
        Ok(OrderExecutionResult {
            order_id: Some(order_id),
            kis_order_no: None,
            status: OrderStatus::DryRun,
            message: "DRY_RUN buy order recorded".to_string(),
        })
    }

    async fn submit_sell_order(&self, _request: SellOrderRequest) -> anyhow::Result<OrderExecutionResult> {
        Ok(OrderExecutionResult {
            order_id: None,
            kis_order_no: None,
            status: OrderStatus::DryRun,
            message: "DRY_RUN sell order scaffold; position management TODO".to_string(),
        })
    }
}

pub struct RealOrderExecutor;
pub struct KisOrderExecutor;

impl RealOrderExecutor {
    pub async fn cancel_order(&self) -> anyhow::Result<()> {
        anyhow::bail!("cancel_order TODO: real KIS order is disabled")
    }

    pub async fn fetch_order_status(&self) -> anyhow::Result<()> {
        anyhow::bail!("fetch_order_status TODO: real KIS order is disabled")
    }
}

use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDate;
use tokio::sync::RwLock;
use tracing::warn;

use crate::{
    config::{AppConfig, TradingMode},
    domain::*,
};

pub type SharedKisClient = Arc<dyn KisClient>;

#[async_trait]
pub trait KisClient: Send + Sync {
    async fn issue_access_token(&self) -> anyhow::Result<String>;
    async fn fetch_stock_universe(&self) -> anyhow::Result<Vec<StockBasic>>;
    async fn fetch_investor_flow_daily(
        &self,
        stock_code: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> anyhow::Result<Vec<InvestorFlowDaily>>;
    async fn fetch_ohlcv_daily(
        &self,
        stock_code: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> anyhow::Result<Vec<OhlcvDaily>>;
    async fn fetch_stock_basic(&self, stock_code: &str) -> anyhow::Result<StockBasic>;
    async fn fetch_financial_ratio(&self, stock_code: &str) -> anyhow::Result<Option<FinancialRatio>>;
    async fn fetch_market_cap(&self, stock_code: &str, date: NaiveDate) -> anyhow::Result<Option<i64>>;
    async fn fetch_current_quote(&self, stock_code: &str) -> anyhow::Result<CurrentQuote>;
    async fn submit_order(&self, request: OrderRequest) -> anyhow::Result<OrderResponse>;
}

pub fn client_from_config(config: &AppConfig) -> SharedKisClient {
    if config.app.trading_mode == TradingMode::Mock || config.kis.credentials_missing() {
        if config.kis.credentials_missing() {
            warn!("KIS API credentials are missing. Running in mock data mode.");
        }
        Arc::new(crate::kis::MockKisClient::new())
    } else {
        let base_url = match config.app.trading_mode {
            TradingMode::Mock => config.kis.base_url_mock.clone(),
            TradingMode::Paper => config.kis.base_url_mock.clone(),
            TradingMode::Real => config.kis.base_url_real.clone(),
        };
        Arc::new(HttpKisClient::new(
            base_url,
            config.kis.app_key.clone(),
            config.kis.app_secret.clone(),
        ))
    }
}

pub struct HttpKisClient {
    http: reqwest::Client,
    base_url: String,
    app_key: String,
    app_secret: String,
    access_token: RwLock<Option<String>>,
}

impl HttpKisClient {
    pub fn new(base_url: String, app_key: String, app_secret: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            app_key,
            app_secret,
            access_token: RwLock::new(None),
        }
    }
}

#[async_trait]
impl KisClient for HttpKisClient {
    async fn issue_access_token(&self) -> anyhow::Result<String> {
        let url = format!("{}/oauth2/tokenP", self.base_url.trim_end_matches('/'));
        let response = self
            .http
            .post(url)
            .json(&serde_json::json!({
                "grant_type": "client_credentials",
                "appkey": self.app_key,
                "appsecret": self.app_secret,
            }))
            .send()
            .await
            .context("request KIS token")?;
        let status = response.status();
        let body = response.text().await.context("read KIS token body")?;
        if !status.is_success() {
            anyhow::bail!("KIS token failed: {status} {}", body.chars().take(300).collect::<String>());
        }
        let value: serde_json::Value = serde_json::from_str(&body).context("parse KIS token json")?;
        let token = value
            .get("access_token")
            .and_then(|v| v.as_str())
            .context("KIS token response missing access_token")?
            .to_string();
        *self.access_token.write().await = Some(token.clone());
        Ok(token)
    }

    async fn fetch_stock_universe(&self) -> anyhow::Result<Vec<StockBasic>> {
        anyhow::bail!("KIS stock universe endpoint TODO: official endpoint/TR_ID required")
    }

    async fn fetch_investor_flow_daily(
        &self,
        _stock_code: &str,
        _from: NaiveDate,
        _to: NaiveDate,
    ) -> anyhow::Result<Vec<InvestorFlowDaily>> {
        anyhow::bail!("KIS investor flow endpoint TODO: official endpoint/TR_ID required")
    }

    async fn fetch_ohlcv_daily(
        &self,
        _stock_code: &str,
        _from: NaiveDate,
        _to: NaiveDate,
    ) -> anyhow::Result<Vec<OhlcvDaily>> {
        anyhow::bail!("KIS OHLCV endpoint TODO: official endpoint/TR_ID required")
    }

    async fn fetch_stock_basic(&self, _stock_code: &str) -> anyhow::Result<StockBasic> {
        anyhow::bail!("KIS stock basic endpoint TODO: official endpoint/TR_ID required")
    }

    async fn fetch_financial_ratio(&self, _stock_code: &str) -> anyhow::Result<Option<FinancialRatio>> {
        anyhow::bail!("KIS financial endpoint TODO: official endpoint/TR_ID required")
    }

    async fn fetch_market_cap(&self, _stock_code: &str, _date: NaiveDate) -> anyhow::Result<Option<i64>> {
        anyhow::bail!("KIS market cap endpoint TODO: official endpoint/TR_ID required")
    }

    async fn fetch_current_quote(&self, _stock_code: &str) -> anyhow::Result<CurrentQuote> {
        anyhow::bail!("KIS quote endpoint TODO: official endpoint/TR_ID required")
    }

    async fn submit_order(&self, _request: OrderRequest) -> anyhow::Result<OrderResponse> {
        anyhow::bail!("real KIS order submission is intentionally not implemented")
    }
}

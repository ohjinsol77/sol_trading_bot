use anyhow::Context;
use tracing::{info, warn};

use super::models::DiscordWebhookPayload;

#[derive(Debug, Clone)]
pub struct DiscordClient {
    webhook_url: Option<String>,
    http: reqwest::Client,
}

impl DiscordClient {
    pub fn new(webhook_url: String) -> Self {
        let webhook_url = if webhook_url.trim().is_empty() {
            None
        } else {
            Some(webhook_url)
        };
        Self {
            webhook_url,
            http: reqwest::Client::new(),
        }
    }

    pub async fn send_text(&self, content: &str) -> anyhow::Result<()> {
        let Some(url) = &self.webhook_url else {
            warn!("Discord webhook URL is empty; message skipped");
            return Ok(());
        };
        let payload = DiscordWebhookPayload {
            content: content.to_string(),
        };
        let response = self
            .http
            .post(url)
            .json(&payload)
            .send()
            .await
            .context("send discord webhook")?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("discord webhook failed: {status} {body}");
        }
        info!("discord message sent");
        Ok(())
    }

    pub async fn send_error(&self, title: &str, error: &str) -> anyhow::Result<()> {
        self.send_text(&format!("[오류 발생]\n{title}\n{error}")).await
    }
}

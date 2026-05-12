use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DiscordWebhookPayload {
    pub content: String,
}

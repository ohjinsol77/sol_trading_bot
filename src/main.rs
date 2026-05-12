mod cli;
mod config;
mod discord;
mod domain;
mod error;
mod kis;
mod order;
mod scheduler;
mod storage;
mod strategy;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands};
use config::AppConfig;
use discord::DiscordClient;
use kis::{client_from_config, SharedKisClient};
use storage::SqliteRepository;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let cli = Cli::parse();
    let config = AppConfig::load().context("config loaded failed")?;
    info!(mode = %config.app.trading_mode, dry_run = config.app.dry_run, "config loaded");

    let repository = SqliteRepository::connect(&config.database.url)
        .await
        .context("database connected failed")?;
    repository.migrate().await.context("migration failed")?;
    info!("migration complete");

    let discord = DiscordClient::new(config.discord.webhook_url.clone());
    let kis_client = client_from_config(&config);

    if let Err(err) = dispatch(cli.command, &config, &repository, &discord, kis_client).await {
        error!(error = ?err, "command failed");
        discord
            .send_error("명령 실행 오류", &format!("{err:#}"))
            .await
            .ok();
        return Err(err);
    }

    Ok(())
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}

async fn dispatch(
    command: Commands,
    config: &AppConfig,
    repository: &SqliteRepository,
    discord: &DiscordClient,
    kis_client: SharedKisClient,
) -> anyhow::Result<()> {
    match command {
        Commands::CollectDaily { date } => {
            let date = cli::resolve_date(date)?;
            discord
                .send_text(&format!("[수집 시작]\n기준일: {date}\n모드: {}", config.app.trading_mode))
                .await?;
            strategy::collect::collect_daily(date, repository, kis_client.as_ref()).await?;
            discord
                .send_text(&format!("[수집 완료]\n기준일: {date}\n모드: {}\nDRY_RUN: {}", config.app.trading_mode, config.app.dry_run))
                .await?;
        }
        Commands::BuildCandidates { date } => {
            let date = cli::resolve_date(date)?;
            let candidates = strategy::candidate_builder::build_candidates(date, config, repository).await?;
            let message = discord::format_candidate_report(date, &candidates);
            discord.send_text(&message).await?;
        }
        Commands::Monitor { date } => {
            let date = cli::resolve_date(date)?;
            if !config.app.dry_run {
                anyhow::bail!(
                    "DRY_RUN=false is not supported in this initial version; real order execution is disabled"
                );
            }
            discord
                .send_text(&format!("[장중 감시 시작]\n기준일: {date}\nDRY_RUN: {}", config.app.dry_run))
                .await?;
            strategy::monitor::monitor(date, config, repository, kis_client.as_ref(), discord).await?;
            discord
                .send_text(&format!("[장중 감시 종료]\n기준일: {date}"))
                .await?;
        }
        Commands::Report { date } => {
            let date = cli::resolve_date(date)?;
            let candidates = repository.list_candidates(date).await?;
            discord.send_text(&discord::format_candidate_report(date, &candidates)).await?;
        }
        Commands::Backtest { from, to } => {
            warn!(%from, %to, "backtest command is scaffolded for future historical replay");
        }
        Commands::HealthCheck => {
            let mut lines = vec![
                "[Health Check]".to_string(),
                format!("모드: {}", config.app.trading_mode),
                format!("DRY_RUN: {}", config.app.dry_run),
                format!("DATABASE_URL: {}", config.database.url),
            ];
            if config.kis.credentials_missing() {
                lines.push("KIS API credentials are missing. Running in mock data mode.".to_string());
            }
            if config.app.dry_run {
                lines.push("DRY_RUN is enabled. No real orders will be submitted.".to_string());
            }
            let message = lines.join("\n");
            println!("{message}");
            discord.send_text(&message).await?;
        }
    }
    Ok(())
}

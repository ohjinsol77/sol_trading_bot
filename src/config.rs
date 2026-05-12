use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{env, fmt, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSection,
    pub kis: KisSection,
    pub discord: DiscordSection,
    pub database: DatabaseSection,
    pub strategy: StrategySection,
    pub risk: RiskSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSection {
    pub env: String,
    pub trading_mode: TradingMode,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TradingMode {
    Mock,
    Paper,
    Real,
}

impl fmt::Display for TradingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingMode::Mock => write!(f, "mock"),
            TradingMode::Paper => write!(f, "paper"),
            TradingMode::Real => write!(f, "real"),
        }
    }
}

impl std::str::FromStr for TradingMode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "mock" => Ok(Self::Mock),
            "paper" => Ok(Self::Paper),
            "real" => Ok(Self::Real),
            _ => anyhow::bail!("unsupported trading mode: {value}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KisSection {
    pub base_url_mock: String,
    pub base_url_real: String,
    pub app_key: String,
    pub app_secret: String,
    pub account_no: String,
    pub account_product_code: String,
    pub enable_real_order: bool,
}

impl KisSection {
    pub fn credentials_missing(&self) -> bool {
        self.app_key.trim().is_empty()
            || self.app_secret.trim().is_empty()
            || self.account_no.trim().is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordSection {
    pub webhook_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSection {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySection {
    pub enable_opening_strategy: bool,
    pub enable_pullback_strategy: bool,
    pub enable_breakout_strategy: bool,
    pub lookback_days: usize,
    pub min_institution_buy_days: i32,
    pub min_foreign_buy_days: i32,
    pub min_total_net_buy_amt_5d: i64,
    pub min_market_cap: i64,
    pub min_avg_trade_amount_20d: i64,
    pub max_rise_rate_5d: rust_decimal::Decimal,
    pub max_prev_day_rise_rate: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSection {
    pub max_daily_buy_count: i64,
    pub max_position_ratio: rust_decimal::Decimal,
    pub max_daily_loss_rate: rust_decimal::Decimal,
    pub max_spread_rate: rust_decimal::Decimal,
    pub max_intraday_rise_rate: rust_decimal::Decimal,
    pub no_new_buy_after: String,
    pub restrict_same_sector: bool,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = if Path::new("config.yaml").exists() {
            let raw = fs::read_to_string("config.yaml").context("read config.yaml")?;
            serde_yaml::from_str::<AppConfig>(&raw).context("parse config.yaml")?
        } else {
            AppConfig::default()
        };

        config.apply_env_overrides()?;
        Ok(config)
    }

    fn apply_env_overrides(&mut self) -> anyhow::Result<()> {
        set_string("APP_ENV", &mut self.app.env);
        if let Ok(value) = env::var("TRADING_MODE") {
            self.app.trading_mode = value.parse()?;
        }
        set_bool("DRY_RUN", &mut self.app.dry_run)?;
        set_string("KIS_BASE_URL_MOCK", &mut self.kis.base_url_mock);
        set_string("KIS_BASE_URL_REAL", &mut self.kis.base_url_real);
        set_string("KIS_APP_KEY", &mut self.kis.app_key);
        set_string("KIS_APP_SECRET", &mut self.kis.app_secret);
        set_string("KIS_ACCOUNT_NO", &mut self.kis.account_no);
        set_string("KIS_ACCOUNT_PRODUCT_CODE", &mut self.kis.account_product_code);
        set_bool("KIS_ENABLE_REAL_ORDER", &mut self.kis.enable_real_order)?;
        set_string("DISCORD_WEBHOOK_URL", &mut self.discord.webhook_url);
        set_string("DATABASE_URL", &mut self.database.url);
        set_bool("ENABLE_OPENING_STRATEGY", &mut self.strategy.enable_opening_strategy)?;
        set_bool("ENABLE_PULLBACK_STRATEGY", &mut self.strategy.enable_pullback_strategy)?;
        set_bool("ENABLE_BREAKOUT_STRATEGY", &mut self.strategy.enable_breakout_strategy)?;
        set_i64("MAX_DAILY_BUY_COUNT", &mut self.risk.max_daily_buy_count)?;
        set_decimal("MAX_POSITION_RATIO", &mut self.risk.max_position_ratio)?;
        set_decimal("MAX_DAILY_LOSS_RATE", &mut self.risk.max_daily_loss_rate)?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSection {
                env: "dev".to_string(),
                trading_mode: TradingMode::Mock,
                dry_run: true,
            },
            kis: KisSection {
                base_url_mock: "https://openapivts.koreainvestment.com:29443".to_string(),
                base_url_real: "https://openapi.koreainvestment.com:9443".to_string(),
                app_key: String::new(),
                app_secret: String::new(),
                account_no: String::new(),
                account_product_code: "01".to_string(),
                enable_real_order: false,
            },
            discord: DiscordSection {
                webhook_url: String::new(),
            },
            database: DatabaseSection {
                url: "sqlite://data/kis_supply_bot.db".to_string(),
            },
            strategy: StrategySection {
                enable_opening_strategy: false,
                enable_pullback_strategy: true,
                enable_breakout_strategy: true,
                lookback_days: 5,
                min_institution_buy_days: 3,
                min_foreign_buy_days: 3,
                min_total_net_buy_amt_5d: 1_000_000_000,
                min_market_cap: 100_000_000_000,
                min_avg_trade_amount_20d: 5_000_000_000,
                max_rise_rate_5d: rust_decimal::Decimal::new(20, 2),
                max_prev_day_rise_rate: rust_decimal::Decimal::new(10, 2),
            },
            risk: RiskSection {
                max_daily_buy_count: 3,
                max_position_ratio: rust_decimal::Decimal::new(10, 2),
                max_daily_loss_rate: rust_decimal::Decimal::new(-3, 2),
                max_spread_rate: rust_decimal::Decimal::new(3, 3),
                max_intraday_rise_rate: rust_decimal::Decimal::new(7, 2),
                no_new_buy_after: "15:10:00".to_string(),
                restrict_same_sector: false,
            },
        }
    }
}

fn set_string(key: &str, target: &mut String) {
    if let Ok(value) = env::var(key) {
        *target = value;
    }
}

fn set_bool(key: &str, target: &mut bool) -> anyhow::Result<()> {
    if let Ok(value) = env::var(key) {
        *target = matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "y");
    }
    Ok(())
}

fn set_i64(key: &str, target: &mut i64) -> anyhow::Result<()> {
    if let Ok(value) = env::var(key) {
        *target = value.parse().with_context(|| format!("parse {key}"))?;
    }
    Ok(())
}

fn set_decimal(key: &str, target: &mut rust_decimal::Decimal) -> anyhow::Result<()> {
    if let Ok(value) = env::var(key) {
        *target = value.parse().with_context(|| format!("parse {key}"))?;
    }
    Ok(())
}

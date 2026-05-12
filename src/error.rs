use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("KIS credentials missing for non-mock operation")]
    MissingKisCredentials,
    #[error("real order is disabled by safety gate")]
    RealOrderDisabled,
    #[error("invalid strategy state: {0}")]
    InvalidStrategyState(String),
}

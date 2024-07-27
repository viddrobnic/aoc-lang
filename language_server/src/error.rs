use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid parameters: {0}")]
    ExtractError(#[from] serde_json::Error),
}

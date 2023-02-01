use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("invalid request arg: {0}")]
    InvalidRequestArg(#[from] reqwest::Error),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("format error: {0}")]
    FormatError(#[from] std::fmt::Error),
    #[error("unknown error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ToolError>;

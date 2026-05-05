use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustickError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Store error: {0}")]
    StoreError(String),

    #[error("Invalid state")]
    InvalidState,

    #[error("Task not found")]
    TaskNotFound,

    #[error("Terminal error: {0}")]
    TerminalError(String),
}

pub type Result<T> = std::result::Result<T, RustickError>;

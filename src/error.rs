use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfixError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dotenv error: {0}")]
    Dotenv(#[from] dotenvy::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Unsupported file format for: {0}")]
    UnsupportedFormat(PathBuf),

    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),
}

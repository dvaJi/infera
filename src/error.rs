use thiserror::Error;

#[derive(Error, Debug)]
pub enum InfsError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Provider not configured: {0}. Run `infs provider connect {0}` to set up.")]
    ProviderNotConfigured(String),

    #[error("Invalid app ID: {0}")]
    InvalidAppId(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not implemented: {0}")]
    #[allow(dead_code)]
    NotImplemented(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Auth error: {0}")]
    AuthError(String),

    #[error("API error from {provider}: HTTP {status} - {message}")]
    ApiError {
        provider: String,
        status: u16,
        message: String,
    },
}

pub mod falai;
pub mod openrouter;
pub mod registry;
pub mod replicate;
pub mod wavespeed;

use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{AppDescriptor, AuthMethod, ProviderDescriptor, RunOutput, RunResponse};
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    fn descriptor(&self) -> &ProviderDescriptor;
    fn supported_auth_methods(&self) -> Vec<AuthMethod>;
    /// Fetch the list of apps/models from the provider.
    /// When an API key is present in `config`, results are fetched live from the provider API.
    /// When no API key is configured, a static fallback list of well-known models is returned.
    /// Returns the complete list of all available apps/models from the provider.
    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError>;
    async fn run_app(
        &self,
        app_id: &str,
        input: serde_json::Value,
        config: &ProviderConfig,
    ) -> Result<RunResponse, InfsError>;
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError>;

    /// Validate credentials with a non-mutating provider API request.
    ///
    /// Providers should override this when they have a lightweight account or
    /// metadata endpoint. The default uses model discovery as a fallback.
    async fn validate_credentials(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        self.validate_config(config)?;
        self.list_apps(config).await.map(|_| ())
    }

    /// Returns true if this provider supports token-by-token streaming output.
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Stream app output, printing tokens to stdout as they arrive.
    ///
    /// Providers that support streaming should override this method.
    /// The default implementation falls back to `run_app` and prints the full response.
    async fn stream_app(
        &self,
        app_id: &str,
        input: serde_json::Value,
        config: &ProviderConfig,
    ) -> Result<(), InfsError> {
        let response = self.run_app(app_id, input, config).await?;
        match response.output {
            RunOutput::Text(text) => println!("{}", text),
            RunOutput::ImageUrls(urls) => {
                for url in urls {
                    println!("{}", url);
                }
            }
            RunOutput::Json(val) => {
                println!("{}", serde_json::to_string_pretty(&val)?)
            }
        }
        Ok(())
    }
}

pub(crate) fn validation_client() -> Result<reqwest::Client, InfsError> {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(InfsError::NetworkError)
}

pub(crate) async fn validate_response(
    response: reqwest::Response,
    provider: &str,
) -> Result<(), InfsError> {
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let message = response
        .text()
        .await
        .unwrap_or_else(|_| "Unknown error".to_string());
    Err(InfsError::ApiError {
        provider: provider.to_string(),
        status: status.as_u16(),
        message,
    })
}

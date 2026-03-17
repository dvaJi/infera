pub mod falai;
pub mod openrouter;
pub mod registry;
pub mod replicate;
pub mod wavespeed;

use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{AppDescriptor, AuthMethod, ProviderDescriptor, RunResponse};
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    fn descriptor(&self) -> &ProviderDescriptor;
    fn supported_auth_methods(&self) -> Vec<AuthMethod>;
    /// Fetch the list of apps/models from the provider.
    /// When an API key is present in `config`, results are fetched live from the provider API.
    /// When no API key is configured, a static fallback list of well-known models is returned.
    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError>;
    async fn run_app(
        &self,
        app_id: &str,
        input: serde_json::Value,
        config: &ProviderConfig,
    ) -> Result<RunResponse, InfsError>;
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError>;
}

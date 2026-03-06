use async_trait::async_trait;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{AppCategory, AppDescriptor, AuthMethod, ProviderDescriptor, RunResponse};
use super::Provider;

pub struct ReplicateProvider {
    descriptor: ProviderDescriptor,
}

impl ReplicateProvider {
    pub fn new() -> Self {
        ReplicateProvider {
            descriptor: ProviderDescriptor {
                id: "replicate".to_string(),
                display_name: "Replicate".to_string(),
                description: "Run AI models in the cloud".to_string(),
                categories: vec![AppCategory::Image],
                website: "https://replicate.com".to_string(),
            },
        }
    }
}

#[async_trait]
impl Provider for ReplicateProvider {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    
    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::ApiKey]
    }
    
    fn list_apps(&self) -> Vec<AppDescriptor> {
        vec![
            AppDescriptor {
                id: "black-forest-labs/flux-schnell".to_string(),
                provider_id: "replicate".to_string(),
                display_name: "FLUX Schnell".to_string(),
                description: "The fastest image generation model by Black Forest Labs".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "black-forest-labs/flux-dev".to_string(),
                provider_id: "replicate".to_string(),
                display_name: "FLUX Dev".to_string(),
                description: "12 billion parameter flow transformer for image generation".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "stability-ai/stable-diffusion".to_string(),
                provider_id: "replicate".to_string(),
                display_name: "Stable Diffusion".to_string(),
                description: "Text-to-image generation by Stability AI".to_string(),
                category: AppCategory::Image,
                tags: vec!["stable-diffusion".to_string(), "image".to_string()],
            },
        ]
    }
    
    async fn run_app(
        &self,
        app_id: &str,
        _input: serde_json::Value,
        _config: &ProviderConfig,
    ) -> Result<RunResponse, InfsError> {
        Err(InfsError::NotImplemented(format!(
            "Replicate provider is scaffolded but not yet implemented. App: {}. Visit https://replicate.com/docs for API details.",
            app_id
        )))
    }
    
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        if config.get_api_key().is_none() {
            return Err(InfsError::ProviderNotConfigured("replicate".to_string()));
        }
        Ok(())
    }
}

use async_trait::async_trait;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{AppCategory, AppDescriptor, AuthMethod, ProviderDescriptor, RunResponse};
use super::Provider;

pub struct FalAiProvider {
    descriptor: ProviderDescriptor,
}

impl FalAiProvider {
    pub fn new() -> Self {
        FalAiProvider {
            descriptor: ProviderDescriptor {
                id: "falai".to_string(),
                display_name: "fal.ai".to_string(),
                description: "Fast inference for image and video generation models".to_string(),
                categories: vec![AppCategory::Image],
                website: "https://fal.ai".to_string(),
            },
        }
    }
}

#[async_trait]
impl Provider for FalAiProvider {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    
    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::ApiKey]
    }
    
    fn list_apps(&self) -> Vec<AppDescriptor> {
        vec![
            AppDescriptor {
                id: "fal-ai/flux-pro".to_string(),
                provider_id: "falai".to_string(),
                display_name: "FLUX Pro".to_string(),
                description: "State-of-the-art image generation with FLUX Pro".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "fal-ai/flux-dev".to_string(),
                provider_id: "falai".to_string(),
                display_name: "FLUX Dev".to_string(),
                description: "FLUX development model for image generation".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "fal-ai/flux-dev-lora".to_string(),
                provider_id: "falai".to_string(),
                display_name: "FLUX Dev LoRA".to_string(),
                description: "FLUX Dev with LoRA fine-tuning support".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "lora".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "fal-ai/stable-diffusion-v3-medium".to_string(),
                provider_id: "falai".to_string(),
                display_name: "Stable Diffusion v3 Medium".to_string(),
                description: "Stable Diffusion v3 Medium model".to_string(),
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
            "fal.ai provider is scaffolded but not yet implemented. App: {}. Visit https://fal.ai/docs for API details.",
            app_id
        )))
    }
    
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        if config.get_api_key().is_none() {
            return Err(InfsError::ProviderNotConfigured("falai".to_string()));
        }
        Ok(())
    }
}

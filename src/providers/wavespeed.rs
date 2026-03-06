use async_trait::async_trait;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{AppCategory, AppDescriptor, AuthMethod, ProviderDescriptor, RunResponse};
use super::Provider;

pub struct WavespeedProvider {
    descriptor: ProviderDescriptor,
}

impl WavespeedProvider {
    pub fn new() -> Self {
        WavespeedProvider {
            descriptor: ProviderDescriptor {
                id: "wavespeed".to_string(),
                display_name: "WaveSpeed AI".to_string(),
                description: "Fast and affordable AI model inference".to_string(),
                categories: vec![AppCategory::Image],
                website: "https://wavespeed.ai".to_string(),
            },
        }
    }
}

#[async_trait]
impl Provider for WavespeedProvider {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    
    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::ApiKey]
    }
    
    fn list_apps(&self) -> Vec<AppDescriptor> {
        vec![
            AppDescriptor {
                id: "wavespeed-ai/flux-dev".to_string(),
                provider_id: "wavespeed".to_string(),
                display_name: "FLUX Dev".to_string(),
                description: "FLUX Dev model via WaveSpeed".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "wavespeed-ai/flux-schnell".to_string(),
                provider_id: "wavespeed".to_string(),
                display_name: "FLUX Schnell".to_string(),
                description: "Fast FLUX Schnell model via WaveSpeed".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "wavespeed-ai/wan2.1-t2v-480p".to_string(),
                provider_id: "wavespeed".to_string(),
                display_name: "Wan2.1 Text-to-Video 480p".to_string(),
                description: "Text to video generation at 480p".to_string(),
                category: AppCategory::Video,
                tags: vec!["video".to_string(), "wan".to_string()],
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
            "WaveSpeed AI provider is scaffolded but not yet implemented. App: {}. Visit https://wavespeed.ai/docs for API details.",
            app_id
        )))
    }
    
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        if config.get_api_key().is_none() {
            return Err(InfsError::ProviderNotConfigured("wavespeed".to_string()));
        }
        Ok(())
    }
}

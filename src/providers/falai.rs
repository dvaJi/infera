use async_trait::async_trait;
use serde::Deserialize;
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

    fn static_apps(&self) -> Vec<AppDescriptor> {
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
                id: "fal-ai/flux/dev".to_string(),
                provider_id: "falai".to_string(),
                display_name: "FLUX Dev".to_string(),
                description: "FLUX development model for image generation".to_string(),
                category: AppCategory::Image,
                tags: vec!["flux".to_string(), "image".to_string()],
            },
            AppDescriptor {
                id: "fal-ai/flux-lora".to_string(),
                provider_id: "falai".to_string(),
                display_name: "FLUX LoRA".to_string(),
                description: "FLUX with LoRA fine-tuning support".to_string(),
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
}

// fal.ai API response types for GET /v1/models
#[derive(Deserialize)]
struct FalModelsResponse {
    models: Vec<FalModel>,
}

#[derive(Deserialize)]
struct FalModel {
    endpoint_id: String,
    metadata: FalModelMetadata,
}

#[derive(Deserialize)]
struct FalModelMetadata {
    display_name: Option<String>,
    category: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

fn map_fal_category(category: &str) -> AppCategory {
    match category {
        "text-to-image" | "image-to-image" | "inpainting" => AppCategory::Image,
        "text-to-video" | "image-to-video" | "video" => AppCategory::Video,
        "text-to-audio" | "text-to-speech" | "audio" => AppCategory::Audio,
        "text-generation" | "chat" => AppCategory::Llm,
        _ => AppCategory::Other,
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

    /// Fetches models live from https://api.fal.ai/v1/models when an API key is configured.
    /// Falls back to a static list of well-known models when not connected.
    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError> {
        let api_key = match config.get_api_key() {
            Some(k) => k.to_string(),
            None => {
                tracing::debug!("falai: no API key configured, returning static model list");
                eprintln!(
                    "fal.ai: showing cached models. Connect with `infs provider connect falai` to see the full live catalog."
                );
                return Ok(self.static_apps());
            }
        };

        let client = reqwest::Client::new();
        // fal.ai uses "Authorization: Key <api_key>" header format
        let response = client
            .get("https://api.fal.ai/v1/models")
            .header("Authorization", format!("Key {}", api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!("falai: /v1/models returned {}: {}", status, body);
            return Err(InfsError::ApiError {
                provider: "falai".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let models_response: FalModelsResponse = response.json().await?;

        let apps = models_response
            .models
            .into_iter()
            .map(|model| {
                let category =
                    map_fal_category(model.metadata.category.as_deref().unwrap_or(""));
                AppDescriptor {
                    id: model.endpoint_id,
                    provider_id: "falai".to_string(),
                    display_name: model
                        .metadata
                        .display_name
                        .unwrap_or_else(|| "Unknown".to_string()),
                    description: model.metadata.description.unwrap_or_default(),
                    category,
                    tags: model.metadata.tags,
                }
            })
            .collect();

        Ok(apps)
    }

    async fn run_app(
        &self,
        app_id: &str,
        _input: serde_json::Value,
        _config: &ProviderConfig,
    ) -> Result<RunResponse, InfsError> {
        // TODO: implement fal.ai execution via https://fal.run/<app_id>
        // See https://docs.fal.ai/model-apis/running-models for API details.
        Err(InfsError::NotImplemented(format!(
            "fal.ai execution is not yet implemented. App: {}. See https://docs.fal.ai/model-apis/running-models",
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

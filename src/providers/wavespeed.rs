use async_trait::async_trait;
use serde::Deserialize;
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
                categories: vec![AppCategory::Image, AppCategory::Video],
                website: "https://wavespeed.ai".to_string(),
                api_key_help_url: "https://wavespeed.ai/dashboard".to_string(),
            },
        }
    }

    fn static_apps(&self) -> Vec<AppDescriptor> {
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
}

// WaveSpeed AI API response types for GET /api/v3/models
#[derive(Deserialize)]
struct WavespeedModelsResponse {
    data: Vec<WavespeedModel>,
}

#[derive(Deserialize)]
struct WavespeedModel {
    model_id: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "type", default)]
    model_type: String,
}

fn map_wavespeed_category(model_type: &str) -> AppCategory {
    match model_type {
        "text-to-image" | "image-to-image" => AppCategory::Image,
        "text-to-video" | "image-to-video" => AppCategory::Video,
        "text-to-audio" | "text-to-speech" => AppCategory::Audio,
        _ => AppCategory::Other,
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

    /// Fetches models live from https://api.wavespeed.ai/api/v3/models when an API key is configured.
    /// Falls back to a static list of well-known models when not connected.
    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError> {
        let api_key = match config.get_api_key() {
            Some(k) => k.to_string(),
            None => {
                tracing::debug!("wavespeed: no API key configured, returning static model list");
                eprintln!(
                    "WaveSpeed AI: showing cached models. Connect with `infs provider connect wavespeed` to see the full live catalog."
                );
                return Ok(self.static_apps());
            }
        };

        let client = reqwest::Client::new();
        // WaveSpeed uses standard "Authorization: Bearer <api_key>" header
        let response = client
            .get("https://api.wavespeed.ai/api/v3/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!("wavespeed: /api/v3/models returned {}: {}", status, body);
            return Err(InfsError::ApiError {
                provider: "wavespeed".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let models_response: WavespeedModelsResponse = response.json().await?;

        let apps = models_response
            .data
            .into_iter()
            .map(|m| {
                let category = map_wavespeed_category(&m.model_type);
                let display_name = m
                    .model_id
                    .split('/')
                    .last()
                    .unwrap_or(&m.model_id)
                    .replace('-', " ")
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                AppDescriptor {
                    id: m.model_id,
                    provider_id: "wavespeed".to_string(),
                    display_name,
                    description: m.description,
                    category,
                    tags: vec![],
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
        // TODO: implement WaveSpeed execution via POST /api/v3/<model_id>
        // See https://wavespeed.ai/docs/rest-api for API details.
        Err(InfsError::NotImplemented(format!(
            "WaveSpeed AI execution is not yet implemented. App: {}. See https://wavespeed.ai/docs/rest-api",
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

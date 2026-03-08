use async_trait::async_trait;
use serde::Deserialize;
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

    fn static_apps(&self) -> Vec<AppDescriptor> {
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
}

// Replicate API response types for GET /v1/models
#[derive(Deserialize)]
struct ReplicateModelsResponse {
    results: Vec<ReplicateModel>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct ReplicateModel {
    owner: String,
    name: String,
    #[serde(default)]
    description: String,
    // Replicate does not return a category field; we infer it from description/name.
}

fn infer_replicate_category(owner: &str, name: &str, description: &str) -> AppCategory {
    let text = format!("{} {} {}", owner, name, description).to_lowercase();
    if text.contains("image") || text.contains("photo") || text.contains("picture")
        || text.contains("flux") || text.contains("stable-diffusion") || text.contains("diffusion")
        || text.contains("midjourney") || text.contains("controlnet")
    {
        AppCategory::Image
    } else if text.contains("video") {
        AppCategory::Video
    } else if text.contains("audio") || text.contains("speech") || text.contains("music") {
        AppCategory::Audio
    } else if text.contains("language") || text.contains("llm") || text.contains("gpt")
        || text.contains("chat") || text.contains("text generation")
    {
        AppCategory::Llm
    } else {
        AppCategory::Other
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

    /// Fetches models live from https://api.replicate.com/v1/models when an API key is configured.
    /// Falls back to a static list of well-known models when not connected.
    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError> {
        let api_key = match config.get_api_key() {
            Some(k) => k.to_string(),
            None => {
                tracing::debug!("replicate: no API key configured, returning static model list");
                eprintln!(
                    "Replicate: showing cached models. Connect with `infs provider connect replicate` to see the full live catalog."
                );
                return Ok(self.static_apps());
            }
        };

        let client = reqwest::Client::new();
        // Replicate uses "Authorization: Token <api_key>" header format
        let response = client
            .get("https://api.replicate.com/v1/models")
            .header("Authorization", format!("Token {}", api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!("replicate: /v1/models returned {}: {}", status, body);
            return Err(InfsError::ApiError {
                provider: "replicate".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let models_response: ReplicateModelsResponse = response.json().await?;

        if let Some(next_url) = &models_response.next {
            // TODO: implement pagination to fetch all models, not just the first page
            tracing::debug!(
                "replicate: more models available at {} (pagination not yet implemented)",
                next_url
            );
        }

        let apps = models_response
            .results
            .into_iter()
            .map(|m| {
                let category = infer_replicate_category(&m.owner, &m.name, &m.description);
                AppDescriptor {
                    id: format!("{}/{}", m.owner, m.name),
                    provider_id: "replicate".to_string(),
                    display_name: format!("{}/{}", m.owner, m.name),
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
        // TODO: implement Replicate execution via POST /v1/models/{owner}/{name}/predictions
        // See https://replicate.com/docs/reference/http#predictions.create for API details.
        Err(InfsError::NotImplemented(format!(
            "Replicate execution is not yet implemented. App: {}. See https://replicate.com/docs/reference/http#predictions.create",
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

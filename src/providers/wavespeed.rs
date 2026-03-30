use super::Provider;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{
    AppCategory, AppDescriptor, AuthMethod, ListOptions, ProviderDescriptor, RunOutput, RunResponse,
};
use async_trait::async_trait;
use serde::Deserialize;

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
            AppDescriptor {
                id: "google/nano-banana-2".to_string(),
                provider_id: "wavespeed".to_string(),
                display_name: "Nano Banana 2".to_string(),
                description: "Google Nano Banana 2 text-to-image model via WaveSpeed".to_string(),
                category: AppCategory::Image,
                tags: vec!["google".to_string(), "image".to_string()],
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

// WaveSpeed AI API response types for POST /api/v3/<model_id> (task submission)
#[derive(Deserialize)]
struct WavespeedSubmitResponse {
    data: WavespeedSubmitData,
}

#[derive(Deserialize)]
struct WavespeedSubmitData {
    id: String,
}

// WaveSpeed AI API response types for GET /api/v3/predictions/<task_id> (polling)
#[derive(Deserialize)]
struct WavespeedPollResponse {
    data: WavespeedPollData,
}

#[derive(Deserialize)]
struct WavespeedPollData {
    status: String,
    #[serde(default)]
    outputs: Vec<String>,
    error: Option<String>,
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
    async fn list_apps(
        &self,
        config: &ProviderConfig,
        options: &ListOptions,
    ) -> Result<Vec<AppDescriptor>, InfsError> {
        let api_key = match config.get_api_key() {
            Some(k) => k.to_string(),
            None => {
                tracing::debug!("wavespeed: no API key configured, returning static model list");
                eprintln!(
                    "WaveSpeed AI: showing cached models. Connect with `infs provider connect wavespeed` to see the full live catalog."
                );
                let all_apps = self.static_apps();
                return Ok(apply_client_pagination(all_apps, options));
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
                    .next_back()
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

        Ok(apply_client_pagination(apps, options))
    }

    async fn run_app(
        &self,
        app_id: &str,
        input: serde_json::Value,
        config: &ProviderConfig,
    ) -> Result<RunResponse, InfsError> {
        let api_key = config
            .get_api_key()
            .ok_or_else(|| InfsError::ProviderNotConfigured("wavespeed".to_string()))?;

        let client = reqwest::Client::new();

        // Step 1: Submit the task via POST /api/v3/<model_id>
        let submit_url = format!("https://api.wavespeed.ai/api/v3/{}", app_id);
        let submit_response = client
            .post(&submit_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&input)
            .send()
            .await?;

        if !submit_response.status().is_success() {
            let status = submit_response.status();
            let body = submit_response.text().await.unwrap_or_default();
            tracing::warn!("wavespeed: POST {} returned {}: {}", app_id, status, body);
            return Err(InfsError::ApiError {
                provider: "wavespeed".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let submit_data: WavespeedSubmitResponse = submit_response.json().await?;
        let task_id = submit_data.data.id;
        tracing::debug!("wavespeed: submitted task {}", task_id);

        // Step 2: Poll GET /api/v3/predictions/<task_id> until completed or failed
        let poll_url = format!("https://api.wavespeed.ai/api/v3/predictions/{}", task_id);
        const MAX_ATTEMPTS: u32 = 60;
        const POLL_INTERVAL_SECS: u64 = 2;

        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
            }

            let poll_response = client
                .get(&poll_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;

            if !poll_response.status().is_success() {
                let status = poll_response.status();
                let body = poll_response.text().await.unwrap_or_default();
                tracing::warn!(
                    "wavespeed: GET predictions/{} returned {}: {}",
                    task_id,
                    status,
                    body
                );
                return Err(InfsError::ApiError {
                    provider: "wavespeed".to_string(),
                    status: status.as_u16(),
                    message: body,
                });
            }

            let poll_data: WavespeedPollResponse = poll_response.json().await?;
            match poll_data.data.status.as_str() {
                "completed" => {
                    tracing::debug!(
                        "wavespeed: task {} completed with {} output(s)",
                        task_id,
                        poll_data.data.outputs.len()
                    );
                    // WaveSpeed returns output URLs for both image and video models.
                    // RunOutput::ImageUrls is used for all URL outputs since there is no VideoUrls variant.
                    return Ok(RunResponse {
                        output: RunOutput::ImageUrls(poll_data.data.outputs),
                        model: app_id.to_string(),
                        provider: "wavespeed".to_string(),
                        usage: None,
                    });
                }
                "failed" => {
                    let error = poll_data.data.error.unwrap_or_else(|| {
                        "Task failed without error details from the API".to_string()
                    });
                    tracing::warn!("wavespeed: task {} failed: {}", task_id, error);
                    return Err(InfsError::ApiError {
                        provider: "wavespeed".to_string(),
                        status: 500,
                        message: format!("Generation failed: {}", error),
                    });
                }
                status => {
                    tracing::debug!(
                        "wavespeed: task {} status: {} (attempt {}/{})",
                        task_id,
                        status,
                        attempt + 1,
                        MAX_ATTEMPTS
                    );
                }
            }
        }

        Err(InfsError::ApiError {
            provider: "wavespeed".to_string(),
            status: 408,
            message: format!(
                "Task {} timed out after {} polling attempts (~{}s)",
                task_id,
                MAX_ATTEMPTS,
                MAX_ATTEMPTS as u64 * POLL_INTERVAL_SECS
            ),
        })
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        if config.get_api_key().is_none() {
            return Err(InfsError::ProviderNotConfigured("wavespeed".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_descriptor() {
        let provider = WavespeedProvider::new();
        let d = provider.descriptor();
        assert_eq!(d.id, "wavespeed");
        assert_eq!(d.display_name, "WaveSpeed AI");
        assert_eq!(d.api_key_help_url, "https://wavespeed.ai/dashboard");
    }

    #[test]
    fn test_static_apps_not_empty() {
        let provider = WavespeedProvider::new();
        let apps = provider.static_apps();
        assert!(!apps.is_empty());
        for app in &apps {
            assert_eq!(app.provider_id, "wavespeed");
        }
    }

    #[test]
    fn test_static_apps_contains_nano_banana_2() {
        let provider = WavespeedProvider::new();
        let apps = provider.static_apps();
        let nb2 = apps.iter().find(|a| a.id == "google/nano-banana-2");
        assert!(
            nb2.is_some(),
            "static apps should include google/nano-banana-2"
        );
        let nb2 = nb2.unwrap();
        assert_eq!(nb2.category, AppCategory::Image);
    }

    #[test]
    fn test_map_wavespeed_category() {
        assert_eq!(map_wavespeed_category("text-to-image"), AppCategory::Image);
        assert_eq!(map_wavespeed_category("image-to-image"), AppCategory::Image);
        assert_eq!(map_wavespeed_category("text-to-video"), AppCategory::Video);
        assert_eq!(map_wavespeed_category("image-to-video"), AppCategory::Video);
        assert_eq!(map_wavespeed_category("text-to-audio"), AppCategory::Audio);
        assert_eq!(map_wavespeed_category("text-to-speech"), AppCategory::Audio);
        assert_eq!(map_wavespeed_category("unknown"), AppCategory::Other);
    }

    #[test]
    fn test_validate_config_requires_api_key() {
        let provider = WavespeedProvider::new();
        let config = ProviderConfig::default();
        assert!(provider.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_with_api_key() {
        let provider = WavespeedProvider::new();
        let mut config = ProviderConfig::default();
        config
            .credentials
            .insert("api_key".to_string(), "test-key".to_string());
        assert!(provider.validate_config(&config).is_ok());
    }

    #[test]
    fn test_poll_response_completed_deserialization() {
        let json = r#"{"data": {"status": "completed", "outputs": ["https://example.com/image.png"], "error": null}}"#;
        let resp: WavespeedPollResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.status, "completed");
        assert_eq!(resp.data.outputs, vec!["https://example.com/image.png"]);
        assert!(resp.data.error.is_none());
    }

    #[test]
    fn test_poll_response_failed_deserialization() {
        let json = r#"{"data": {"status": "failed", "outputs": [], "error": "out of credits"}}"#;
        let resp: WavespeedPollResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.status, "failed");
        assert!(resp.data.outputs.is_empty());
        assert_eq!(resp.data.error.as_deref(), Some("out of credits"));
    }

    #[test]
    fn test_submit_response_deserialization() {
        let json = r#"{"data": {"id": "task-abc-123"}}"#;
        let resp: WavespeedSubmitResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.id, "task-abc-123");
    }
}

fn apply_client_pagination(apps: Vec<AppDescriptor>, options: &ListOptions) -> Vec<AppDescriptor> {
    let offset = options.offset();
    apps.into_iter()
        .skip(offset)
        .take(options.per_page)
        .collect()
}

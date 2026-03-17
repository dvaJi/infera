use super::Provider;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{
    AppCategory, AppDescriptor, AuthMethod, ProviderDescriptor, RunOutput, RunResponse,
};
use async_trait::async_trait;
use serde::Deserialize;

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
                api_key_help_url: "https://fal.ai/dashboard/keys".to_string(),
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

// fal.ai queue API response types
#[derive(Deserialize)]
struct FalQueueSubmitResponse {
    request_id: String,
}

#[derive(Deserialize)]
struct FalQueueStatusResponse {
    status: String,
    error: Option<String>,
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

/// Maximum number of polling attempts before timing out (~5 minutes at 2s intervals).
const FAL_MAX_POLL_ATTEMPTS: u32 = 150;
/// Seconds to wait between status poll requests.
const FAL_POLL_INTERVAL_SECS: u64 = 2;

/// Parse a fal.ai result JSON into a `RunOutput`.
///
/// - A top-level `images` array with `url` fields → `RunOutput::ImageUrls`
/// - A top-level `output` string → `RunOutput::Text`
/// - Any other shape → `RunOutput::Json`
fn parse_fal_output(result: serde_json::Value) -> RunOutput {
    // Image generation models: { "images": [{"url": "...", ...}, ...] }
    if let Some(images) = result.get("images").and_then(|v| v.as_array()) {
        let urls: Vec<String> = images
            .iter()
            .filter_map(|img| img.get("url").and_then(|u| u.as_str()).map(String::from))
            .collect();
        if !urls.is_empty() {
            return RunOutput::ImageUrls(urls);
        }
    }

    // Text output
    if let Some(text) = result.get("output").and_then(|v| v.as_str()) {
        return RunOutput::Text(text.to_string());
    }

    RunOutput::Json(result)
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
                let category = map_fal_category(model.metadata.category.as_deref().unwrap_or(""));
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
        input: serde_json::Value,
        config: &ProviderConfig,
    ) -> Result<RunResponse, InfsError> {
        let api_key = config
            .get_api_key()
            .ok_or_else(|| InfsError::ProviderNotConfigured("falai".to_string()))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(InfsError::NetworkError)?;

        // Step 1: Submit the request via POST https://queue.fal.run/<app_id>
        let submit_url = format!("https://queue.fal.run/{}", app_id);
        let submit_response = client
            .post(&submit_url)
            .header("Authorization", format!("Key {}", api_key))
            .json(&input)
            .send()
            .await?;

        if !submit_response.status().is_success() {
            let status = submit_response.status();
            let body = submit_response.text().await.unwrap_or_default();
            tracing::warn!("falai: POST {} returned {}: {}", app_id, status, body);
            return Err(InfsError::ApiError {
                provider: "falai".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let submit_data: FalQueueSubmitResponse = submit_response.json().await?;
        let request_id = submit_data.request_id;
        tracing::debug!("falai: submitted request {}", request_id);

        // Step 2: Poll GET https://queue.fal.run/<app_id>/requests/<request_id>/status
        let status_url = format!(
            "https://queue.fal.run/{}/requests/{}/status",
            app_id, request_id
        );

        for attempt in 0..FAL_MAX_POLL_ATTEMPTS {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(FAL_POLL_INTERVAL_SECS)).await;
            }

            let status_response = client
                .get(&status_url)
                .header("Authorization", format!("Key {}", api_key))
                .send()
                .await?;

            if !status_response.status().is_success() {
                let http_status = status_response.status();
                let body = status_response.text().await.unwrap_or_default();
                tracing::warn!("falai: status poll returned {}: {}", http_status, body);
                return Err(InfsError::ApiError {
                    provider: "falai".to_string(),
                    status: http_status.as_u16(),
                    message: format!("Request {}: {}", request_id, body),
                });
            }

            let status_data: FalQueueStatusResponse = status_response.json().await?;
            tracing::debug!(
                "falai: request {} status: {} (attempt {}/{})",
                request_id,
                status_data.status,
                attempt + 1,
                FAL_MAX_POLL_ATTEMPTS
            );

            match status_data.status.as_str() {
                "COMPLETED" => {
                    // Step 3: Fetch the result
                    let result_url =
                        format!("https://queue.fal.run/{}/requests/{}", app_id, request_id);
                    let result_response = client
                        .get(&result_url)
                        .header("Authorization", format!("Key {}", api_key))
                        .send()
                        .await?;

                    if !result_response.status().is_success() {
                        let http_status = result_response.status();
                        let body = result_response.text().await.unwrap_or_default();
                        tracing::warn!("falai: result fetch returned {}: {}", http_status, body);
                        return Err(InfsError::ApiError {
                            provider: "falai".to_string(),
                            status: http_status.as_u16(),
                            message: format!("Request {}: {}", request_id, body),
                        });
                    }

                    let result: serde_json::Value = result_response.json().await?;
                    let output = parse_fal_output(result);
                    return Ok(RunResponse {
                        output,
                        model: app_id.to_string(),
                        provider: "falai".to_string(),
                        usage: None,
                    });
                }
                "ERROR" => {
                    let error_msg = status_data
                        .error
                        .unwrap_or_else(|| "Request failed without error details".to_string());
                    tracing::warn!("falai: request {} failed: {}", request_id, error_msg);
                    return Err(InfsError::ApiError {
                        provider: "falai".to_string(),
                        status: 500,
                        message: format!("Request {}: {}", request_id, error_msg),
                    });
                }
                _ => {
                    // "IN_QUEUE" or "IN_PROGRESS" — keep polling
                }
            }
        }

        Err(InfsError::ApiError {
            provider: "falai".to_string(),
            status: 408,
            message: format!(
                "Request {} timed out after {} polling attempts (~{}s)",
                request_id,
                FAL_MAX_POLL_ATTEMPTS,
                FAL_MAX_POLL_ATTEMPTS as u64 * FAL_POLL_INTERVAL_SECS
            ),
        })
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        if config.get_api_key().is_none() {
            return Err(InfsError::ProviderNotConfigured("falai".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_provider_descriptor() {
        let provider = FalAiProvider::new();
        let d = provider.descriptor();
        assert_eq!(d.id, "falai");
        assert_eq!(d.display_name, "fal.ai");
        assert_eq!(d.api_key_help_url, "https://fal.ai/dashboard/keys");
    }

    #[test]
    fn test_static_apps_not_empty() {
        let provider = FalAiProvider::new();
        let apps = provider.static_apps();
        assert!(!apps.is_empty());
        for app in &apps {
            assert_eq!(app.provider_id, "falai");
        }
    }

    #[test]
    fn test_static_apps_are_image_category() {
        let provider = FalAiProvider::new();
        let apps = provider.static_apps();
        for app in &apps {
            assert_eq!(app.category, AppCategory::Image);
        }
    }

    #[test]
    fn test_map_fal_category_image() {
        assert_eq!(map_fal_category("text-to-image"), AppCategory::Image);
        assert_eq!(map_fal_category("image-to-image"), AppCategory::Image);
        assert_eq!(map_fal_category("inpainting"), AppCategory::Image);
    }

    #[test]
    fn test_map_fal_category_video() {
        assert_eq!(map_fal_category("text-to-video"), AppCategory::Video);
        assert_eq!(map_fal_category("image-to-video"), AppCategory::Video);
        assert_eq!(map_fal_category("video"), AppCategory::Video);
    }

    #[test]
    fn test_map_fal_category_audio() {
        assert_eq!(map_fal_category("text-to-audio"), AppCategory::Audio);
        assert_eq!(map_fal_category("text-to-speech"), AppCategory::Audio);
        assert_eq!(map_fal_category("audio"), AppCategory::Audio);
    }

    #[test]
    fn test_map_fal_category_llm() {
        assert_eq!(map_fal_category("text-generation"), AppCategory::Llm);
        assert_eq!(map_fal_category("chat"), AppCategory::Llm);
    }

    #[test]
    fn test_map_fal_category_other() {
        assert_eq!(map_fal_category("unknown"), AppCategory::Other);
        assert_eq!(map_fal_category(""), AppCategory::Other);
    }

    #[test]
    fn test_parse_fal_output_images() {
        let result = json!({
            "images": [
                {"url": "https://cdn.fal.ai/image1.png", "width": 512, "height": 512},
                {"url": "https://cdn.fal.ai/image2.png", "width": 512, "height": 512}
            ],
            "seed": 42
        });
        if let RunOutput::ImageUrls(urls) = parse_fal_output(result) {
            assert_eq!(urls.len(), 2);
            assert_eq!(urls[0], "https://cdn.fal.ai/image1.png");
            assert_eq!(urls[1], "https://cdn.fal.ai/image2.png");
        } else {
            panic!("Expected RunOutput::ImageUrls");
        }
    }

    #[test]
    fn test_parse_fal_output_single_image() {
        let result = json!({
            "images": [{"url": "https://cdn.fal.ai/image.png"}]
        });
        if let RunOutput::ImageUrls(urls) = parse_fal_output(result) {
            assert_eq!(urls.len(), 1);
            assert_eq!(urls[0], "https://cdn.fal.ai/image.png");
        } else {
            panic!("Expected RunOutput::ImageUrls");
        }
    }

    #[test]
    fn test_parse_fal_output_text() {
        let result = json!({"output": "Hello, world!"});
        if let RunOutput::Text(s) = parse_fal_output(result) {
            assert_eq!(s, "Hello, world!");
        } else {
            panic!("Expected RunOutput::Text");
        }
    }

    #[test]
    fn test_parse_fal_output_json_fallback() {
        let result = json!({"some_other_field": "value", "count": 3});
        assert!(matches!(parse_fal_output(result), RunOutput::Json(_)));
    }

    #[test]
    fn test_parse_fal_output_empty_images_falls_through_to_json() {
        // images array exists but has no url fields → falls through to Json
        let result = json!({"images": [{"width": 512}]});
        assert!(matches!(parse_fal_output(result), RunOutput::Json(_)));
    }

    #[test]
    fn test_queue_submit_response_deserialization() {
        let json = r#"{"request_id": "abc-123-xyz"}"#;
        let resp: FalQueueSubmitResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.request_id, "abc-123-xyz");
    }

    #[test]
    fn test_queue_status_in_queue_deserialization() {
        let json = r#"{"status": "IN_QUEUE"}"#;
        let resp: FalQueueStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "IN_QUEUE");
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_queue_status_completed_deserialization() {
        let json = r#"{"status": "COMPLETED"}"#;
        let resp: FalQueueStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "COMPLETED");
    }

    #[test]
    fn test_queue_status_error_deserialization() {
        let json = r#"{"status": "ERROR", "error": "out of credits"}"#;
        let resp: FalQueueStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "ERROR");
        assert_eq!(resp.error.as_deref(), Some("out of credits"));
    }

    #[test]
    fn test_validate_config_requires_api_key() {
        let provider = FalAiProvider::new();
        let config = ProviderConfig::default();
        assert!(provider.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_with_api_key() {
        let provider = FalAiProvider::new();
        let mut config = ProviderConfig::default();
        config
            .credentials
            .insert("api_key".to_string(), "test-key".to_string());
        assert!(provider.validate_config(&config).is_ok());
    }
}

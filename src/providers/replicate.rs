use super::Provider;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{
    AppCategory, AppDescriptor, AuthMethod, ListOptions, ProviderDescriptor, RunOutput, RunResponse,
};
use async_trait::async_trait;
use serde::Deserialize;

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
                categories: vec![
                    AppCategory::Image,
                    AppCategory::Video,
                    AppCategory::Audio,
                    AppCategory::Llm,
                    AppCategory::Other,
                ],
                website: "https://replicate.com".to_string(),
                api_key_help_url: "https://replicate.com/account/api-tokens".to_string(),
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
                description: "12 billion parameter flow transformer for image generation"
                    .to_string(),
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

// Replicate prediction API response types
#[derive(Deserialize)]
struct ReplicatePrediction {
    id: String,
    status: Option<String>,
    output: Option<serde_json::Value>,
    error: Option<String>,
}

/// Parse a Replicate prediction output into a `RunOutput`.
///
/// - A JSON string becomes `RunOutput::Text`.
/// - A JSON array of strings that contain URLs becomes `RunOutput::ImageUrls`
///   (covers images, videos, and audio file URLs).
/// - A JSON array of plain strings (e.g. LLM token chunks) is joined and
///   returned as `RunOutput::Text`.
/// - Any other JSON value is returned as `RunOutput::Json`.
fn parse_replicate_output(output: Option<serde_json::Value>) -> RunOutput {
    match output {
        None => RunOutput::Text(String::new()),
        Some(serde_json::Value::String(s)) => RunOutput::Text(s),
        Some(serde_json::Value::Array(arr)) => {
            let strings: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if strings.len() == arr.len() {
                // If any element looks like a URL, treat the whole array as file URLs
                if strings
                    .iter()
                    .any(|s| s.starts_with("http://") || s.starts_with("https://"))
                {
                    RunOutput::ImageUrls(strings)
                } else {
                    // LLM streaming output — token chunks joined together
                    RunOutput::Text(strings.join(""))
                }
            } else {
                RunOutput::Json(serde_json::Value::Array(arr))
            }
        }
        Some(val) => RunOutput::Json(val),
    }
}

fn infer_replicate_category(owner: &str, name: &str, description: &str) -> AppCategory {
    let text = format!("{} {} {}", owner, name, description).to_lowercase();
    if text.contains("image")
        || text.contains("photo")
        || text.contains("picture")
        || text.contains("flux")
        || text.contains("stable-diffusion")
        || text.contains("diffusion")
        || text.contains("midjourney")
        || text.contains("controlnet")
    {
        AppCategory::Image
    } else if text.contains("video") {
        AppCategory::Video
    } else if text.contains("audio") || text.contains("speech") || text.contains("music") {
        AppCategory::Audio
    } else if text.contains("language")
        || text.contains("llm")
        || text.contains("gpt")
        || text.contains("chat")
        || text.contains("text generation")
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
    /// Uses server-side cursor-based pagination to fetch all pages.
    async fn list_apps(
        &self,
        config: &ProviderConfig,
        options: &ListOptions,
    ) -> Result<Vec<AppDescriptor>, InfsError> {
        let api_key = match config.get_api_key() {
            Some(k) => k.to_string(),
            None => {
                tracing::debug!("replicate: no API key configured, returning static model list");
                eprintln!(
                    "Replicate: showing cached models. Connect with `infs provider connect replicate` to see the full live catalog."
                );
                let all_apps = self.static_apps();
                return Ok(apply_client_pagination(all_apps, options));
            }
        };

        let client = reqwest::Client::new();
        let mut all_models: Vec<ReplicateModel> = Vec::new();
        let mut next_url: Option<String> = Some("https://api.replicate.com/v1/models".to_string());

        while let Some(url) = next_url.take() {
            let response = client
                .get(&url)
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
            all_models.extend(models_response.results);
            next_url = models_response.next;
        }

        tracing::debug!("replicate: fetched {} models total", all_models.len());

        let apps: Vec<AppDescriptor> = all_models
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
            .ok_or_else(|| InfsError::ProviderNotConfigured("replicate".to_string()))?;

        // app_id is "owner/name" (e.g. "black-forest-labs/flux-schnell")
        let parts: Vec<&str> = app_id.splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(InfsError::InvalidAppId(format!(
                "Replicate app ID must be in 'owner/name' format, got: {}",
                app_id
            )));
        }
        let owner = parts[0];
        let name = parts[1];

        let client = reqwest::Client::new();

        // Step 1: Create a prediction
        let create_url = format!(
            "https://api.replicate.com/v1/models/{}/{}/predictions",
            owner, name
        );
        let create_body = serde_json::json!({ "input": input });

        let create_response = client
            .post(&create_url)
            .header("Authorization", format!("Token {}", api_key))
            .json(&create_body)
            .send()
            .await?;

        let create_status = create_response.status();
        if !create_status.is_success() {
            let body = create_response.text().await.unwrap_or_default();
            tracing::warn!(
                "replicate: create prediction returned {}: {}",
                create_status,
                body
            );
            return Err(InfsError::ApiError {
                provider: "replicate".to_string(),
                status: create_status.as_u16(),
                message: body,
            });
        }

        let prediction: ReplicatePrediction = create_response.json().await?;
        tracing::debug!("replicate: created prediction {}", prediction.id);

        // Step 2: Poll until succeeded, failed, or canceled
        let prediction_url = format!("https://api.replicate.com/v1/predictions/{}", prediction.id);
        // Poll up to 150 times with 2-second intervals (~5 minutes total)
        let max_polls = 150;
        for _ in 0..max_polls {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let poll_response = client
                .get(&prediction_url)
                .header("Authorization", format!("Token {}", api_key))
                .send()
                .await?;

            let poll_status = poll_response.status();
            if !poll_status.is_success() {
                let body = poll_response.text().await.unwrap_or_default();
                return Err(InfsError::ApiError {
                    provider: "replicate".to_string(),
                    status: poll_status.as_u16(),
                    message: body,
                });
            }

            let current: ReplicatePrediction = poll_response.json().await?;
            tracing::debug!(
                "replicate: prediction {} status: {:?}",
                current.id,
                current.status
            );

            match current.status.as_deref() {
                Some("succeeded") => {
                    let output = parse_replicate_output(current.output);
                    return Ok(RunResponse {
                        output,
                        model: app_id.to_string(),
                        provider: "replicate".to_string(),
                        usage: None,
                    });
                }
                Some("failed") | Some("canceled") => {
                    let error_msg = current
                        .error
                        .unwrap_or_else(|| "Prediction failed".to_string());
                    return Err(InfsError::ApiError {
                        provider: "replicate".to_string(),
                        status: 500,
                        message: error_msg,
                    });
                }
                _ => {
                    // "starting" or "processing" — keep polling
                }
            }
        }

        Err(InfsError::ApiError {
            provider: "replicate".to_string(),
            status: 408,
            message: "Prediction timed out after waiting 5 minutes".to_string(),
        })
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        if config.get_api_key().is_none() {
            return Err(InfsError::ProviderNotConfigured("replicate".to_string()));
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
        let provider = ReplicateProvider::new();
        let d = provider.descriptor();
        assert_eq!(d.id, "replicate");
        assert_eq!(d.display_name, "Replicate");
        assert_eq!(
            d.api_key_help_url,
            "https://replicate.com/account/api-tokens"
        );
    }

    #[test]
    fn test_descriptor_covers_all_categories() {
        let provider = ReplicateProvider::new();
        let cats = &provider.descriptor().categories;
        assert!(cats.contains(&AppCategory::Image));
        assert!(cats.contains(&AppCategory::Video));
        assert!(cats.contains(&AppCategory::Audio));
        assert!(cats.contains(&AppCategory::Llm));
        assert!(cats.contains(&AppCategory::Other));
    }

    #[test]
    fn test_static_apps_not_empty() {
        let provider = ReplicateProvider::new();
        let apps = provider.static_apps();
        assert!(!apps.is_empty());
        for app in &apps {
            assert_eq!(app.provider_id, "replicate");
        }
    }

    #[test]
    fn test_infer_replicate_category_image() {
        assert_eq!(
            infer_replicate_category("black-forest-labs", "flux-schnell", "image generation"),
            AppCategory::Image
        );
    }

    #[test]
    fn test_infer_replicate_category_video() {
        assert_eq!(
            infer_replicate_category("some-org", "video-gen", "video synthesis"),
            AppCategory::Video
        );
    }

    #[test]
    fn test_infer_replicate_category_audio() {
        assert_eq!(
            infer_replicate_category("some-org", "whisper", "speech recognition"),
            AppCategory::Audio
        );
    }

    #[test]
    fn test_infer_replicate_category_llm() {
        assert_eq!(
            infer_replicate_category("meta", "llama-2", "language model"),
            AppCategory::Llm
        );
    }

    #[test]
    fn test_infer_replicate_category_other() {
        assert_eq!(
            infer_replicate_category("some-org", "some-model", "some description"),
            AppCategory::Other
        );
    }

    #[test]
    fn test_parse_output_none() {
        assert!(matches!(parse_replicate_output(None), RunOutput::Text(s) if s.is_empty()));
    }

    #[test]
    fn test_parse_output_string() {
        let out = parse_replicate_output(Some(json!("Hello, world!")));
        if let RunOutput::Text(s) = out {
            assert_eq!(s, "Hello, world!");
        } else {
            panic!("Expected RunOutput::Text");
        }
    }

    #[test]
    fn test_parse_output_url_array() {
        let out = parse_replicate_output(Some(json!([
            "https://example.com/image1.png",
            "https://example.com/image2.png"
        ])));
        if let RunOutput::ImageUrls(urls) = out {
            assert_eq!(urls.len(), 2);
            assert_eq!(urls[0], "https://example.com/image1.png");
        } else {
            panic!("Expected RunOutput::ImageUrls");
        }
    }

    #[test]
    fn test_parse_output_token_array() {
        // LLM streaming: array of token strings (no URLs)
        let out = parse_replicate_output(Some(json!(["Hello", ", ", "world", "!"])));
        if let RunOutput::Text(s) = out {
            assert_eq!(s, "Hello, world!");
        } else {
            panic!("Expected RunOutput::Text");
        }
    }

    #[test]
    fn test_parse_output_json_object() {
        let out = parse_replicate_output(Some(json!({"key": "value"})));
        if let RunOutput::Json(val) = out {
            assert_eq!(val["key"], "value");
        } else {
            panic!("Expected RunOutput::Json");
        }
    }

    #[test]
    fn test_parse_output_mixed_array() {
        // Array with non-string elements → Json
        let out = parse_replicate_output(Some(json!([1, 2, 3])));
        assert!(matches!(out, RunOutput::Json(_)));
    }
}

fn apply_client_pagination(apps: Vec<AppDescriptor>, options: &ListOptions) -> Vec<AppDescriptor> {
    let offset = options.offset();
    apps.into_iter()
        .skip(offset)
        .take(options.per_page)
        .collect()
}

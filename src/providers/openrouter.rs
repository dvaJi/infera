use super::Provider;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{
    AppCategory, AppDescriptor, AuthMethod, ProviderDescriptor, RunOutput, RunResponse, UsageInfo,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct OpenRouterProvider {
    descriptor: ProviderDescriptor,
}

impl OpenRouterProvider {
    pub fn new() -> Self {
        OpenRouterProvider {
            descriptor: ProviderDescriptor {
                id: "openrouter".to_string(),
                display_name: "OpenRouter".to_string(),
                description: "Unified API for hundreds of LLM models".to_string(),
                categories: vec![AppCategory::Llm],
                website: "https://openrouter.ai".to_string(),
                api_key_help_url: "https://openrouter.ai/keys".to_string(),
            },
        }
    }

    fn static_apps(&self) -> Vec<AppDescriptor> {
        vec![
            AppDescriptor {
                id: "openai/gpt-4o".to_string(),
                provider_id: "openrouter".to_string(),
                display_name: "GPT-4o".to_string(),
                description: "OpenAI's most capable multimodal model".to_string(),
                category: AppCategory::Llm,
                tags: vec!["openai".to_string(), "gpt".to_string()],
            },
            AppDescriptor {
                id: "openai/gpt-4o-mini".to_string(),
                provider_id: "openrouter".to_string(),
                display_name: "GPT-4o Mini".to_string(),
                description: "Affordable and capable small model from OpenAI".to_string(),
                category: AppCategory::Llm,
                tags: vec!["openai".to_string(), "gpt".to_string()],
            },
            AppDescriptor {
                id: "anthropic/claude-sonnet-4-5".to_string(),
                provider_id: "openrouter".to_string(),
                display_name: "Claude Sonnet 4.5".to_string(),
                description: "Anthropic's latest Claude Sonnet model".to_string(),
                category: AppCategory::Llm,
                tags: vec!["anthropic".to_string(), "claude".to_string()],
            },
            AppDescriptor {
                id: "google/gemini-flash-1.5".to_string(),
                provider_id: "openrouter".to_string(),
                display_name: "Gemini Flash 1.5".to_string(),
                description: "Google's fast and efficient Gemini model".to_string(),
                category: AppCategory::Llm,
                tags: vec!["google".to_string(), "gemini".to_string()],
            },
            AppDescriptor {
                id: "meta-llama/llama-3.1-8b-instruct".to_string(),
                provider_id: "openrouter".to_string(),
                display_name: "Llama 3.1 8B Instruct (free)".to_string(),
                description: "Meta's Llama 3.1 8B model, available for free".to_string(),
                category: AppCategory::Llm,
                tags: vec!["meta".to_string(), "llama".to_string(), "free".to_string()],
            },
            AppDescriptor {
                id: "mistralai/mistral-7b-instruct".to_string(),
                provider_id: "openrouter".to_string(),
                display_name: "Mistral 7B Instruct (free)".to_string(),
                description: "Mistral AI's 7B instruction model, available for free".to_string(),
                category: AppCategory::Llm,
                tags: vec!["mistral".to_string(), "free".to_string()],
            },
        ]
    }
}

/// OpenRouter model as returned by GET /api/v1/models
#[derive(Deserialize)]
struct OpenRouterModel {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
}

#[derive(Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    model: Option<String>,
    usage: Option<OpenRouterUsage>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[async_trait]
impl Provider for OpenRouterProvider {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }

    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::ApiKey]
    }

    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError> {
        let api_key = match config.get_api_key() {
            Some(k) => k.to_string(),
            None => {
                tracing::debug!("openrouter: no API key configured, returning static model list");
                eprintln!(
                    "OpenRouter: showing cached models. Connect with `infs provider connect openrouter` to see the full live catalog."
                );
                return Ok(self.static_apps());
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .get("https://openrouter.ai/api/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/dvaJi/infs")
            .header("X-Title", "infs")
            .send()
            .await?;

        if !response.status().is_success() {
            tracing::warn!(
                "openrouter: /api/v1/models returned {}, falling back to static list",
                response.status()
            );
            return Ok(self.static_apps());
        }

        let models_response: OpenRouterModelsResponse = response.json().await?;

        let apps = models_response
            .data
            .into_iter()
            .map(|m| AppDescriptor {
                id: m.id,
                provider_id: "openrouter".to_string(),
                display_name: m.name,
                description: m.description,
                category: AppCategory::Llm,
                tags: vec![],
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
            .ok_or_else(|| InfsError::ProviderNotConfigured("openrouter".to_string()))?;

        // Normalize input: accept {"prompt": "..."} or {"messages": [...]}
        let messages = if let Some(prompt) = input.get("prompt").and_then(|v| v.as_str()) {
            vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }]
        } else if let Some(messages_val) = input.get("messages") {
            serde_json::from_value(messages_val.clone())?
        } else {
            return Err(InfsError::InvalidInput(
                "Input must have 'prompt' string or 'messages' array".to_string(),
            ));
        };

        let request = ChatCompletionRequest {
            model: app_id.to_string(),
            messages,
        };

        let client = reqwest::Client::new();
        let response = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/dvaJi/infs")
            .header("X-Title", "infs")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(InfsError::ApiError {
                provider: "openrouter".to_string(),
                status: status.as_u16(),
                message: error_text,
            });
        }

        let completion: ChatCompletionResponse = response.json().await?;

        let content = completion
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(RunResponse {
            output: RunOutput::Text(content),
            model: completion.model.unwrap_or_else(|| app_id.to_string()),
            provider: "openrouter".to_string(),
            usage: completion.usage.map(|u| UsageInfo {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> {
        if config.get_api_key().is_none() {
            return Err(InfsError::ProviderNotConfigured("openrouter".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn normalize_input(input: serde_json::Value) -> Result<Vec<ChatMessage>, InfsError> {
        if let Some(prompt) = input.get("prompt").and_then(|v| v.as_str()) {
            Ok(vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }])
        } else if let Some(messages_val) = input.get("messages") {
            Ok(serde_json::from_value(messages_val.clone())?)
        } else {
            Err(InfsError::InvalidInput(
                "Input must have 'prompt' string or 'messages' array".to_string(),
            ))
        }
    }

    #[test]
    fn test_input_prompt_normalization() {
        let messages = normalize_input(json!({"prompt": "Hello, world!"})).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello, world!");
    }

    #[test]
    fn test_input_messages_passthrough() {
        let input = json!({
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hi"}
            ]
        });
        let messages = normalize_input(input).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
    }

    #[test]
    fn test_input_invalid_returns_error() {
        let result = normalize_input(json!({"something_else": "value"}));
        assert!(result.is_err());
        if let Err(InfsError::InvalidInput(msg)) = result {
            assert!(msg.contains("'prompt'"));
        } else {
            panic!("Expected InvalidInput error");
        }
    }

    #[test]
    fn test_provider_descriptor() {
        let provider = OpenRouterProvider::new();
        let d = provider.descriptor();
        assert_eq!(d.id, "openrouter");
        assert_eq!(d.display_name, "OpenRouter");
        assert_eq!(d.api_key_help_url, "https://openrouter.ai/keys");
    }

    #[test]
    fn test_static_apps_not_empty() {
        let provider = OpenRouterProvider::new();
        let apps = provider.static_apps();
        assert!(!apps.is_empty());
        for app in &apps {
            assert_eq!(app.provider_id, "openrouter");
        }
    }
}

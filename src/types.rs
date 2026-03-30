use crate::error::InfsError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProviderId(pub String);

#[allow(dead_code)]
impl ProviderId {
    pub fn new(s: impl Into<String>) -> Self {
        ProviderId(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppId {
    pub provider: String,
    pub app: String,
}

impl AppId {
    pub fn parse(s: &str) -> Result<Self, InfsError> {
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(InfsError::InvalidAppId(format!(
                "'{}' is not a valid app ID. Expected format: provider/app-id",
                s
            )));
        }
        Ok(AppId {
            provider: parts[0].to_string(),
            app: parts[1].to_string(),
        })
    }
}

impl std::fmt::Display for AppId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.provider, self.app)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppCategory {
    Image,
    Llm,
    Audio,
    Video,
    Other,
}

impl std::fmt::Display for AppCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppCategory::Image => write!(f, "image"),
            AppCategory::Llm => write!(f, "llm"),
            AppCategory::Audio => write!(f, "audio"),
            AppCategory::Video => write!(f, "video"),
            AppCategory::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDescriptor {
    pub id: String,
    pub provider_id: String,
    pub display_name: String,
    pub description: String,
    pub category: AppCategory,
    pub tags: Vec<String>,
}

impl AppDescriptor {
    pub fn full_id(&self) -> String {
        format!("{}/{}", self.provider_id, self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub categories: Vec<AppCategory>,
    pub website: String,
    /// URL where the user can obtain an API key for this provider.
    pub api_key_help_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey,
    OAuth,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::ApiKey => write!(f, "API Key"),
            AuthMethod::OAuth => write!(f, "OAuth"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderConnectionStatus {
    Connected,
    NotConnected,
}

impl std::fmt::Display for ProviderConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderConnectionStatus::Connected => write!(f, "Connected"),
            ProviderConnectionStatus::NotConnected => write!(f, "Not Connected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RunRequest {
    pub app_id: String,
    pub provider_id: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub page: usize,
    pub per_page: usize,
}

impl ListOptions {
    pub fn new(page: usize, per_page: usize) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.max(1),
        }
    }

    pub fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RunOutput {
    Text(String),
    ImageUrls(Vec<String>),
    Json(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResponse {
    pub output: RunOutput,
    pub model: String,
    pub provider: String,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_id_parse_valid() {
        let app_id = AppId::parse("openrouter/anthropic/claude-sonnet-4-5").unwrap();
        assert_eq!(app_id.provider, "openrouter");
        assert_eq!(app_id.app, "anthropic/claude-sonnet-4-5");
    }

    #[test]
    fn test_app_id_parse_complex() {
        // openrouter apps have slashes in the app part
        let app_id = AppId::parse("openrouter/openai/gpt-4o").unwrap();
        assert_eq!(app_id.provider, "openrouter");
        assert_eq!(app_id.app, "openai/gpt-4o");
    }

    #[test]
    fn test_app_id_parse_invalid_no_slash() {
        assert!(AppId::parse("invalid-format").is_err());
    }

    #[test]
    fn test_app_id_parse_empty() {
        assert!(AppId::parse("").is_err());
    }

    #[test]
    fn test_app_id_parse_empty_app() {
        assert!(AppId::parse("openrouter/").is_err());
    }

    #[test]
    fn test_app_id_parse_empty_provider() {
        assert!(AppId::parse("/gpt-4o").is_err());
    }

    #[test]
    fn test_app_id_display() {
        let app_id = AppId {
            provider: "openrouter".to_string(),
            app: "gpt-4o".to_string(),
        };
        assert_eq!(app_id.to_string(), "openrouter/gpt-4o");
    }
}

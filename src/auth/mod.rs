use std::collections::HashMap;
use dialoguer::{Input, Password};
use crate::error::InfsError;
use crate::types::AuthMethod;

pub struct AuthMethodDescriptor {
    pub method: AuthMethod,
    pub fields: Vec<AuthField>,
}

pub struct AuthField {
    pub key: String,
    pub label: String,
    pub secret: bool,
    pub help: String,
}

pub fn get_api_key_descriptor(provider_name: &str, help_url: &str) -> AuthMethodDescriptor {
    AuthMethodDescriptor {
        method: AuthMethod::ApiKey,
        fields: vec![
            AuthField {
                key: "api_key".to_string(),
                label: format!("{} API Key", provider_name),
                secret: true,
                help: format!("Get your API key from: {}", help_url),
            },
        ],
    }
}

pub fn prompt_credentials(descriptor: &AuthMethodDescriptor) -> Result<HashMap<String, String>, InfsError> {
    let mut credentials = HashMap::new();
    
    for field in &descriptor.fields {
        if !field.help.is_empty() {
            eprintln!("  {}", field.help);
        }
        
        let value = if field.secret {
            Password::new()
                .with_prompt(&field.label)
                .interact()
                .map_err(|e| InfsError::AuthError(format!("Failed to read input: {}", e)))?
        } else {
            Input::new()
                .with_prompt(&field.label)
                .interact_text()
                .map_err(|e| InfsError::AuthError(format!("Failed to read input: {}", e)))?
        };
        
        credentials.insert(field.key.clone(), value);
    }
    
    Ok(credentials)
}

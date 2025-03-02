use std::env;
use std::error::Error;
use tracing::info;
use async_trait::async_trait;

use crate::types::{
    Content, ContentPart, GenerateContentRequest, GenerateContentResponse, PartResponse, Role
};
use crate::gemini_client::GeminiClient;

/// Asynchronous trait to represent an LLM provider.
#[async_trait]
pub trait LLMProvider {
    async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>>;
}

/// A Gemini provider that implements the LLMProvider trait asynchronously.
pub struct GeminiProvider {
    pub client: GeminiClient,
}

impl GeminiProvider {
    /// Creates a new GeminiProvider using the GEMINI_API_KEY environment variable.
    pub fn new() -> Self {
        let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
        let client = GeminiClient::new(api_key);
        Self { client }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        info!("Calling Gemini API...");
        // Log just the first line for brevity
        let first_line = prompt.lines().next().unwrap_or("").to_string();
        if first_line.len() > 50 {
            info!("Prompt first line: {}...", &first_line[..50]);
        } else {
            info!("Prompt first line: {}", first_line);
        }

        // Construct the request with one user message.
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![ContentPart::Text(prompt.to_string())],
                role: Role::User,
            }],
            tools: None,
        };

        let response: GenerateContentResponse = self
            .client
            .generate_content("gemini-2.0-flash", &request)
            .await?;

        // Extract and return the first text candidate from the response.
        if let Some(candidates) = response.candidates {
            if let Some(candidate) = candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    return match part {
                        PartResponse::Text(text) => {
                            let preview = if text.len() > 50 {
                                format!("{}...", &text[..50])
                            } else {
                                text.clone()
                            };
                            info!("Received response: {}", preview);
                            Ok(text.clone())
                        },
                        PartResponse::FunctionCall(function_call) => Ok(format!(
                            "Function call: {} with args: {}",
                            function_call.name, function_call.arguments
                        )),
                        PartResponse::FunctionResponse(function_response) => Ok(format!(
                            "Function response: {} with payload: {:?}",
                            function_response.name, function_response.response.content
                        )),
                    };
                }
            }
        }

        Err("No valid candidate returned from Gemini API".into())
    }
}

/// Factory function to select the proper LLM provider.
pub fn get_llm_provider(provider_name: &str) -> Result<Box<dyn LLMProvider + Send + Sync>, Box<dyn Error>> {
    match provider_name.to_lowercase().as_str() {
        "gemini" => Ok(Box::new(GeminiProvider::new())),
        other => Err(format!("Unknown provider: {}", other).into()),
    }
}

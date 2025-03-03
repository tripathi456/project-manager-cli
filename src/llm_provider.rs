use std::env;
use std::error::Error;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use tracing::info;

use async_trait::async_trait;

use crate::types::{
    Content, ContentPart, GenerateContentRequest, GenerateContentResponse, PartResponse, Role
};
use crate::gemini_client::{GeminiClient, GeminiError, init_step_processors};

/// Asynchronous trait to represent an LLM provider.
#[async_trait]
pub trait LLMProvider {
    async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>>;
    
    /// Call the API with a specific step number to use step-specific processing
    async fn call_api_for_step(&self, prompt: &str, step: u32) -> Result<String, Box<dyn Error>>;
}

/// A Gemini provider that implements the LLMProvider trait asynchronously.
pub struct GeminiProvider {
    pub client: GeminiClient,
    function_handlers: HashMap<String, Box<dyn Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>>,
}

impl GeminiProvider {
    /// Creates a new GeminiProvider using the GEMINI_API_KEY environment variable.
    pub fn new() -> Self {
        let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
        let client = GeminiClient::new(api_key);
        
        // Initialize step processors
        init_step_processors();
        
        // Initialize function handlers (empty for now)
        let function_handlers = HashMap::new();
        
        Self { client, function_handlers }
    }
    
    /// Add a function handler
    pub fn add_function_handler<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        self.function_handlers.insert(name.to_string(), Box::new(handler));
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Default to step 0 (no special processing)
        self.call_api_for_step(prompt, 0).await
    }
    
    async fn call_api_for_step(&self, prompt: &str, step: u32) -> Result<String, Box<dyn Error>> {
        info!("Calling Gemini API for step {}...", step);
        // Log just the first line for brevity
        let first_line = prompt.lines().next().unwrap_or("").to_string();
        if first_line.len() > 50 {
            info!("Prompt first line: {}...", &first_line[..50]);
        } else {
            info!("Prompt first line: {}", first_line);
        }

        // Use the step-specific processing
        let response = self.client.process_step(
            "gemini-2.0-flash-001", 
            prompt, 
            step, 
            &self.function_handlers
        ).await?;
        
        let preview = if response.len() > 50 {
            format!("{}...", &response[..50])
        } else {
            response.clone()
        };
        info!("Received response: {}", preview);
        
        Ok(response)
    }
}

/// Factory function to select the proper LLM provider.
pub fn get_llm_provider(provider_name: &str) -> Result<Box<dyn LLMProvider + Send + Sync>, Box<dyn Error>> {
    match provider_name.to_lowercase().as_str() {
        "gemini" => Ok(Box::new(GeminiProvider::new())),
        other => Err(format!("Unknown provider: {}", other).into()),
    }
}

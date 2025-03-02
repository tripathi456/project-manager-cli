// Re-exports for testing
pub mod types;
pub mod gemini_client;
pub mod llm_provider;
pub mod pipeline;
pub mod template_loader;

// Import the types directly for convenience
pub use types::{
    Content, ContentPart, GenerateContentRequest, GenerateContentResponse, 
    PartResponse, Role, Candidate
};

use tracing::info;

/// Helper function for logging multiline prompts.
pub fn log_prompt(title: &str, prompt_text: &str) {
    info!("{}", title);
    for (i, line) in prompt_text.lines().enumerate() {
        info!("  {}: {}", i + 1, line);
    }
}
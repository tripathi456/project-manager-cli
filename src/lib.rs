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

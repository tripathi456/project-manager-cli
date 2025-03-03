use std::env;
use std::error::Error;

use dotenvy::dotenv;

use docgen::gemini_client::{GeminiClient, init_step_processors};
use docgen::types::{
    Content, ContentPart, DynamicRetrieval, DynamicRetrievalConfig, GenerateContentRequest,
    PartResponse, Role, ToolConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get API key from environment
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    // Initialize the Gemini client
    let client = GeminiClient::new(api_key);
    let model_name = "gemini-2.0-flash-001"; // Use the appropriate model

    // Initialize step processors (required for process_step)
    init_step_processors();

    // Example 1: Using process_step for step 2 (Domain Analysis with Google Search)
    println!("Example 1: Using process_step for Domain Analysis (step 2)");
    let prompt = "What's the weather like in London, UK?";
    let response = client.process_step(model_name, prompt, 2, &std::collections::HashMap::new()).await?;
    println!("Response: {}\n", response);

    // Example 2: Using generate_content_with_google_search directly
    println!("Example 2: Using generate_content_with_google_search directly");
    let prompt = "What are the latest developments in quantum computing?";
    let response = client.generate_content_with_google_search(model_name, prompt).await?;
    println!("Response: {}\n", response);

    // Example 3: Using generate_content with manual configuration
    println!("Example 3: Using generate_content with manual configuration");
    let prompt = "What are the top tourist attractions in Paris, France?";
    
    let request = GenerateContentRequest {
        contents: vec![Content {
            parts: vec![ContentPart::Text(prompt.to_string())],
            role: Role::User,
        }],
        tools: Some(vec![ToolConfig::DynamicRetieval {
            google_search_retrieval: DynamicRetrieval {
                dynamic_retrieval_config: DynamicRetrievalConfig {
                    mode: "MODE_DYNAMIC".to_string(),
                    dynamic_threshold: 0.5,
                },
            },
        }]),
    };

    let response = client.generate_content(model_name, &request).await?;
    
    if let Some(candidates) = response.candidates {
        if let Some(first) = candidates.first() {
            for part in &first.content.parts {
                match part {
                    PartResponse::Text(text) => println!("Response: {}", text),
                    _ => println!("Received non-text response"),
                }
            }
        }
    }

    Ok(())
}

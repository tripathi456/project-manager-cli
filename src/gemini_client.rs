use std::collections::HashMap;
use std::sync::OnceLock;
use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use crate::types::{
    Content, ContentPart, DynamicRetrieval, DynamicRetrievalConfig, FunctionResponse,
    FunctionResponsePayload, GenerateContentRequest, GenerateContentResponse, PartResponse, Role,
    ToolConfig,
};

#[derive(Debug, thiserror::Error)]
pub enum GeminiError {
    #[error("HTTP Error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API Error: {0}")]
    Api(String),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Function execution error: {0}")]
    FunctionExecution(String),
}

pub struct GeminiClient {
    api_key: String,
    http_client: Client,
    api_url: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        GeminiClient {
            api_key,
            http_client: Client::new(),
            api_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    pub async fn generate_content(
        &self,
        model: &str,
        request: &GenerateContentRequest,
    ) -> Result<GenerateContentResponse, GeminiError> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.api_url, model, self.api_key
        );

        let response = self.http_client.post(&url).json(request).send().await?;

        if response.status().is_success() {
            let content: GenerateContentResponse = response.json().await?;
            Ok(content)
        } else {
            let error_text = response.text().await?;
            Err(GeminiError::Api(error_text))
        }
    }

    pub async fn generate_content_with_function_calling(
        &self,
        model: &str,
        mut request: GenerateContentRequest,
        function_handlers: &HashMap<
            String,
            Box<dyn Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
        >,
    ) -> Result<GenerateContentResponse, GeminiError> {
        loop {
            let response = self.generate_content(model, &request).await?;

            if let Some(candidates) = &response.candidates {
                if let Some(candidate) = candidates.first() {
                    if let Some(part) = candidate.content.parts.first() {
                        match part {
                            PartResponse::Text(_) => return Ok(response),
                            PartResponse::FunctionCall(function_call) => {
                                if let Some(handler) = function_handlers.get(&function_call.name) {
                                    match handler(&mut function_call.arguments.clone()) {
                                        Ok(result) => {
                                            request.contents.push(Content {
                                                parts: vec![ContentPart::FunctionCall(
                                                    function_call.clone(),
                                                )],
                                                role: Role::User,
                                            });

                                            request.contents.push(Content {
                                                parts: vec![ContentPart::FunctionResponse(
                                                    FunctionResponse {
                                                        name: function_call.name.clone(),
                                                        response: FunctionResponsePayload {
                                                            content: result,
                                                        },
                                                    },
                                                )],
                                                role: Role::Tool,
                                            });
                                        }
                                        Err(e) => return Err(GeminiError::FunctionExecution(e)),
                                    }
                                } else {
                                    return Err(GeminiError::FunctionExecution(format!(
                                        "Unknown function: {}",
                                        function_call.name
                                    )));
                                }
                            }
                            PartResponse::FunctionResponse(_) => return Ok(response),
                        }
                    } else {
                        return Ok(response);
                    }
                } else {
                    return Ok(response);
                }
            } else {
                return Ok(response);
            }
        }
    }

    /// Example helper that demonstrates how to enable Google Search grounding
    /// when generating content with Gemini.
    pub async fn generate_content_with_google_search(
        &self,
        model: &str,
        user_text: &str,
    ) -> Result<String, GeminiError> {
        // Construct the request
        let request = GenerateContentRequest {
            contents: vec![Content {
                role: Role::User,
                parts: vec![ContentPart::Text(user_text.to_string())],
            }],
            // Insert a single tool config for Google Search retrieval
            tools: Some(vec![ToolConfig::DynamicRetieval {
                google_search_retrieval: DynamicRetrieval {
                    dynamic_retrieval_config: DynamicRetrievalConfig {
                        mode: "live".to_string(),
                        dynamic_threshold: 0.5,
                    },
                },
            }]),
            temperature: Some(1.0),
            top_p: Some(0.95),
            top_k: Some(40),
            max_output_tokens: Some(8192),
            response_mime_type: Some("text/plain".to_string()),
        };

        // Call the standard generate_content method
        let response = self.generate_content(model, &request).await?;

        // Extract text from the first candidate
        if let Some(candidates) = response.candidates {
            if let Some(first) = candidates.first() {
                if let Some(PartResponse::Text(text)) = first.content.parts.first() {
                    return Ok(text.clone());
                }
            }
        }

        Err(GeminiError::Api("No text response received".to_string()))
    }

    /// Process a prompt using the appropriate method based on the step number.
    /// For step 2 (Domain Analysis), it uses Google Search grounding.
    /// For all other steps, it uses function calling.
    pub async fn process_step(
        &self,
        model: &str,
        prompt: &str,
        step: u32,
        function_handlers: &HashMap<
            String,
            Box<dyn Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
        >,
    ) -> Result<String, GeminiError> {
        // Get the map of step -> StepProcessor variant
        let processors_map = STEP_PROCESSORS
            .get()
            .expect("Step processors not initialized");

        // Look up which processor this step uses
        if let Some(step_processor) = processors_map.get(&step) {
            match step_processor {
                StepProcessor::DomainAnalysis => {
                    // Use the specialized domain_analysis_processor
                    domain_analysis_processor(self, model, prompt, function_handlers).await
                }
                StepProcessor::FunctionCalling => {
                    // Use the default processor
                    default_processor(self, model, prompt, function_handlers).await
                }
            }
        } else {
            // If the step is not in the map, default to function-calling
            default_processor(self, model, prompt, function_handlers).await
        }
    }

    /// Process a prompt using function calling (default for most steps)
    async fn process_with_function_calling(
        &self,
        model: &str,
        prompt: &str,
        function_handlers: &HashMap<
            String,
            Box<dyn Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
        >,
    ) -> Result<String, GeminiError> {
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![ContentPart::Text(prompt.to_string())],
                role: Role::User,
            }],
            tools: None,
            temperature: Some(0.7),
            top_p: Some(0.95),
            top_k: Some(40),
            max_output_tokens: Some(8192),
            response_mime_type: Some("text/plain".to_string()),
        };

        let response = self
            .generate_content_with_function_calling(model, request, function_handlers)
            .await?;

        // Extract text from the response
        if let Some(candidates) = response.candidates {
            if let Some(first) = candidates.first() {
                if let Some(PartResponse::Text(text)) = first.content.parts.first() {
                    return Ok(text.clone());
                }
            }
        }

        Err(GeminiError::Api("No text response received".to_string()))
    }

    /// Process a prompt using Google Search grounding (for step 2 - domain analysis)
    async fn process_with_google_search(
        &self,
        model: &str,
        prompt: &str,
        _function_handlers: &HashMap<
            String,
            Box<dyn Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
        >,
    ) -> Result<String, GeminiError> {
        self.generate_content_with_google_search(model, prompt).await
    }
}

/// Represents the different kinds of step processors we can have.
enum StepProcessor {
    /// For domain analysis (e.g., step 2) with Google Search grounding.
    DomainAnalysis,
    /// For all other steps, use the default function-calling approach.
    FunctionCalling,
}

// Global mapping of step numbers to processor types
static STEP_PROCESSORS: OnceLock<HashMap<u32, StepProcessor>> = OnceLock::new();

// --- Processor implementations ---

// Domain Analysis Processor (step 2)
fn domain_analysis_processor<'a>(
    client: &'a GeminiClient,
    model: &'a str,
    prompt: &'a str,
    _handlers: &'a HashMap<
        String,
        Box<dyn Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
    >,
) -> Pin<Box<dyn Future<Output = Result<String, GeminiError>> + Send + 'a>> {
    Box::pin(async move {
        // Directly call the method on the original client:
        client.generate_content_with_google_search(model, prompt).await
    })
}

// Default Processor (function calling) for other steps
fn default_processor<'a>(
    client: &'a GeminiClient,
    model: &'a str,
    prompt: &'a str,
    handlers: &'a HashMap<
        String,
        Box<dyn Fn(&mut serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
    >,
) -> Pin<Box<dyn Future<Output = Result<String, GeminiError>> + Send + 'a>> {
    Box::pin(async move {
        // Use the existing method on the client that performs function-calling:
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![ContentPart::Text(prompt.to_string())],
                role: Role::User,
            }],
            tools: None,
            temperature: Some(0.7),
            top_p: Some(0.95),
            top_k: Some(40),
            max_output_tokens: Some(8192),
            response_mime_type: Some("text/plain".to_string()),
        };

        let response = client
            .generate_content_with_function_calling(model, request, handlers)
            .await?;

        // Extract text from the response
        if let Some(candidates) = response.candidates {
            if let Some(first) = candidates.first() {
                if let Some(PartResponse::Text(text)) = first.content.parts.first() {
                    return Ok(text.clone());
                }
            }
        }

        Err(GeminiError::Api("No text response received".to_string()))
    })
}

/// Initialize the step processors mapping using our enum-based approach.
pub fn init_step_processors() {
    let mut processors: HashMap<u32, StepProcessor> = HashMap::new();

    // Step 2 (Domain Analysis) uses Google Search grounding
    processors.insert(2, StepProcessor::DomainAnalysis);

    // For all other steps, we fall back to the default function-calling processor

    STEP_PROCESSORS.get_or_init(|| processors);
}

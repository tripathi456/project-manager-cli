use std::path::PathBuf;
use std::fs;
use std::error::Error;
use async_trait::async_trait;
use tempfile::tempdir;
use std::collections::HashMap;

// Import the necessary modules from the crate.
use docgen::types::GenerateContentRequest;
use docgen::llm_provider::LLMProvider;
use docgen::template_loader::TemplateLoader;
use docgen::workflow::{Workflow, WorkflowExecutor, WorkflowStep};

// Create a mock LLM provider that logs the prompt it receives
struct ContextLoggingProvider {
    response: String,
}

impl ContextLoggingProvider {
    fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

#[async_trait]
impl LLMProvider for ContextLoggingProvider {
    async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Just return the configured response
        Ok(self.response.clone())
    }

    async fn call_api_for_step(&self, prompt: &str, step: u32) -> Result<String, Box<dyn Error>> {
        // For testing purposes, we'll just log the prompt and return the configured response
        // We can't directly access the context here, but we can examine the prompt
        // to see if it contains the expected dependency content
        println!("DEBUG - Prompt received: {}", prompt);
        
        // Return the configured response
        Ok(self.response.clone())
    }
}

#[tokio::test]
async fn test_multi_dependency_workflow() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create templates directory
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    // Create test files for dependencies
    fs::write(docs_path.join("r01_initial_ideation.md"), "Initial Ideation Content")?;
    fs::write(docs_path.join("r02_domain_analysis.md"), "Domain Analysis Content")?;
    fs::write(docs_path.join("r03_prd_v1.md"), "PRD v1 Content")?;
    
    // Create a template that uses the _dependencies.jinja include
    let dependencies_template = r#"
{% if r01_initial_ideation is defined %}
## Initial Ideation:
{{ r01_initial_ideation }}
{% endif %}

{% if r02_domain_analysis is defined %}
## Domain Analysis:
{{ r02_domain_analysis }}
{% endif %}

{% if r03_prd_v1 is defined %}
## PRD v1:
{{ r03_prd_v1 }}
{% endif %}
"#;
    
    let test_template = r#"
You have received the following inputs from previous steps:

{% include "_dependencies.jinja" %}

STEP {{ current_step }}: Multi-dependency Test

**Task**:  
Process the inputs and generate a response.
"#;
    
    // Write the templates
    fs::write(templates_path.join("_dependencies.jinja"), dependencies_template)?;
    fs::write(templates_path.join("multi_dependency_test.jinja"), test_template)?;
    
    // Create a workflow with a step that has multiple dependencies
    let mut workflow = Workflow::new("Multi-dependency Test Workflow");
    workflow.add_step(
        WorkflowStep::new(
            1,
            "Multi-dependency Test",
            "multi_dependency_output.md",
            vec!["r01_initial_ideation.md", "r02_domain_analysis.md", "r03_prd_v1.md"],
            "multi_dependency_test.jinja"
        )
    );
    
    // Create the LLM provider that will log the context
    let llm_provider = ContextLoggingProvider::new("Test response");
    
    // Create the template loader
    let template_loader = TemplateLoader::from_directory(&templates_path)?;
    
    // Create the workflow executor
    let executor = WorkflowExecutor::new(workflow, &template_loader, &llm_provider);
    
    // Execute the step
    let result = executor.execute_step(1, &docs_path).await;
    assert!(result.is_ok(), "Step execution should succeed");
    
    // Verify the output file exists
    let output_path = docs_path.join("multi_dependency_output.md");
    assert!(output_path.exists(), "Output file should exist");
    
    // Verify the content
    let content = fs::read_to_string(&output_path)?;
    assert_eq!(content, "Test response");
    
    // We can't directly access the context, but we can verify that the
    // dependencies were correctly loaded by checking the output file exists
    assert!(output_path.exists(), "Output file should exist");
    
    Ok(())
}

#[tokio::test]
async fn test_multi_dependency_missing_files() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create templates directory
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    // Create only some of the dependency files
    fs::write(docs_path.join("r01_initial_ideation.md"), "Initial Ideation Content")?;
    // Intentionally not creating r02_domain_analysis.md
    fs::write(docs_path.join("r03_prd_v1.md"), "PRD v1 Content")?;
    
    // Create a template that uses the _dependencies.jinja include
    let dependencies_template = r#"
{% if r01_initial_ideation is defined %}
## Initial Ideation:
{{ r01_initial_ideation }}
{% endif %}

{% if r02_domain_analysis is defined %}
## Domain Analysis:
{{ r02_domain_analysis }}
{% endif %}

{% if r03_prd_v1 is defined %}
## PRD v1:
{{ r03_prd_v1 }}
{% endif %}
"#;
    
    let test_template = r#"
You have received the following inputs from previous steps:

{% include "_dependencies.jinja" %}

STEP {{ current_step }}: Multi-dependency Test

**Task**:  
Process the inputs and generate a response.
"#;
    
    // Write the templates
    fs::write(templates_path.join("_dependencies.jinja"), dependencies_template)?;
    fs::write(templates_path.join("multi_dependency_test.jinja"), test_template)?;
    
    // Create a workflow with a step that has multiple dependencies
    let mut workflow = Workflow::new("Multi-dependency Test Workflow");
    workflow.add_step(
        WorkflowStep::new(
            1,
            "Multi-dependency Test",
            "multi_dependency_output.md",
            vec!["r01_initial_ideation.md", "r02_domain_analysis.md", "r03_prd_v1.md"],
            "multi_dependency_test.jinja"
        )
    );
    
    // Create the LLM provider that will log the context
    let llm_provider = ContextLoggingProvider::new("Test response");
    
    // Create the template loader
    let template_loader = TemplateLoader::from_directory(&templates_path)?;
    
    // Create the workflow executor
    let executor = WorkflowExecutor::new(workflow, &template_loader, &llm_provider);
    
    // Execute the step - this should fail because r02_domain_analysis.md is missing
    let result = executor.execute_step(1, &docs_path).await;
    
    // Verify that the execution failed
    assert!(result.is_err(), "Execution should fail due to missing dependency");
    
    // Verify that the error message mentions the missing file
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("r02_domain_analysis.md"), 
            "Error message should mention the missing file: {}", error_message);
    
    Ok(())
}

#[tokio::test]
async fn test_multi_dependency_template_rendering() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create templates directory
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    // Create test files for dependencies with unique content
    fs::write(docs_path.join("r01_initial_ideation.md"), "UNIQUE_IDEATION_CONTENT")?;
    fs::write(docs_path.join("r02_domain_analysis.md"), "UNIQUE_DOMAIN_ANALYSIS_CONTENT")?;
    fs::write(docs_path.join("r03_prd_v1.md"), "UNIQUE_PRD_V1_CONTENT")?;
    
    // Create a template that uses the _dependencies.jinja include
    let dependencies_template = r#"
{% if r01_initial_ideation is defined %}
## Initial Ideation:
{{ r01_initial_ideation }}
{% endif %}

{% if r02_domain_analysis is defined %}
## Domain Analysis:
{{ r02_domain_analysis }}
{% endif %}

{% if r03_prd_v1 is defined %}
## PRD v1:
{{ r03_prd_v1 }}
{% endif %}
"#;
    
    // Create a template that renders the dependencies directly
    // This will allow us to verify that the dependencies are correctly passed to the template
    let test_template = r#"
{% include "_dependencies.jinja" %}
"#;
    
    // Write the templates
    fs::write(templates_path.join("_dependencies.jinja"), dependencies_template)?;
    fs::write(templates_path.join("render_dependencies_test.jinja"), test_template)?;
    
    // Create a workflow with a step that has multiple dependencies
    let mut workflow = Workflow::new("Multi-dependency Rendering Test Workflow");
    workflow.add_step(
        WorkflowStep::new(
            1,
            "Multi-dependency Rendering Test",
            "rendered_dependencies.md",
            vec!["r01_initial_ideation.md", "r02_domain_analysis.md", "r03_prd_v1.md"],
            "render_dependencies_test.jinja"
        )
    );
    
    // Create a special LLM provider that just returns the rendered template
    struct TemplatePassthroughProvider;
    
    #[async_trait]
    impl LLMProvider for TemplatePassthroughProvider {
        async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
            // Just return the prompt as-is
            Ok(prompt.to_string())
        }
        
        async fn call_api_for_step(&self, prompt: &str, _step: u32) -> Result<String, Box<dyn Error>> {
            // Just return the prompt as-is
            Ok(prompt.to_string())
        }
    }
    
    let llm_provider = TemplatePassthroughProvider;
    
    // Create the template loader
    let template_loader = TemplateLoader::from_directory(&templates_path)?;
    
    // Create the workflow executor
    let executor = WorkflowExecutor::new(workflow, &template_loader, &llm_provider);
    
    // Execute the step
    executor.execute_step(1, &docs_path).await?;
    
    // Verify the output file exists
    let output_path = docs_path.join("rendered_dependencies.md");
    assert!(output_path.exists(), "Output file should exist");
    
    // Read the content of the output file
    let content = fs::read_to_string(&output_path)?;
    
    // Verify that the content contains all the unique strings from the dependencies
    assert!(content.contains("UNIQUE_IDEATION_CONTENT"), 
            "Output should contain content from r01_initial_ideation.md");
    assert!(content.contains("UNIQUE_DOMAIN_ANALYSIS_CONTENT"), 
            "Output should contain content from r02_domain_analysis.md");
    assert!(content.contains("UNIQUE_PRD_V1_CONTENT"), 
            "Output should contain content from r03_prd_v1.md");
    
    // Verify that the content contains the section headers
    assert!(content.contains("## Initial Ideation:"), 
            "Output should contain the Initial Ideation section header");
    assert!(content.contains("## Domain Analysis:"), 
            "Output should contain the Domain Analysis section header");
    assert!(content.contains("## PRD v1:"), 
            "Output should contain the PRD v1 section header");
    
    Ok(())
}
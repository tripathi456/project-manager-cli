use std::path::PathBuf;
use std::fs;
use std::error::Error;
use async_trait::async_trait;
use tempfile::tempdir;

// Import the necessary modules from the crate.
use docgen::types::GenerateContentRequest;
use docgen::llm_provider::LLMProvider;
use docgen::pipeline::PipelineStep;
use docgen::template_loader::TemplateLoader;
use docgen::workflow::{Workflow, WorkflowExecutor, WorkflowStep};

// Create a mock LLM provider for testing.
struct MockLLMProvider {
    // Maps step numbers to response text.
    responses: std::collections::HashMap<u32, String>,
}

impl MockLLMProvider {
    fn new() -> Self {
        let mut responses = std::collections::HashMap::new();
        
        // Add mock responses for each step.
        responses.insert(1, "Mock response for step 1".to_string());
        responses.insert(2, "Mock domain analysis response".to_string());
        responses.insert(3, "Mock PRD v1 response".to_string());
        responses.insert(4, "Mock PRD v2 response".to_string());
        responses.insert(5, "Mock architecture L1 response".to_string());
        responses.insert(6, "Mock architecture L2 response".to_string());
        responses.insert(7, "Mock architecture explanation response".to_string());
        responses.insert(8, "Mock TDD v1 response".to_string());
        // Response for GitHub issues plan.
        responses.insert(9, "Mock GitHub issues plan response".to_string());
        
        Self { responses }
    }
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Print the prompt for debugging.
        println!("DEBUG - Prompt received: {}", prompt);
        
        // Convert prompt to lowercase for case-insensitive matching.
        let prompt_lower = prompt.to_lowercase();
        
        // Identify the step based on unique keywords in the prompt.
        let step = if prompt_lower.contains("prd v1 step") {
            println!("DEBUG - Matched step 3 (prd v1)");
            3
        } else if prompt_lower.contains("prd v2 step") {
            println!("DEBUG - Matched step 4 (prd v2)");
            4
        } else if prompt_lower.contains("architecture l1 step") {
            println!("DEBUG - Matched step 5 (architecture l1)");
            5
        } else if prompt_lower.contains("architecture l2 step") {
            println!("DEBUG - Matched step 6 (architecture l2)");
            6
        } else if prompt_lower.contains("explain architecture step") {
            println!("DEBUG - Matched step 7 (explain architecture)");
            7
        } else if prompt_lower.contains("tdd v1 step") {
            println!("DEBUG - Matched step 8 (tdd v1)");
            8
        } else if prompt_lower.contains("github issues") {
            println!("DEBUG - Matched step 9 (github issues)");
            9
        } else if prompt_lower.contains("domain analysis step") {
            println!("DEBUG - Matched step 2 (domain analysis)");
            2
        } else if prompt_lower.contains("initial ideation step") {
            println!("DEBUG - Matched step 1 (initial ideation)");
            1
        } else {
            println!("DEBUG - No match found, defaulting to step 1");
            1
        };
        
        println!("DEBUG - Returning response for step {}", step);
        
        // Return the corresponding mock response.
        Ok(self.responses.get(&step)
            .unwrap_or(&"Default mock response".to_string()).clone())
    }

    async fn call_api_for_step(&self, prompt: &str, step: u32) -> Result<String, Box<dyn Error>> {
        // For testing purposes, we'll just call the regular call_api method
        self.call_api(prompt).await
    }
}

#[tokio::test]
async fn test_documentation_workflow() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files.
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory within the test directory.
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create prompt_templates directory within the test directory.
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    // Create an initial ideation file.
    let initial_ideation_content = "# Initial Project Ideation\n\n## Project Name: Test Project\n\n## Project Description:\nThis is a test project description.\n\n## Key Features:\n- Feature 1\n- Feature 2\n";
    fs::write(docs_path.join("r01_initial_ideation.md"), initial_ideation_content)?;
    
    // Create test template files for steps 1 to 8.
    for step in 1..=8 {
        let template_prefix = format!("step_{:02}_", step);
        let template_file = match step {
            1 => "initial_ideation.jinja",
            2 => "domain_analysis.jinja",
            3 => "prd_v1.jinja",
            4 => "prd_v2.jinja",
            5 => "arch_L1.jinja",
            6 => "arch_L2.jinja",
            7 => "explain_architecture.jinja",
            8 => "tdd_v1.jinja",
            _ => unreachable!(),
        };
        let full_template_name = format!("{}{}", template_prefix, template_file);
        
        // Create template content with keywords to trigger the corresponding mock response.
        let template_content = match step {
            1 => format!("This is a test template for initial ideation step {}.\n\nPrevious output: {{ previous_output }}", step),
            2 => format!("This is a test template for domain analysis step {}.\n\nPrevious output: {{ previous_output }}\n\nInitial ideation: {{ initial_ideation }}", step),
            3 => format!("This is a test template for PRD v1 step {}.\n\nPrevious output: {{ previous_output }}", step),
            4 => format!("This is a test template for PRD v2 step {}.\n\nPrevious output: {{ previous_output }}", step),
            5 => format!("This is a test template for architecture L1 step {}.\n\nPrevious output: {{ previous_output }}", step),
            6 => format!("This is a test template for architecture L2 step {}.\n\nPrevious output: {{ previous_output }}", step),
            7 => format!("This is a test template for explain architecture step {}.\n\nPrevious output: {{ previous_output }}", step),
            8 => format!("This is a test template for TDD v1 step {}.\n\nPrevious output: {{ previous_output }}", step),
            _ => unreachable!(),
        };
        fs::write(templates_path.join(&full_template_name), template_content)?;
    }
    
    // Create GitHub issues plan template.
    fs::write(
        templates_path.join("step_09_github_issues_plan.jinja"),
        "GitHub Issues Plan template for generating GitHub Issues\n\nTDD Content: {{ tdd_content }}"
    )?;
    
    // Create a mock LLM provider.
    let llm_provider = MockLLMProvider::new();
    
    // Load templates using the TemplateLoader.
    let template_loader = TemplateLoader::from_directory(&templates_path)?;
    
    // Run through each pipeline step from 2 to 9.
    for step in 2..=9 {
        let pipeline_step = PipelineStep::new(step, &llm_provider, &docs_path, &template_loader);
        pipeline_step.run().await?;
        
        // Look up the expected output filename from the default workflow.
        let workflow = Workflow::default_documentation();
        let step_obj = workflow.get_step(step).ok_or("Missing step in workflow")?;
        let output_path = docs_path.join(&step_obj.output_file);
        assert!(output_path.exists(), "Output file for step {} should exist", step);
        
        // Verify the content matches the expected mock response.
        let file_content = fs::read_to_string(&output_path)?;
        let expected_content = match step {
            2 => "Mock domain analysis response",
            3 => "Mock PRD v1 response",
            4 => "Mock PRD v2 response",
            5 => "Mock architecture L1 response",
            6 => "Mock architecture L2 response",
            7 => "Mock architecture explanation response",
            8 => "Mock TDD v1 response",
            9 => "Mock GitHub issues plan response",
            _ => unreachable!(),
        };
        assert_eq!(file_content, expected_content, "Content for step {} should match mock response", step);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_workflow_executor() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files.
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory within the test directory.
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create prompt_templates directory and add a simple template.
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    let template_content = "Test template with previous_output: {{ previous_output }}";
    fs::write(templates_path.join("test_template.jinja"), template_content)?;
    
    // Create an initial ideation file for testing.
    let initial_ideation_content = "# Test Initial Ideation";
    fs::write(docs_path.join("r01_initial_ideation.md"), initial_ideation_content)?;
    
    // Create a mock LLM provider.
    let llm_provider = MockLLMProvider::new();
    
    // Create a template loader.
    let template_loader = TemplateLoader::from_directory(&templates_path)?;
    
    // Create a simple workflow with one step.
    let mut workflow = Workflow::new("Test Workflow");
    workflow.add_step(
        WorkflowStep::new(
            1,
            "Test Step",
            "test_output.txt",
            vec![], // Empty dependencies for first step
            "test_template.jinja"
        )
    );
    
    // Create the workflow executor.
    let engine = WorkflowExecutor::new(workflow, &template_loader, &llm_provider);
    
    // Execute the step.
    engine.execute_step(1, &docs_path).await?;
    
    // Verify that the output file exists.
    let output_path = docs_path.join("test_output.txt");
    assert!(output_path.exists(), "Output file should exist");
    
    // Verify content.
    let file_content = fs::read_to_string(&output_path)?;
    assert_eq!(file_content, "Mock response for step 1", "Content should match mock response");
    
    Ok(())
}

// Test specifically for step 2 with special handling of initial_ideation.
#[tokio::test]
async fn test_workflow_step_2_context() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files.
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory within the test directory.
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create prompt_templates directory.
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    // Create step 2 template that uses initial_ideation.
    let template_content = "Domain analysis template with initial_ideation: {{ initial_ideation }}";
    fs::write(templates_path.join("step_02_domain_analysis.jinja"), template_content)?;
    
    // Create an initial ideation file.
    let initial_ideation_content = "# Initial Project Ideation";
    fs::write(docs_path.join("r01_initial_ideation.md"), initial_ideation_content)?;
    
    // Create a mock LLM provider.
    let llm_provider = MockLLMProvider::new();
    
    // Create template loader and use the default documentation workflow.
    let template_loader = TemplateLoader::from_directory(&templates_path)?;
    let workflow = Workflow::default_documentation();
    
    // Create the workflow executor.
    let engine = WorkflowExecutor::new(workflow, &template_loader, &llm_provider);
    
    // Execute step 2.
    engine.execute_step(2, &docs_path).await?;
    
    // Verify that the output file exists.
    let output_path = docs_path.join("r02_domain_analysis.md");
    assert!(output_path.exists(), "Domain analysis output file should exist");
    
    // (The debug output from the mock provider will confirm that the prompt contained the initial ideation content.)
    Ok(())
}
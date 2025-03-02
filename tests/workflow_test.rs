use std::fs;
use std::error::Error;
use async_trait::async_trait;
use tempfile::tempdir;
use tera::Context;
use std::sync::atomic::{AtomicU32, Ordering};

// Import the necessary modules from the crate
use docgen::llm_provider::LLMProvider;
use docgen::pipeline::{run_single_step, generate_github_issues_plan, step_mapping};
use docgen::template_loader::load_templates;

// Create a mock LLM provider for testing
struct MockLLMProvider {
    // Maps step numbers to response text
    responses: std::collections::HashMap<u32, String>,
    // Counter to track which step we're on
    counter: AtomicU32,
}

impl MockLLMProvider {
    fn new() -> Self {
        let mut responses = std::collections::HashMap::new();
        
        // Add mock responses for each step
        responses.insert(1, "Mock response for step 1".to_string());
        responses.insert(2, "Mock domain analysis response".to_string());
        responses.insert(3, "Mock PRD v1 response".to_string());
        responses.insert(4, "Mock PRD v2 response".to_string());
        responses.insert(5, "Mock architecture L1 response".to_string());
        responses.insert(6, "Mock architecture L2 response".to_string());
        responses.insert(7, "Mock architecture explanation response".to_string());
        responses.insert(8, "Mock TDD v1 response".to_string());
        
        // Response for GitHub issues plan
        responses.insert(9, "Mock GitHub issues plan response".to_string());
        
        Self { 
            responses,
            counter: AtomicU32::new(1),
        }
    }
    
    // Reset the counter
    fn reset_counter(&self) {
        self.counter.store(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // For GitHub issues plan, we need to use step 9
        if prompt.contains("GitHub Issues") {
            return Ok(self.responses.get(&9).unwrap_or(&"Default mock response".to_string()).clone());
        }
        
        // Get the current step from the counter
        let step = self.counter.fetch_add(1, Ordering::SeqCst);
        
        // Return the corresponding mock response
        Ok(self.responses.get(&step).unwrap_or(&"Default mock response".to_string()).clone())
    }
}

#[tokio::test]
async fn test_documentation_workflow() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory within the test directory
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create prompt_templates directory within the test directory
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    // Create an initial ideation file
    let initial_ideation_content = "# Initial Project Ideation\n\n## Project Name: Test Project\n\n## Project Description:\nThis is a test project description.\n\n## Key Features:\n- Feature 1\n- Feature 2\n";
    fs::write(docs_path.join("r01_initial_ideation.txt"), initial_ideation_content)?;
    
    // Create test template files
    for step in 1..=8 {
        let template_name = format!("step_{:02}_", step);
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
        
        let full_template_name = format!("{}{}", template_name, template_file);
        let template_content = match step {
            1 => format!("This is a test template for step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            2 => format!("This is a test template for domain analysis step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            3 => format!("This is a test template for PRD v1 step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            4 => format!("This is a test template for PRD v2 step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            5 => format!("This is a test template for architecture L1 step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            6 => format!("This is a test template for architecture L2 step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            7 => format!("This is a test template for explain architecture step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            8 => format!("This is a test template for TDD v1 step {}.\n\nPrevious output: {{{{ previous_output }}}}", step),
            _ => unreachable!(),
        };
        fs::write(templates_path.join(&full_template_name), template_content)?;
    }
    
    // Create GitHub issues plan template
    fs::write(templates_path.join("github_issues_plan.jinja"), 
             "GitHub Issues Plan template\n\nTDD Content: {{ tdd_content }}")?;
    
    // Create a mock LLM provider
    let llm_provider = MockLLMProvider::new();
    
    // Reset the counter
    llm_provider.reset_counter();
    
    // Load templates
    let tera = load_templates(Some(&templates_path))?;
    
    // Run through each step of the pipeline
    for step in 2..=8 {
        run_single_step(step, &llm_provider, &docs_path, &tera).await?;
        
        // Verify that the output file exists
        let step_mapping = step_mapping();
        let (output_filename, _) = step_mapping.get(&step).unwrap();
        let output_path = docs_path.join(output_filename);
        assert!(output_path.exists(), "Output file for step {} should exist", step);
        
        // Verify the content matches the expected mock response
        let file_content = fs::read_to_string(&output_path)?;
        let expected_content = match step {
            2 => "Mock domain analysis response",
            3 => "Mock PRD v1 response",
            4 => "Mock PRD v2 response",
            5 => "Mock architecture L1 response",
            6 => "Mock architecture L2 response",
            7 => "Mock architecture explanation response",
            8 => "Mock TDD v1 response",
            _ => unreachable!(),
        };
        assert_eq!(file_content, expected_content, "Content for step {} should match mock response", step);
    }
    
    // Set the step for GitHub issues plan
    llm_provider.set_step(9);
    
    // Test GitHub issues plan generation
    generate_github_issues_plan(&llm_provider, &docs_path, &tera).await?;
    
    // Verify GitHub issues plan file exists
    let github_issues_path = docs_path.join("github_issues_plan.md");
    assert!(github_issues_path.exists(), "GitHub issues plan file should exist");
    
    // Verify content
    let github_issues_content = fs::read_to_string(&github_issues_path)?;
    assert_eq!(github_issues_content, "Mock GitHub issues plan response", 
              "GitHub issues plan content should match mock response");
    
    Ok(())
}

#[tokio::test]
async fn test_workflow_step_1() {
    // Create a mock LLM provider
    let mock_provider = MockLLMProvider::new();
    
    // Set the step
    mock_provider.set_step(1);

    // Create a temporary directory for test files
    let temp_dir = tempfile::tempdir().unwrap();
    let docs_path = temp_dir.path().to_path_buf();
    
    // Create templates directory and add test template
    let templates_path = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_path).unwrap();
    fs::write(
        templates_path.join("step_01_initial_ideation.jinja"),
        "Test template for step 1"
    ).unwrap();

    // Initialize Tera template engine
    let tera = load_templates(Some(&templates_path)).unwrap();

    // Run step 1
    let result = run_single_step(1, &mock_provider, &docs_path, &tera).await;
    assert!(result.is_ok());

    // Verify output file was created
    let output_path = docs_path.join("r01_initial_ideation.txt");
    assert!(output_path.exists());
    let content = fs::read_to_string(output_path).unwrap();
    assert_eq!(content, "Mock response for step 1");
}

// Test specifically for step 1 which has special handling
#[tokio::test]
async fn test_step1_template_generation() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory for test files
    let test_dir = tempdir()?;
    let test_path = test_dir.path();
    
    // Create docs directory within the test directory
    let docs_path = test_path.join("docs");
    fs::create_dir_all(&docs_path)?;
    
    // Create prompt_templates directory within the test directory
    let templates_path = test_path.join("prompt_templates");
    fs::create_dir_all(&templates_path)?;
    
    // Create the step 1 template
    let step1_template_content = "# Initial Project Ideation\n\n## Project Name: \n{% if project_name %}{{ project_name }}{% else %}My Project{% endif %}\n\n## Project Description:\nDescribe your project here.\n";
    fs::write(templates_path.join("step_01_initial_ideation.jinja"), step1_template_content)?;
    
    // Load templates
    let tera = load_templates(Some(&templates_path))?;
    
    // Create a context with a project name
    let mut context = Context::new();
    context.insert("project_name", "Test Project");
    
    // Render the template
    let rendered = tera.render("step_01_initial_ideation.jinja", &context)?;
    
    // Write to the output file
    fs::write(docs_path.join("r01_initial_ideation.txt"), &rendered)?;
    
    // Verify the file exists
    let ideation_path = docs_path.join("r01_initial_ideation.txt");
    assert!(ideation_path.exists(), "Initial ideation file should exist");
    
    // Verify the content contains the project name
    let file_content = fs::read_to_string(&ideation_path)?;
    assert!(file_content.contains("Test Project"), "Content should contain the project name");
    
    Ok(())
}
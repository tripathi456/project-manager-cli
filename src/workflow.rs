use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use tracing::{error, info};

use crate::llm_provider::LLMProvider;
use crate::log_prompt;
use crate::template_loader::TemplateRenderer;
use serde_json::{json, Value};

/// Represents a step in a workflow
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub step_number: u32,
    pub description: String,
    pub output_file: String,
    pub previous_file: String,
    pub template_file: String,
}

impl WorkflowStep {
    pub fn new(
        step_number: u32,
        description: &str,
        output_file: &str,
        previous_file: &str,
        template_file: &str,
    ) -> Self {
        Self {
            step_number,
            description: description.to_string(),
            output_file: output_file.to_string(),
            previous_file: previous_file.to_string(),
            template_file: template_file.to_string(),
        }
    }
}

/// Represents a workflow consisting of multiple steps
#[derive(Debug, Clone)]
pub struct Workflow {
    pub name: String,
    pub steps: HashMap<u32, WorkflowStep>,
}

impl Workflow {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: HashMap::new(),
        }
    }
    
    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.insert(step.step_number, step);
    }
    
    pub fn get_step(&self, step_number: u32) -> Option<&WorkflowStep> {
        self.steps.get(&step_number)
    }
    
    /// Create a default documentation workflow
    pub fn default_documentation() -> Self {
        let mut workflow = Self::new("Documentation Workflow");
        
        // Step 1: Initial Ideation
        workflow.add_step(WorkflowStep::new(
            1,
            "Initial Ideation",
            "r01_initial_ideation.txt",
            "",
            "step_01_initial_ideation.jinja",
        ));
        
        // Step 2: Domain Analysis
        workflow.add_step(WorkflowStep::new(
            2,
            "Domain Analysis",
            "r02_domain_analysis.txt",
            "r01_initial_ideation.txt",
            "step_02_domain_analysis.jinja",
        ));
        
        // Step 3: PRD v1
        workflow.add_step(WorkflowStep::new(
            3,
            "PRD v1",
            "r03_prd_v1.txt",
            "r02_domain_analysis.txt",
            "step_03_prd_v1.jinja",
        ));
        
        // Step 4: PRD v2
        workflow.add_step(WorkflowStep::new(
            4,
            "PRD v2",
            "r04_prd_v2.txt",
            "r03_prd_v1.txt",
            "step_04_prd_v2.jinja",
        ));
        
        // Step 5: Architecture L1
        workflow.add_step(WorkflowStep::new(
            5,
            "Architecture L1",
            "r05_arch_L1.txt",
            "r04_prd_v2.txt",
            "step_05_arch_L1.jinja",
        ));
        
        // Step 6: Architecture L2
        workflow.add_step(WorkflowStep::new(
            6,
            "Architecture L2",
            "r06_arch_L2.txt",
            "r05_arch_L1.txt",
            "step_06_arch_L2.jinja",
        ));
        
        // Step 7: Explain Architecture
        workflow.add_step(WorkflowStep::new(
            7,
            "Explain Architecture",
            "r07_explain_architecture.txt",
            "r06_arch_L2.txt",
            "step_07_explain_architecture.jinja",
        ));
        
        // Step 8: TDD v1
        workflow.add_step(WorkflowStep::new(
            8,
            "TDD v1",
            "r08_tdd_v1.txt",
            "r07_explain_architecture.txt",
            "step_08_tdd_v1.jinja",
        ));
        
        // Step 9: GitHub Issues Plan
        workflow.add_step(WorkflowStep::new(
            9,
            "GitHub Issues Plan",
            "github_issues_plan.md",
            "r08_tdd_v1.txt",
            "github_issues_plan.jinja",
        ));
        
        workflow
    }
}

/// A struct for executing a workflow of steps
pub struct WorkflowExecutor<'a> {
    workflow: Workflow,
    /// Now we can hold a trait object safely, because `TemplateRenderer` is object-safe
    template_loader: &'a dyn TemplateRenderer,
    llm_provider: &'a dyn LLMProvider,
}

impl<'a> WorkflowExecutor<'a> {
    pub fn new(
        workflow: Workflow,
        template_loader: &'a dyn TemplateRenderer,
        llm_provider: &'a dyn LLMProvider,
    ) -> Self {
        Self {
            workflow,
            template_loader,
            llm_provider,
        }
    }
    
    pub async fn execute_step<P: AsRef<Path>>(
        &self,
        step_number: u32,
        docs_path: P
    ) -> Result<(), Box<dyn Error>> {
        let docs_path = docs_path.as_ref();
        
        // Get the step
        let step = self.workflow.get_step(step_number)
            .ok_or_else(|| format!("Invalid step number: {}", step_number))?;
        
        info!("Executing step {}: {}", step_number, step.description);
        
        // Prepare a context as a HashMap<String, String>
        let mut ctx_map = HashMap::new();
        ctx_map.insert("current_step".to_string(), step_number.to_string());
        
        // For steps after the first, read the previous output
        if !step.previous_file.is_empty() {
            let previous_file_path = docs_path.join(&step.previous_file);
            if !previous_file_path.exists() {
                // For step 2, if r01 exists in docs directory, use that instead
                if step_number == 2 {
                    let r01_path = docs_path.join("r01_initial_ideation.txt");
                    if r01_path.exists() {
                        let content = fs::read_to_string(&r01_path)?;
                        ctx_map.insert("previous_output".to_string(), content.clone());
                        ctx_map.insert("initial_ideation".to_string(), content);
                    } else {
                        error!("Required input file does not exist: {}", previous_file_path.display());
                        return Err(format!("Required input file does not exist: {}", previous_file_path.display()).into());
                    }
                } else {
                    error!("Required input file does not exist: {}", previous_file_path.display());
                    return Err(format!("Required input file does not exist: {}", previous_file_path.display()).into());
                }
            } else {
                let content = fs::read_to_string(&previous_file_path)?;
                ctx_map.insert("previous_output".to_string(), content.clone());
                
                // For step 2, also insert the initial ideation content
                if step_number == 2 {
                    ctx_map.insert("initial_ideation".to_string(), content);
                }
            }
        }
        
        // Convert the HashMap context to `serde_json::Value`.
        let ctx_value = serde_json::to_value(&ctx_map)?;
        
        // Render the template using the new method
        let prompt = self.template_loader.render_value(&step.template_file, &ctx_value)?;
        
        // Log the prompt
        log_prompt(&format!("Prompt for step {}:", step_number), &prompt);
        
        // Call the LLM provider
        let response = self.llm_provider.call_api(&prompt).await?;
        
        // Write the response to the output file
        let output_path = docs_path.join(&step.output_file);
        fs::write(&output_path, &response)?;
        info!("Step {} completed. Output written to: {}", step_number, output_path.display());
        
        Ok(())
    }
    
    /// Generate a plan for GitHub issues
    pub async fn generate_github_issues_plan<P: AsRef<Path>>(
        &self,
        docs_path: P
    ) -> Result<(), Box<dyn Error>> {
        let docs_path = docs_path.as_ref();
        
        // Read the TDD content
        let tdd_path = docs_path.join("r08_tdd_v1.txt");
        if !tdd_path.exists() {
            error!("TDD file does not exist: {}", tdd_path.display());
            return Err(format!("TDD file does not exist: {}", tdd_path.display()).into());
        }
        
        let tdd_content = fs::read_to_string(&tdd_path)?;
        
        // Convert it to JSON
        let ctx_map = serde_json::json!({
            "tdd_content": tdd_content
        });
        
        // Render the template
        let prompt = self.template_loader.render_value("github_issues_plan.jinja", &ctx_map)?;
        
        // Log the prompt
        log_prompt("Prompt for GitHub issues plan:", &prompt);
        
        // Call the LLM provider
        let response = self.llm_provider.call_api(&prompt).await?;
        
        // Write the response to the output file
        let output_path = docs_path.join("github_issues_plan.md");
        fs::write(&output_path, &response)?;
        info!("GitHub issues plan generated. Output written to: {}", output_path.display());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::{always, eq};

    use mockall::mock;
    use tempfile::TempDir;
    use std::error::Error;

    // Mock the object-safe trait
    mock! {
        pub TemplateRendererMock {}
        impl TemplateRenderer for TemplateRendererMock {
            fn render_value(&self, template_name: &str, context: &serde_json::Value)
                -> Result<String, Box<dyn Error>>;
        }
    }

    mock! {
        pub LLMProviderMock {}
        #[async_trait::async_trait]
        impl LLMProvider for LLMProviderMock {
            async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>>;
        }
    }

    #[tokio::test]
    async fn test_execute_step() {
        // Create a temp directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a mock LLM provider
        let mut llm_provider = MockLLMProviderMock::new();
        llm_provider
            .expect_call_api()
            .with(always())
            .returning(|_| Ok("Test response".to_string()));
            
        // Create a mock template renderer
        let mut template_renderer = MockTemplateRendererMock::new();
        template_renderer
            .expect_render_value()
            .with(eq("step_01_initial_ideation.jinja"), always())
            .returning(|_, _| Ok("Test template".to_string()));
            
        // Create a workflow executor
        let workflow = Workflow::default_documentation();
        let executor = WorkflowExecutor::new(workflow, &template_renderer, &llm_provider);
        
        // Execute step 1
        let result = executor.execute_step(1, temp_path).await;
        assert!(result.is_ok());
        
        // Verify the output file exists
        let output_path = temp_path.join("r01_initial_ideation.txt");
        assert!(output_path.exists());
        
        // Verify the output content
        let content = fs::read_to_string(output_path).unwrap();
        assert_eq!(content, "Test response");
    }
}

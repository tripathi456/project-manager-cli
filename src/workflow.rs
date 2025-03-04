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
    pub dependencies: Vec<String>,
    pub template_file: String,
}

impl WorkflowStep {
    pub fn new(
        step_number: u32,
        description: &str,
        output_file: &str,
        dependencies: Vec<&str>,
        template_file: &str,
    ) -> Self {
        Self {
            step_number,
            description: description.to_string(),
            output_file: output_file.to_string(),
            dependencies: dependencies.iter().map(|&s| s.to_string()).collect(),
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
            "r01_initial_ideation.md",
            vec![],
            "step_01_initial_ideation.jinja",
        ));
        
        // Step 2: Domain Analysis
        workflow.add_step(WorkflowStep::new(
            2,
            "Domain Analysis",
            "r02_domain_analysis.md",
            vec!["r01_initial_ideation.md"],
            "step_02_domain_analysis.jinja",
        ));
        
        // Step 3: PRD v1
        workflow.add_step(WorkflowStep::new(
            3,
            "PRD v1",
            "r03_prd_v1.md",
            vec!["r02_domain_analysis.md"],
            "step_03_prd_v1.jinja",
        ));
        
        // Step 4: PRD v2
        workflow.add_step(WorkflowStep::new(
            4,
            "PRD v2",
            "r04_prd_v2.md",
            vec!["r03_prd_v1.md"],
            "step_04_prd_v2.jinja",
        ));
        
        // Step 5: Architecture L1
        workflow.add_step(WorkflowStep::new(
            5,
            "Architecture L1",
            "r05_arch_L1.md",
            vec!["r04_prd_v2.md"],
            "step_05_arch_L1.jinja",
        ));
        
        // Step 6: Architecture L2
        workflow.add_step(WorkflowStep::new(
            6,
            "Architecture L2",
            "r06_arch_L2.md",
            vec!["r05_arch_L1.md"],
            "step_06_arch_L2.jinja",
        ));
        
        // Step 7: Explain Architecture
        workflow.add_step(WorkflowStep::new(
            7,
            "Explain Architecture",
            "r07_explain_architecture.md",
            vec!["r06_arch_L2.md"],
            "step_07_explain_architecture.jinja",
        ));
        
        // Step 8: TDD v1
        workflow.add_step(WorkflowStep::new(
            8,
            "TDD v1",
            "r08_tdd_v1.md",
            vec!["r07_explain_architecture.md"],
            "step_08_tdd_v1.jinja",
        ));
        
        // Step 9: GitHub Issues Plan
        workflow.add_step(WorkflowStep::new(
            9,
            "GitHub Issues Plan",
            "github_issues_plan.md",
            vec!["r08_tdd_v1.md"],
            "step_09_github_issues_plan.jinja",
        ));
        
        workflow
    }
}

/// A struct for executing a workflow of steps
pub struct WorkflowExecutor<'a> {
    workflow: Workflow,
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
    
    /// Execute a specific step in the workflow
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
        
        // Prepare context for template rendering
        let ctx_value = self.prepare_context(step, docs_path)?;
        
        // Render the template
        let prompt = self.render_template(&step.template_file, &ctx_value)?;
        
        // Log the prompt
        log_prompt(&format!("Prompt for step {}:", step_number), &prompt);
        
        // Call the LLM provider and write the response
        self.process_llm_response(step, &prompt, docs_path).await?;
        
        Ok(())
    }
    
    /// Prepare the context for template rendering
    fn prepare_context(&self, step: &WorkflowStep, docs_path: &Path) -> Result<Value, Box<dyn Error>> {
        let mut ctx_map = HashMap::new();
        ctx_map.insert("current_step".to_string(), step.step_number.to_string());
        
        // For steps with dependencies, read the previous outputs
        if !step.dependencies.is_empty() {
            self.load_dependencies(step, docs_path, &mut ctx_map)?;
        }
        
        // Convert the HashMap context to `serde_json::Value`
        Ok(serde_json::to_value(&ctx_map)?)
    }
    
    /// Load dependencies for a step
    fn load_dependencies(
        &self, 
        step: &WorkflowStep, 
        docs_path: &Path, 
        ctx_map: &mut HashMap<String, String>
    ) -> Result<(), Box<dyn Error>> {
        // Add all dependencies to the context
        self.load_all_dependencies(step, docs_path, ctx_map)?;
        
        Ok(())
    }
    
    /// Load all dependencies for a step
    fn load_all_dependencies(
        &self,
        step: &WorkflowStep,
        docs_path: &Path,
        ctx_map: &mut HashMap<String, String>
    ) -> Result<(), Box<dyn Error>> {
        for dependency in &step.dependencies {
            let dependency_path = docs_path.join(dependency);
            if dependency_path.exists() {
                let content = fs::read_to_string(&dependency_path)?;
                
                // Use the filename without extension as the key
                let key = Path::new(dependency)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                ctx_map.insert(key.clone(), content.clone());
                
                // If this is the first dependency, also set it as previous_output
                if dependency == step.dependencies.first().unwrap() {
                    ctx_map.insert("previous_output".to_string(), content.clone());
                    
                    // For step 2, also insert the initial ideation content
                    if step.step_number == 2 && key == "r01_initial_ideation" {
                        ctx_map.insert("initial_ideation".to_string(), content);
                    }
                }
            } else {
                // Return an error if a dependency is missing
                error!("Required dependency file does not exist: {}", dependency_path.display());
                return Err(format!("Required dependency file does not exist: {}", dependency_path.display()).into());
            }
        }
        
        Ok(())
    }
    
    /// Render a template with the given context
    fn render_template(&self, template_file: &str, context: &Value) -> Result<String, Box<dyn Error>> {
        self.template_loader.render_value(template_file, context)
    }
    
    /// Process the LLM response and write it to the output file
    async fn process_llm_response<P: AsRef<Path>>(
        &self,
        step: &WorkflowStep,
        prompt: &str,
        docs_path: P
    ) -> Result<(), Box<dyn Error>> {
        let docs_path = docs_path.as_ref();
        
        // Call the LLM provider
        let response = self.llm_provider.call_api_for_step(prompt, step.step_number).await?;
        
        // Write the response to the output file
        let output_path = docs_path.join(&step.output_file);
        fs::write(&output_path, &response)?;
        info!("Step {} completed. Output written to: {}", step.step_number, output_path.display());
        
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
            async fn call_api(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
                Ok("Test response".to_string())
            }
            
            async fn call_api_for_step(&self, prompt: &str, step_number: u32) -> Result<String, Box<dyn Error>> {
                Ok("Test response".to_string())
            }
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
            .expect_call_api_for_step()
            .with(always(), always())
            .returning(|_, _| Ok("Test response".to_string()));
            
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
        let output_path = temp_path.join("r01_initial_ideation.md");
        assert!(output_path.exists());
        
        // Verify the output content
        let content = fs::read_to_string(output_path).unwrap();
        assert_eq!(content, "Test response");
    }
}
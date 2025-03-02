use std::error::Error;
use std::path::{Path, PathBuf};

use crate::llm_provider::LLMProvider;
use crate::template_loader::TemplateRenderer;
use crate::workflow::{Workflow, WorkflowExecutor};

/// A struct for running a single step of the pipeline
pub struct PipelineStep<'a> {
    step: u32,
    llm_provider: &'a dyn LLMProvider,
    docs_path: PathBuf,
    template_loader: &'a dyn TemplateRenderer,
}

impl<'a> PipelineStep<'a> {
    pub fn new<P: AsRef<Path>>(
        step: u32,
        llm_provider: &'a dyn LLMProvider,
        docs_path: P,
        template_loader: &'a dyn TemplateRenderer,
    ) -> Self {
        PipelineStep {
            step,
            llm_provider,
            docs_path: docs_path.as_ref().to_path_buf(),
            template_loader,
        }
    }

    /// Runs a single step of the pipeline using the new workflow-based approach
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let workflow = Workflow::default_documentation();
        let executor = WorkflowExecutor::new(workflow, self.template_loader, self.llm_provider);
        executor.execute_step(self.step, &self.docs_path).await
    }
}

/// Generates a plan for GitHub issues using the new workflow-based approach
pub async fn generate_github_issues_plan<P: AsRef<Path>>(
    llm_provider: &dyn LLMProvider,
    docs_path: P,
    template_loader: &dyn TemplateRenderer,
) -> Result<(), Box<dyn Error>> {
    let workflow = Workflow::default_documentation();
    let executor = WorkflowExecutor::new(workflow, template_loader, llm_provider);
    executor.generate_github_issues_plan(docs_path).await
}

// src/main.rs

use clap::{Parser, Subcommand};
use docgen::log_prompt;
use docgen::llm_provider::get_llm_provider;
use docgen::pipeline::{PipelineStep, generate_github_issues_plan};
use docgen::template_loader::{TemplateLoader, TemplateRenderer};
use docgen::workflow::Workflow;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// Embedded prompt_templates folder.
static EMBEDDED_TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/prompt_templates");

/// CLI arguments and subcommands
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Run the documentation pipeline
    Run {
        /// Starting step (1 = initial ideation, 2 = domain analysis, 3 = PRD v1, etc.)
        #[arg(long, default_value = "2")]
        step: u32,
        /// Path to the documentation folder.
        #[arg(long)]
        docs_path: PathBuf,
    },
    /// Generates GitHub issues plan.
    GenerateGithubIssues {
        /// Path to the documentation folder.
        #[arg(long)]
        docs_path: PathBuf,
    },
    /// Views a prompt for a given step.
    ViewPrompt {
        /// Step number.
        #[arg(long)]
        step: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Get the LLM provider (choose "gemini" or any other known name)
    let llm_provider = get_llm_provider("gemini")?;
    
    // Collect embedded templates
    let templates: Vec<(String, String)> = EMBEDDED_TEMPLATES
        .files()
        .filter_map(|file| {
            let path = file.path().to_string_lossy().to_string();
            file.contents_utf8().map(|contents| (path, contents.to_string()))
        })
        .collect();

    // Create template loader from embedded templates
    let template_loader = TemplateLoader::from_embedded(&templates)?;
    
    match &cli.command {
        Commands::Run { step, docs_path } => {
            let pipeline_step = PipelineStep::new(*step, &*llm_provider, docs_path, &template_loader);
            pipeline_step.run().await?;
        }
        Commands::GenerateGithubIssues { docs_path } => {
            generate_github_issues_plan(&*llm_provider, docs_path, &template_loader).await?;
        }
        Commands::ViewPrompt { step } => {
            let workflow = Workflow::default_documentation();
            let step_obj = workflow.get_step(*step)
                .ok_or_else(|| format!("Invalid step number: {}", step))?;
            
            // Render the template with an empty context
            let context = HashMap::<String,String>::new();

            // Convert the `HashMap` or any struct into a `serde_json::Value`.
            let context_value = serde_json::to_value(&context)?;

            let rendered = template_loader.render_value(&step_obj.template_file, &context_value)?;
            
            // Print the rendered template
            println!("{}", rendered);
        }
    }
    
    Ok(())
}

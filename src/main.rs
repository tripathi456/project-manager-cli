// src/main.rs

use clap::{Parser, Subcommand};
use include_dir::{include_dir, Dir};
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

// Import our modules instead of defining them inline
mod types;
mod gemini_client;
mod llm_provider;
mod pipeline;
mod template_loader;

use types::{Content, ContentPart, GenerateContentRequest, Role};
use llm_provider::{LLMProvider, get_llm_provider};
use pipeline::{run_single_step, generate_github_issues_plan};
use template_loader::load_templates;

/// Embedded prompt_templates folder.
/// Ensure you have a folder named "prompt_templates" in your project root.
static EMBEDDED_TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/prompt_templates");

/// Helper function for logging multiline prompts.
fn log_prompt(title: &str, prompt_text: &str) {
    info!("{}", title);
    for (i, line) in prompt_text.lines().enumerate() {
        info!("  {}: {}", i + 1, line);
    }
}

/// CLI definition using Clap.
#[derive(Parser)]
#[command(
    name = "docgen",
    about = "A CLI tool for generating project documentation using multiple LLM providers."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Supported subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Runs the documentation generation pipeline.
    Run {
        /// Starting step (1 = initial ideation, 2 = domain analysis, 3 = PRD v1, etc.)
        #[arg(long, default_value = "2")]
        step: u32,
        /// Which LLM provider to use? (gemini, openai, etc.)
        #[arg(long, default_value = "gemini")]
        provider: String,
        /// Optional custom prompt templates folder.
        #[arg(long)]
        prompt_folder: Option<PathBuf>,
    },
}

/// Initializes the tracing subscriber for structured logging.
///
/// Logs are written to a daily-rotated log file (`app.log`) and to stdout.
fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    // Create a rolling file appender
    let file_appender = RollingFileAppender::new(Rotation::DAILY, ".", "app.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    
    // Set up tracing with both console and file output
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false)
        )
        .with(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();
    
    guard
}

/// Main entry point.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing for structured logging.
    let _guard = init_tracing();
    info!("Starting documentation generator CLI");

    // Parse command-line arguments.
    let cli = Cli::parse();

    // Handle each subcommand.
    match cli.command {
        Commands::Run {
            step,
            provider,
            prompt_folder,
        } => {
            info!("Running documentation pipeline from step {}", step);
            
            // Ensure the docs folder exists
            let docs_folder = Path::new("docs");
            if !docs_folder.exists() {
                fs::create_dir(docs_folder)?;
                info!("Created docs folder: {}", docs_folder.display());
            }
            
            // Handle step 1 (initial ideation)
            let initial_ideation_path = docs_folder.join("r01_initial_ideation.txt");
            if step == 1 {
                info!("Creating/updating initial ideation file based on template");
                
                // Load the templates
                let tera = match prompt_folder {
                    Some(ref folder) => load_templates(Some(folder))?,
                    None => {
                        let mut tera = tera::Tera::default();
                        for file in EMBEDDED_TEMPLATES.files() {
                            let path = file.path().to_string_lossy();
                            if let Some(contents) = file.contents_utf8() {
                                tera.add_raw_template(&path, contents)?;
                            }
                        }
                        tera
                    }
                };
                
                // Render the template with an empty context
                let context = tera::Context::new();
                let rendered = tera.render("step_01_initial_ideation.jinja", &context)?;
                
                // Write to the output file
                fs::write(&initial_ideation_path, rendered)?;
                info!("Created/updated initial ideation file: {}", initial_ideation_path.display());
                info!("Step 1 done. Please edit {} and re-run starting from step 2", 
                    initial_ideation_path.display());
                return Ok(());
            }
            
            // Validate the initial ideation file is not empty if proceeding beyond step 1
            if step > 1 {
                if !initial_ideation_path.exists() {
                    error!("Initial ideation file does not exist. Please run with --step 1 first.");
                    return Err("Initial ideation file does not exist".into());
                }
                
                let content = fs::read_to_string(&initial_ideation_path)?;
                if content.trim().is_empty() {
                    error!("Initial ideation file is empty. Please write your project ideas in {} before proceeding", 
                        initial_ideation_path.display());
                    return Err("Initial ideation file cannot be empty".into());
                }
            }
            
            // Get the LLM provider
            let llm_provider = get_llm_provider(&provider)?;
            
            // Load the templates 
            let tera = if let Some(folder) = prompt_folder {
                load_templates(Some(&folder))?
            } else {
                let mut tera = tera::Tera::default();
                for file in EMBEDDED_TEMPLATES.files() {
                    let path = file.path().to_string_lossy();
                    if let Some(contents) = file.contents_utf8() {
                        tera.add_raw_template(&path, contents)?;
                    }
                }
                tera
            };
            
            // Run the pipeline from the specified step
            for current_step in step..=8 {
                info!("Running step {}", current_step);
                run_single_step(current_step, llm_provider.as_ref(), docs_folder, &tera).await?;
                info!("Completed step {}", current_step);
            }
            
            // Generate GitHub issues plan
            info!("Generating GitHub issues plan");
            generate_github_issues_plan(llm_provider.as_ref(), docs_folder, &tera).await?;
            info!("Completed GitHub issues plan generation");
            
            info!("Documentation pipeline completed successfully!");
        }
    }

    Ok(())
}

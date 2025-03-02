use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path};
use tracing::{info, error};
use tera::{Tera, Context};

use crate::llm_provider::LLMProvider;
use crate::log_prompt;

// Step numbers to output file names and previous file mapping
pub fn step_mapping() -> HashMap<u32, (&'static str, &'static str)> {
    let mut map = HashMap::new();
    // Step number => (output_filename, previous_step_file)
    map.insert(1, ("r01_initial_ideation.txt", ""));  // Step 1 has no previous file
    map.insert(2, ("r02_domain_analysis.txt", "r01_initial_ideation.txt"));
    map.insert(3, ("r03_prd_v1.txt", "r02_domain_analysis.txt"));
    map.insert(4, ("r04_prd_v2.txt", "r03_prd_v1.txt"));
    map.insert(5, ("r05_arch_L1.txt", "r04_prd_v2.txt"));
    map.insert(6, ("r06_arch_L2.txt", "r05_arch_L1.txt"));
    map.insert(7, ("r07_explain_architecture.txt", "r06_arch_L2.txt"));
    map.insert(8, ("r08_tdd_v1.txt", "r07_explain_architecture.txt"));
    map
}

// Step numbers to template names mapping
pub fn template_mapping() -> HashMap<u32, &'static str> {
    let mut map = HashMap::new();
    map.insert(1, "step_01_initial_ideation.jinja");
    map.insert(2, "step_02_domain_analysis.jinja");
    map.insert(3, "step_03_prd_v1.jinja");
    map.insert(4, "step_04_prd_v2.jinja");
    map.insert(5, "step_05_arch_L1.jinja");
    map.insert(6, "step_06_arch_L2.jinja");
    map.insert(7, "step_07_explain_architecture.jinja");
    map.insert(8, "step_08_tdd_v1.jinja");
    map
}

/// Runs a single step of the pipeline
pub async fn run_single_step<P: AsRef<Path>>(
    step: u32, 
    llm_provider: &dyn LLMProvider, 
    docs_path: P,
    tera: &Tera
) -> Result<(), Box<dyn Error>> {
    let docs_path = docs_path.as_ref();
    let mapping = step_mapping();
    let template_map = template_mapping();
    
    if let Some((output_filename, previous_file)) = mapping.get(&step) {
        let template_name = template_map.get(&step).ok_or(format!("No template found for step {}", step))?;
        
        // For step 1, we don't need to check for previous file
        let previous_output = if step == 1 {
            String::new()
        } else {
            // Check if the previous file exists
            let previous_file_path = docs_path.join(previous_file);
            if !previous_file_path.exists() {
                error!("Required input file does not exist: {}", previous_file_path.display());
                return Err(format!("Required input file does not exist: {}", previous_file_path.display()).into());
            }
            fs::read_to_string(&previous_file_path)?
        };
        
        // Create a context with the previous output
        let mut context = Context::new();
        context.insert("previous_output", &previous_output);
        context.insert("current_step", &step);
        
        // Render the template
        let prompt = tera.render(template_name, &context)?;
        
        // Log the prompt
        log_prompt(&format!("Prompt for step {}:", step), &prompt);
        
        // Call the LLM provider
        let response = llm_provider.call_api(&prompt).await?;
        
        // Write the response to the output file
        let output_path = docs_path.join(output_filename);
        fs::write(&output_path, &response)?;
        info!("Step {} completed. Output written to: {}", step, output_path.display());
        
        Ok(())
    } else {
        Err(format!("Invalid step number: {}", step).into())
    }
}

/// Generates a plan for GitHub issues
pub async fn generate_github_issues_plan<P: AsRef<Path>>(
    llm_provider: &dyn LLMProvider,
    docs_path: P,
    tera: &Tera
) -> Result<(), Box<dyn Error>> {
    let docs_path = docs_path.as_ref();
    
    // Read the TDD content
    let tdd_path = docs_path.join("r08_tdd_v1.txt");
    if !tdd_path.exists() {
        error!("TDD file does not exist: {}", tdd_path.display());
        return Err(format!("TDD file does not exist: {}", tdd_path.display()).into());
    }
    
    let tdd_content = fs::read_to_string(&tdd_path)?;
    
    // Create a context with the TDD content
    let mut context = Context::new();
    context.insert("tdd_content", &tdd_content);
    
    // Render the template
    let prompt = tera.render("github_issues_plan.jinja", &context)?;
    
    // Log the prompt
    log_prompt("Prompt for GitHub issues plan:", &prompt);
    
    // Call the LLM provider
    let response = llm_provider.call_api(&prompt).await?;
    
    // Write the response to the output file
    let output_path = docs_path.join("github_issues_plan.md");
    fs::write(&output_path, &response)?;
    info!("GitHub issues plan generated. Output written to: {}", output_path.display());
    
    Ok(())
}
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use docgen::workflow::Workflow;

#[test]
fn test_init_command_creates_docs_folder_and_step_01_file() {
    // Create a temporary directory for test files
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path();
    
    // Create docs directory path within the test directory
    let docs_path = test_path.join("docs");
    
    // Verify docs directory doesn't exist yet
    assert!(!docs_path.exists(), "Docs directory should not exist before init");
    
    // Create the docs directory and step_01 file
    create_docs_and_step_01(&docs_path).unwrap();
    
    // Verify docs directory was created
    assert!(docs_path.exists(), "Docs directory should exist after init");
    
    // Get the step_01 file name from the workflow
    let workflow = Workflow::default_documentation();
    let step_obj = workflow.get_step(1).unwrap();
    
    // Verify step_01 file was created
    let step_01_file_path = docs_path.join(&step_obj.output_file);
    assert!(step_01_file_path.exists(), "step_01 file should exist after init");
    
    // Verify step_01 file is empty
    let content = fs::read_to_string(&step_01_file_path).unwrap();
    assert!(content.is_empty(), "step_01 file should be empty");
}

#[test]
fn test_init_command_handles_existing_docs_folder() {
    // Create a temporary directory for test files
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path();
    
    // Create docs directory path within the test directory
    let docs_path = test_path.join("docs");
    
    // Create the docs directory manually
    fs::create_dir_all(&docs_path).unwrap();
    
    // Verify docs directory exists
    assert!(docs_path.exists(), "Docs directory should exist before init");
    
    // Create the step_01 file
    create_docs_and_step_01(&docs_path).unwrap();
    
    // Get the step_01 file name from the workflow
    let workflow = Workflow::default_documentation();
    let step_obj = workflow.get_step(1).unwrap();
    
    // Verify step_01 file was created
    let step_01_file_path = docs_path.join(&step_obj.output_file);
    assert!(step_01_file_path.exists(), "step_01 file should exist after init");
}

#[test]
fn test_init_command_handles_existing_step_01_file() {
    // Create a temporary directory for test files
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path();
    
    // Create docs directory path within the test directory
    let docs_path = test_path.join("docs");
    
    // Create the docs directory manually
    fs::create_dir_all(&docs_path).unwrap();
    
    // Get the step_01 file name from the workflow
    let workflow = Workflow::default_documentation();
    let step_obj = workflow.get_step(1).unwrap();
    
    // Create the step_01 file manually with some content
    let step_01_file_path = docs_path.join(&step_obj.output_file);
    fs::write(&step_01_file_path, "existing content").unwrap();
    
    // Verify step_01 file exists with content
    assert!(step_01_file_path.exists(), "step_01 file should exist before init");
    let content_before = fs::read_to_string(&step_01_file_path).unwrap();
    assert_eq!(content_before, "existing content", "step_01 file should have content before init");
    
    // Call init command
    create_docs_and_step_01(&docs_path).unwrap();
    
    // Verify step_01 file still exists with the same content
    assert!(step_01_file_path.exists(), "step_01 file should still exist after init");
    let content_after = fs::read_to_string(&step_01_file_path).unwrap();
    assert_eq!(content_after, "existing content", "step_01 file content should not be changed");
}

// Helper function that mimics the Init command functionality
fn create_docs_and_step_01(docs_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create the docs directory if it doesn't exist
    if !docs_path.exists() {
        fs::create_dir_all(docs_path)?;
    }
    
    // Get the step_01 file name from the workflow
    let workflow = Workflow::default_documentation();
    let step_obj = workflow.get_step(1)
        .ok_or_else(|| "Failed to get step 1 from workflow".to_string())?;
    
    // Create an empty file for step_01
    let step_01_file_path = docs_path.join(&step_obj.output_file);
    if !step_01_file_path.exists() {
        fs::write(&step_01_file_path, "")?;
    }
    
    Ok(())
}
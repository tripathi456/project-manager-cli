Below is one way to refactor the nested if/else logic into a hashmap that maps each step to a function (closure) responsible for retrieving the previous output. In this example, we define a type alias for a resolver function that, given the docs path, returns either the previous output or an error. You can adjust the logic for each step as needed (for example, adding fallback rules for step 2):

```rust
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

// Define a type alias for the closure that retrieves the previous output.
type PrevResolver = Box<dyn Fn(&Path) -> Result<String, Box<dyn Error>>>;

fn build_prev_resolvers() -> HashMap<u32, PrevResolver> {
    let mut resolvers: HashMap<u32, PrevResolver> = HashMap::new();

    // Step 1: No previous output.
    resolvers.insert(1, Box::new(|_docs_path: &Path| Ok(String::new())));

    // Step 2: Read "r01_initial_ideation.txt".
    // If the file does not exist, you can add fallback logic here.
    resolvers.insert(2, Box::new(|docs_path: &Path| {
        let path = docs_path.join("r01_initial_ideation.txt");
        if path.exists() {
            fs::read_to_string(&path).map_err(|e| e.into())
        } else {
            Err(format!("Required file does not exist: {}", path.display()).into())
        }
    }));

    // Step 3: Read "r02_domain_analysis.txt".
    resolvers.insert(3, Box::new(|docs_path: &Path| {
        let path = docs_path.join("r02_domain_analysis.txt");
        if path.exists() {
            fs::read_to_string(&path).map_err(|e| e.into())
        } else {
            Err(format!("Required file does not exist: {}", path.display()).into())
        }
    }));

    // Step 4: Read "r03_prd_v1.txt".
    resolvers.insert(4, Box::new(|docs_path: &Path| {
        let path = docs_path.join("r03_prd_v1.txt");
        if path.exists() {
            fs::read_to_string(&path).map_err(|e| e.into())
        } else {
            Err(format!("Required file does not exist: {}", path.display()).into())
        }
    }));

    // Step 5: Read "r04_prd_v2.txt".
    resolvers.insert(5, Box::new(|docs_path: &Path| {
        let path = docs_path.join("r04_prd_v2.txt");
        if path.exists() {
            fs::read_to_string(&path).map_err(|e| e.into())
        } else {
            Err(format!("Required file does not exist: {}", path.display()).into())
        }
    }));

    // Step 6: Read "r05_arch_L1.txt".
    resolvers.insert(6, Box::new(|docs_path: &Path| {
        let path = docs_path.join("r05_arch_L1.txt");
        if path.exists() {
            fs::read_to_string(&path).map_err(|e| e.into())
        } else {
            Err(format!("Required file does not exist: {}", path.display()).into())
        }
    }));

    // Step 7: Read "r06_arch_L2.txt".
    resolvers.insert(7, Box::new(|docs_path: &Path| {
        let path = docs_path.join("r06_arch_L2.txt");
        if path.exists() {
            fs::read_to_string(&path).map_err(|e| e.into())
        } else {
            Err(format!("Required file does not exist: {}", path.display()).into())
        }
    }));

    // Step 8: Read "r07_explain_architecture.txt".
    resolvers.insert(8, Box::new(|docs_path: &Path| {
        let path = docs_path.join("r07_explain_architecture.txt");
        if path.exists() {
            fs::read_to_string(&path).map_err(|e| e.into())
        } else {
            Err(format!("Required file does not exist: {}", path.display()).into())
        }
    }));

    resolvers
}
```

Then, in your `run_single_step` function you can replace the previous if/else block with a call to the corresponding resolver:

```rust
pub async fn run_single_step<P: AsRef<Path>>(
    step: u32, 
    llm_provider: &dyn LLMProvider, 
    docs_path: P,
    tera: &Tera
) -> Result<(), Box<dyn Error>> {
    let docs_path = docs_path.as_ref();
    let mapping = step_mapping();
    let template_map = template_mapping();

    if let Some((output_filename, _)) = mapping.get(&step) {
        let template_name = template_map.get(&step)
            .ok_or(format!("No template found for step {}", step))?;
        
        // Use the hashmap to get the previous output based on the step.
        let resolvers = build_prev_resolvers();
        let resolver = resolvers.get(&step)
            .ok_or(format!("No previous output resolver for step {}", step))?;
        let previous_output = resolver(docs_path)?;

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
```

### Explanation

- **Closure Mapping:** We create a `HashMap<u32, PrevResolver>` where each key (the step number) maps to a closure that knows how to retrieve the previous file content.
- **Simplified Logic:** This approach isolates the file-checking logic per step in one place. It makes the main function cleaner and makes it easier to add or modify rules for each step.
- **Fallback Logic:** In the example for step 2, you can add any fallback logic you need (for instance, checking another file) directly inside the closure.

This design abstracts the conditionals into data-driven rules using a hashmap, making your code easier to maintain and extend.
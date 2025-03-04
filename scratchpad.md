# Multiple Dependencies for Workflow Steps

## Changes Made

1. Updated the `WorkflowStep` struct to support multiple dependencies:
   - Changed `previous_file: String` to `dependencies: Vec<String>`
   - Updated the constructor to accept a vector of dependencies

2. Updated the `execute_step` method in `WorkflowExecutor` to handle multiple dependencies:
   - For backward compatibility, the first dependency is still used as `previous_output`
   - Each dependency is now added to the context with its filename (without extension) as the key
   - This allows templates to access any dependency by name

3. Updated the default workflow to use the new constructor with multiple dependencies

4. Added a new test case `test_workflow_multiple_dependencies` to verify the functionality

5. Created a new template example `step_multi_dependency_example.jinja` that demonstrates how to use multiple dependencies

## How to Use Multiple Dependencies

### Creating a Workflow Step with Multiple Dependencies

```rust
// Example of creating a step with multiple dependencies
let step = WorkflowStep::new(
    10,
    "Multi-dependency Step",
    "output_file.md",
    vec!["dependency1.md", "dependency2.md", "dependency3.md"],
    "template_file.jinja"
);
```

### Accessing Dependencies in Templates

In your Jinja templates, you can now access each dependency by its filename (without extension):

```jinja
{% if dependency1 is defined %}
## Content from Dependency 1:
{{ dependency1 }}
{% endif %}

{% if dependency2 is defined %}
## Content from Dependency 2:
{{ dependency2 }}
{% endif %}

{% if dependency3 is defined %}
## Content from Dependency 3:
{{ dependency3 }}
{% endif %}
```

For backward compatibility, the first dependency is still available as `previous_output`:

```jinja
## Content from Previous Output (first dependency):
{{ previous_output }}
```

### Example Template

A complete example template is provided in `prompt_templates/step_multi_dependency_example.jinja`. This template shows how to check for the existence of each dependency and include it in the prompt if it exists.

## Edge Cases Handled

1. **Backward Compatibility**: The first dependency is still available as `previous_output` to maintain compatibility with existing templates.

2. **Missing Dependencies**: The template uses `{% if dependency is defined %}` to check if a dependency exists before trying to use it.

3. **Special Case for Step 2**: The special handling for step 2 (using `initial_ideation`) is preserved.

4. **Empty Dependencies**: Steps with no dependencies (like step 1) are handled correctly.

5. **File Not Found**: If a dependency file doesn't exist, an appropriate error is returned.

## Current Task
Implementing GitHub issues plan generation as step 9 in the pipeline.

### Progress
[X] Added step 9 to the default_documentation workflow in workflow.rs
[X] Updated generate_github_issues_plan function in pipeline.rs to use step 9
[X] Updated workflow_test.rs to test step 9 as a pipeline step
[X] Removed direct call to generate_github_issues_plan in tests

### Next Steps
[ ] Run tests to verify the changes work correctly
[ ] Consider adding more comprehensive tests for step 9

## Lessons
- Using mockall for creating mock implementations of traits and types
- Using tempfile for creating temporary test directories
- Creating module structure to improve testability
- Pipeline step 2 should check both previous_file location and docs directory for r01_initial_ideation.txt
- Template context variables should match between pipeline and template files
- Implementing functionality as pipeline steps improves mental model and testability
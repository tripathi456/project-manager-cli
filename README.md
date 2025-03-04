# DocGen - AI-Powered Project Documentation Generator

DocGen is a command-line tool that leverages Large Language Models (LLMs) to automate the generation of comprehensive project documentation. It follows a structured, step-by-step approach to create various documentation artifacts, from initial ideation to test-driven development plans and GitHub issues.

## Features

- **Step-by-Step Documentation Pipeline**: Generate documentation in a logical sequence, with each step building on the previous one.
- **LLM Integration**: Currently supports Google's Gemini API, with an extensible architecture for adding more providers.
- **Embedded Templates**: Includes predefined prompt templates for consistent documentation generation.
- **Custom Templates**: Option to use your own prompt templates folder.
- **Structured Output**: All documentation is saved to a `docs` folder for easy access and version control.
- **GitHub Issues Plan**: Automatically generate GitHub issues based on your test-driven development plan.

## Documentation Pipeline

DocGen follows a 9-step documentation pipeline:

1. **Initial Ideation**: Define your project's core concept, features, and goals.
2. **Domain Analysis**: Analyze the problem domain and identify key entities and relationships.
3. **PRD v1**: Generate an initial Product Requirements Document.
4. **PRD v2**: Refine the PRD with additional details and considerations.
5. **Architecture L1**: Create a high-level architecture overview.
6. **Architecture L2**: Develop a detailed architecture with components and interactions.
7. **Architecture Explanation**: Generate a comprehensive explanation of the architecture.
8. **TDD Plan**: Create a test-driven development plan.
9. **GitHub Issues Plan**: Convert the TDD plan into structured GitHub issues.

## Installation

### Prerequisites

- Rust and Cargo (latest stable version)
- API key for supported LLM providers (currently Gemini)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/tripathi456/project-manager-cli/
cd project-manager-cli

# Build the project
cargo build --release

# The binary will be available at target/release/docgen
```

## Usage

### Setting Up API Keys

Set your API key as an environment variable:

```bash
# For Gemini
export GEMINI_API_KEY="your-api-key-here"
```

### Running the Documentation Pipeline

```bash
# Start from step 1 (initial ideation)
docgen run --step 1

# After editing the initial ideation file, continue from step 2
docgen run --step 2

# Run with a specific LLM provider
docgen run --step 2 --provider gemini

# Use custom prompt templates
docgen run --step 2 --prompt-folder /path/to/your/templates
```

### Output

All documentation is saved to a `docs` folder in your current directory, with files named according to the step:

- `r01_initial_ideation.md`
- `r02_domain_analysis.md`
- `r03_prd_v1.md`
- `r04_prd_v2.md`
- `r05_arch_L1.md`
- `r06_arch_L2.md`
- `r07_explain_architecture.md`
- `r08_tdd_v1.md`
- `step_09_github_issues_plan.md`

## Extending DocGen

### Adding New LLM Providers

1. Implement the `LLMProvider` trait for your provider
2. Add your provider to the `get_llm_provider` factory function

### Creating Custom Templates

Create your own templates following the Jinja2 syntax and use the `--prompt-folder` option to specify your templates directory.

## Logging

DocGen uses structured logging with output to both console and a daily-rotated log file (`app.log`).

## License

[MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
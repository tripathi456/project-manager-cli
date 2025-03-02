use std::error::Error;
use std::path::Path;
use tera::Tera;
use tracing::info;

/// Loads Tera templates from the specified directory
/// If no directory is provided, it uses the default "./prompt_templates/"
pub fn load_templates<P: AsRef<Path>>(
    templates_dir: Option<P>
) -> Result<Tera, Box<dyn Error>> {
    let templates_pattern = match templates_dir {
        Some(dir) => {
            let dir_path = dir.as_ref();
            let pattern = dir_path.join("**/*.jinja").to_string_lossy().to_string();
            info!("Loading templates from pattern: {}", pattern);
            pattern
        },
        None => {
            let pattern = "./prompt_templates/**/*.jinja".to_string();
            info!("Loading templates from default pattern: {}", pattern);
            pattern
        }
    };
    
    match Tera::new(&templates_pattern) {
        Ok(tera) => {
            info!("Successfully loaded templates");
            Ok(tera)
        },
        Err(e) => {
            Err(format!("Failed to load templates: {}", e).into())
        }
    }
}

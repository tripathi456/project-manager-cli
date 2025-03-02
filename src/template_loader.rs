use std::error::Error;
use std::fs;
use std::path::Path;
use minijinja::Environment;
use tracing::info;
use serde_json::Value;

/// Make the trait object-safe by removing generics.
///
/// We accept a `serde_json::Value` for the context. The caller
/// will convert their custom `HashMap` or other data into `Value` first.
pub trait TemplateRenderer {
    fn render_value(
        &self,
        template_name: &str,
        context: &Value
    ) -> Result<String, Box<dyn Error>>;
}

/// A struct for loading and rendering templates using minijinja
/// and implementing `TemplateRenderer` as an object-safe trait.
pub struct TemplateLoader {
    env: Environment<'static>,
}

impl TemplateLoader {
    /// Create a new TemplateLoader from a directory on disk
    pub fn from_directory<P: AsRef<Path>>(templates_dir: P) -> Result<Self, Box<dyn Error>> {
        let mut env = Environment::new();
        let templates_dir = templates_dir.as_ref();
        info!("Loading templates from directory: {}", templates_dir.display());

        for entry in fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path)?;
                env.add_template_owned(name.clone(), content)?;
                info!("Added template: {}", name);
            }
        }
        Ok(Self { env })
    }

    /// Create a new TemplateLoader from embedded templates.
    /// Each template is `(name, content)`.
    pub fn from_embedded(templates: &[(String, String)]) -> Result<Self, Box<dyn Error>> {
        let mut env = Environment::new();
        for (name, content) in templates {
            env.add_template_owned(name.clone(), content.clone())?;
            info!("Added embedded template: {}", name);
        }
        Ok(Self { env })
    }
}

/// Implement the now object-safe trait.
impl TemplateRenderer for TemplateLoader {
    fn render_value(
        &self,
        template_name: &str,
        context: &Value
    ) -> Result<String, Box<dyn Error>> {
        let template = self.env.get_template(template_name)?;
        let rendered = template.render(context)?;
        Ok(rendered)
    }
}

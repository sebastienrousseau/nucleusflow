//! # Template Rendering Module
//!
//! Provides flexible template rendering capabilities with support for various template engines.
//! The module offers a pluggable architecture for template rendering with built-in support
//! for Handlebars templates.
//!
//! ## Features
//!
//! - Pluggable template engine architecture
//! - Built-in Handlebars support with helpers
//! - Template caching and validation
//! - Partial template support
//! - Custom helper registration

use crate::{NucleusFlowError, Result, TemplateRenderer};
use handlebars::{
    Context, Handlebars, Helper, Output, RenderContext, RenderError,
    RenderErrorReason,
};
use parking_lot::RwLock;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::convert::From;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Represents a custom template helper with helper name and execution.
pub trait TemplateHelper: Send + Sync {
    /// Executes the helper with the given parameters and context.
    fn execute(
        &self,
        params: &[JsonValue],
        context: &JsonValue,
    ) -> Result<JsonValue>;

    /// Returns the name of the helper for registration.
    fn name(&self) -> &str;
}

/// Provides details for template validation errors.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message detailing the validation issue
    pub message: String,
    /// Line number where error occurred, if available
    pub line: Option<usize>,
    /// Column number where error occurred, if available
    pub column: Option<usize>,
    /// Template source snippet where error occurred
    pub source: Option<String>,
}

impl From<ValidationError> for NucleusFlowError {
    fn from(error: ValidationError) -> Self {
        NucleusFlowError::TemplateRenderingError {
            message: error.message,
            template: String::new(),
            source: None,
        }
    }
}

/// Renderer for Handlebars templates with caching and custom helpers.
#[derive(Clone)]
pub struct HandlebarsRenderer {
    engine: Arc<RwLock<Handlebars<'static>>>, // Handlebars engine
    template_dir: PathBuf,                    // Directory for templates
    template_cache: Arc<RwLock<HashMap<String, String>>>, // Cache for loaded templates
    helpers: Arc<RwLock<HashMap<String, Box<dyn TemplateHelper>>>>, // Custom registered helpers
    strict_mode: bool, // Flag for strict mode
}

impl std::fmt::Debug for HandlebarsRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlebarsRenderer")
            .field("template_dir", &self.template_dir)
            .field("strict_mode", &self.strict_mode)
            .finish()
    }
}

impl HandlebarsRenderer {
    /// Creates a new instance of `HandlebarsRenderer`.
    pub fn new(template_dir: &Path) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_dev_mode(cfg!(debug_assertions));
        handlebars.register_escape_fn(handlebars::html_escape);

        let mut renderer = Self {
            engine: Arc::new(RwLock::new(handlebars)),
            template_dir: template_dir.to_path_buf(),
            template_cache: Arc::new(RwLock::new(HashMap::new())),
            helpers: Arc::new(RwLock::new(HashMap::new())),
            strict_mode: false,
        };

        renderer =
            renderer.with_helper("uppercase", helpers::UppercaseHelper);
        renderer.load_templates()?;
        Ok(renderer)
    }

    /// Enables or disables strict mode, affecting how missing variables and undefined helpers are handled.
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self.engine.write().set_strict_mode(strict);
        self
    }

    /// Registers a custom helper with the renderer.
    pub fn with_helper<H>(self, name: &str, helper: H) -> Self
    where
        H: TemplateHelper + Clone + 'static,
    {
        _ = self
            .helpers
            .write()
            .insert(name.to_string(), Box::new(helper.clone()));
        self.register_helper(name, helper);
        self
    }

    /// Registers a partial template.
    pub fn with_partial(
        self,
        name: &str,
        template: &str,
    ) -> Result<Self> {
        self.engine
            .write()
            .register_partial(name, template)
            .map_err(|e| NucleusFlowError::TemplateRenderingError {
                message: format!(
                    "Failed to register partial '{}': {}",
                    name, e
                ),
                template: name.to_string(),
                source: Some(Box::new(e)),
            })?;
        Ok(self)
    }

    /// Loads templates from the directory, caching and validating them.
    fn load_templates(&self) -> Result<()> {
        let mut engine = self.engine.write();
        let mut cache = self.template_cache.write();

        for entry in
            std::fs::read_dir(&self.template_dir).map_err(|e| {
                NucleusFlowError::TemplateRenderingError {
                    message: format!(
                        "Failed to read template directory: {}",
                        e
                    ),
                    template: String::new(),
                    source: Some(Box::new(e)),
                }
            })?
        {
            let entry = entry.map_err(|e| {
                NucleusFlowError::TemplateRenderingError {
                    message: format!(
                        "Failed to read directory entry: {}",
                        e
                    ),
                    template: String::new(),
                    source: Some(Box::new(e)),
                }
            })?;
            let path = entry.path();

            if path.is_file()
                && path.extension().and_then(|s| s.to_str())
                    == Some("hbs")
            {
                let template_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| {
                        NucleusFlowError::TemplateRenderingError {
                            message: "Invalid template filename"
                                .to_string(),
                            template: path.display().to_string(),
                            source: None,
                        }
                    })?;

                let template_content = std::fs::read_to_string(&path)
                    .map_err(|e| {
                    NucleusFlowError::TemplateRenderingError {
                        message: format!(
                            "Failed to read template file: {}",
                            e
                        ),
                        template: path.display().to_string(),
                        source: Some(Box::new(e)),
                    }
                })?;

                self.validate_template(&template_content).map_err(
                    |e| NucleusFlowError::TemplateRenderingError {
                        message: format!(
                            "Template validation failed: {}",
                            e
                        ),
                        template: template_name.to_string(),
                        source: None,
                    },
                )?;

                engine
                    .register_template_string(
                        template_name,
                        &template_content,
                    )
                    .map_err(|e| {
                        NucleusFlowError::TemplateRenderingError {
                            message: format!(
                                "Failed to register template: {}",
                                e
                            ),
                            template: template_name.to_string(),
                            source: Some(Box::new(e)),
                        }
                    })?;

                _ = cache.insert(
                    template_name.to_string(),
                    template_content,
                );
            }
        }
        Ok(())
    }

    /// Registers a helper function with the Handlebars engine.
    fn register_helper<H>(&self, name: &str, helper: H)
    where
        H: TemplateHelper + 'static,
    {
        let helper_fn = move |h: &Helper,
                              _: &Handlebars,
                              ctx: &Context,
                              _: &mut RenderContext,
                              out: &mut dyn Output|
              -> std::result::Result<
            (),
            RenderError,
        > {
            let params: Vec<JsonValue> =
                h.params().iter().map(|p| p.value().clone()).collect();

            let result =
                helper.execute(&params, ctx.data()).map_err(|e| {
                    RenderError::from(RenderErrorReason::Other(
                        e.to_string(),
                    ))
                })?;
            out.write(&result.to_string())?;
            Ok(())
        };

        self.engine
            .write()
            .register_helper(name, Box::new(helper_fn));
    }

    /// Validates the template syntax to catch errors early.
    fn validate_template(&self, template: &str) -> Result<()> {
        let engine = self.engine.read();
        _ = engine
            .render_template(template, &JsonValue::Null)
            .map_err(|e| ValidationError {
                message: e.to_string(),
                line: e.line_no,
                column: e.column_no,
                source: Some(template.to_string()),
            })?;

        let mut brackets = Vec::new();
        let mut in_tag = false;

        for (i, c) in template.chars().enumerate() {
            match c {
                '{' if in_tag => brackets.push(('{', i)),
                '}' if brackets.pop().is_none() => {
                    return Err(ValidationError {
                        message: "Unmatched closing brace".to_string(),
                        line: None,
                        column: Some(i),
                        source: Some(template.to_string()),
                    }
                    .into());
                }
                '{' => in_tag = true,
                _ => {}
            }
        }

        if !brackets.is_empty() {
            return Err(ValidationError {
                message: "Unmatched opening brace".to_string(),
                line: None,
                column: Some(brackets[0].1),
                source: Some(template.to_string()),
            }
            .into());
        }

        Ok(())
    }

    /// Validates template context variables in strict mode.
    fn validate_context(
        &self,
        template: &str,
        context: &JsonValue,
    ) -> Result<()> {
        let template_content = self
            .template_cache
            .read()
            .get(template)
            .ok_or_else(|| NucleusFlowError::TemplateRenderingError {
                message: format!(
                    "Template '{}' not found in cache",
                    template
                ),
                template: template.to_string(),
                source: None,
            })?
            .clone();

        let mut required_vars = Vec::new();
        let mut current_var = String::new();
        let mut in_var = false;

        for c in template_content.chars() {
            match c {
                '{' => {
                    current_var.clear();
                    in_var = true;
                }
                '}' if in_var => {
                    required_vars.push(current_var.clone());
                    in_var = false;
                }
                c if in_var => current_var.push(c),
                _ => {}
            }
        }

        for var in required_vars {
            if context.get(&var).is_none() {
                return Err(NucleusFlowError::TemplateRenderingError {
                    message: format!(
                        "Missing required variable '{}'",
                        var
                    ),
                    template: template.to_string(),
                    source: None,
                });
            }
        }

        Ok(())
    }
}

impl TemplateRenderer for HandlebarsRenderer {
    fn render(
        &self,
        template: &str,
        context: &JsonValue,
    ) -> Result<String> {
        if self.strict_mode {
            self.validate_context(template, context)?;
        }

        self.engine.read().render(template, context).map_err(|e| {
            NucleusFlowError::TemplateRenderingError {
                message: format!("Template rendering failed: {}", e),
                template: template.to_string(),
                source: Some(Box::new(e)),
            }
        })
    }

    fn validate(
        &self,
        template: &str,
        context: &JsonValue,
    ) -> Result<()> {
        if !self.template_cache.read().contains_key(template) {
            return Err(NucleusFlowError::TemplateRenderingError {
                message: format!("Template '{}' not found", template),
                template: template.to_string(),
                source: None,
            });
        }

        if self.strict_mode {
            self.validate_context(template, context)?;
        }

        Ok(())
    }
}

/// Built-in helpers for template processing.
pub mod helpers {
    use super::*;

    /// Helper to convert text to uppercase.
    #[derive(Debug, Clone, Copy)]
    pub struct UppercaseHelper;

    impl TemplateHelper for UppercaseHelper {
        fn execute(
            &self,
            params: &[JsonValue],
            _context: &JsonValue,
        ) -> Result<JsonValue> {
            let text = params
                .first()
                .and_then(|p| p.as_str())
                .ok_or_else(|| {
                    NucleusFlowError::TemplateRenderingError {
                    message:
                        "Uppercase helper requires a string parameter"
                            .to_string(),
                    template: String::new(),
                    source: None,
                }
                })?;
            Ok(JsonValue::String(text.to_uppercase()))
        }

        fn name(&self) -> &str {
            "uppercase"
        }
    }
}

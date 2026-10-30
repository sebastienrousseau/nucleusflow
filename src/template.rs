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

use crate::{ProcessingError, Result, TemplateRenderer};
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
    /// Error details detailing the validation issue
    pub details: String,
    /// Line number where error occurred, if available
    pub line: Option<usize>,
    /// Column number where error occurred, if available
    pub column: Option<usize>,
    /// Template source snippet where error occurred
    pub source: Option<String>,
}

impl From<ValidationError> for ProcessingError {
    fn from(error: ValidationError) -> Self {
        ProcessingError::TemplateProcessing {
            details: error.details,
            template_name: String::new(),
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
            .map_err(|e| ProcessingError::TemplateProcessing {
                details: format!(
                    "Failed to register partial '{}': {}",
                    name, e
                ),
                template_name: name.to_string(),
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
                ProcessingError::TemplateProcessing {
                    details: format!(
                        "Failed to read template directory: {}",
                        e
                    ),
                    template_name: String::new(),
                    source: Some(Box::new(e)),
                }
            })?
        {
            let entry = entry.map_err(|e| {
                ProcessingError::TemplateProcessing {
                    details: format!(
                        "Failed to read directory entry: {}",
                        e
                    ),
                    template_name: String::new(),
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
                        ProcessingError::TemplateProcessing {
                            details: "Invalid template filename"
                                .to_string(),
                            template_name: path.display().to_string(),
                            source: None,
                        }
                    })?;

                let template_content = std::fs::read_to_string(&path)
                    .map_err(|e| {
                    ProcessingError::TemplateProcessing {
                        details: format!(
                            "Failed to read template file: {}",
                            e
                        ),
                        template_name: path.display().to_string(),
                        source: Some(Box::new(e)),
                    }
                })?;

                Self::validate_template_internal(
                    &engine,
                    &template_content,
                )
                .map_err(|e| {
                    ProcessingError::TemplateProcessing {
                        details: format!(
                            "Template validation failed: {}",
                            e
                        ),
                        template_name: template_name.to_string(),
                        source: None,
                    }
                })?;

                engine
                    .register_template_string(
                        template_name,
                        &template_content,
                    )
                    .map_err(|e| {
                        ProcessingError::TemplateProcessing {
                            details: format!(
                                "Failed to register template: {}",
                                e
                            ),
                            template_name: template_name.to_string(),
                            source: Some(Box::new(e)),
                        }
                    })?;

                _ = cache.insert(
                    template_name.to_string(),
                    template_content,
                );
            }
        }

        if !cache.contains_key("default") {
            let default_template = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{{#if title}}{{title}}{{else}}NucleusFlow{{/if}}</title>\n</head>\n<body>\n{{{content}}}\n</body>\n</html>";
            engine
                .register_template_string("default", default_template)
                .map_err(|e| ProcessingError::TemplateProcessing {
                    details: format!(
                        "Failed to register default template: {}",
                        e
                    ),
                    template_name: "default".to_string(),
                    source: Some(Box::new(e)),
                })?;
            _ = cache.insert(
                "default".to_string(),
                default_template.to_string(),
            );
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
            match result {
                JsonValue::String(s) => out.write(&s)?,
                other => out.write(&other.to_string())?,
            }
            Ok(())
        };

        self.engine
            .write()
            .register_helper(name, Box::new(helper_fn));
    }

    /// Validates the template syntax to catch errors early.
    pub fn validate_template(&self, template: &str) -> Result<()> {
        let engine = self.engine.read();
        Self::validate_template_internal(&engine, template)
    }

    /// Internal helper validating template syntax against a given Handlebars engine.
    fn validate_template_internal(
        engine: &Handlebars,
        template: &str,
    ) -> Result<()> {
        _ = engine
            .render_template(template, &JsonValue::Null)
            .map_err(|e| ValidationError {
                details: e.to_string(),
                line: e.line_no,
                column: e.column_no,
                source: Some(template.to_string()),
            })?;

        let mut open_brackets = Vec::new();

        for (i, c) in template.chars().enumerate() {
            match c {
                '{' => open_brackets.push(i),
                '}' if open_brackets.pop().is_none() => {
                    return Err(ValidationError {
                        details: "Unmatched closing brace".to_string(),
                        line: None,
                        column: Some(i),
                        source: Some(template.to_string()),
                    }
                    .into());
                }
                _ => {}
            }
        }

        if let Some(&first_open) = open_brackets.first() {
            return Err(ValidationError {
                details: "Unmatched opening brace".to_string(),
                line: None,
                column: Some(first_open),
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
            .ok_or_else(|| ProcessingError::TemplateProcessing {
                details: format!(
                    "Template '{}' not found in cache",
                    template
                ),
                template_name: template.to_string(),
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
                return Err(ProcessingError::TemplateProcessing {
                    details: format!(
                        "Missing required variable '{}'",
                        var
                    ),
                    template_name: template.to_string(),
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
            ProcessingError::TemplateProcessing {
                details: format!("Template rendering failed: {}", e),
                template_name: template.to_string(),
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
            return Err(ProcessingError::TemplateProcessing {
                details: format!("Template '{}' not found", template),
                template_name: template.to_string(),
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
                .ok_or_else(|| ProcessingError::TemplateProcessing {
                    details:
                        "Uppercase helper requires a string parameter"
                            .to_string(),
                    template_name: String::new(),
                    source: None,
                })?;
            Ok(JsonValue::String(text.to_uppercase()))
        }

        fn name(&self) -> &str {
            "uppercase"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validation_error_and_conversion() {
        let err = ValidationError {
            details: "syntax error".to_string(),
            line: Some(10),
            column: Some(5),
            source: Some("{{#if}}".to_string()),
        };
        assert_eq!(err.details, "syntax error");
        assert_eq!(err.line, Some(10));
        assert_eq!(err.column, Some(5));

        let proc_err: ProcessingError = err.into();
        match proc_err {
            ProcessingError::TemplateProcessing { details, .. } => {
                assert_eq!(details, "syntax error");
            }
            _ => panic!("Expected TemplateProcessing error"),
        }
    }

    #[test]
    fn test_handlebars_renderer_debug() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path()).unwrap();
        let debug_str = format!("{:?}", renderer);
        assert!(debug_str.contains("HandlebarsRenderer"));
        assert!(debug_str.contains("strict_mode: false"));
    }

    #[test]
    fn test_load_templates_from_disk() {
        let dir = tempdir().unwrap();
        let post_path = dir.path().join("post.hbs");
        std::fs::write(
            &post_path,
            "<h1>{{title}}</h1><main>{{{content}}}</main>",
        )
        .unwrap();

        // Also create a non-hbs file to test filtering
        let ignore_path = dir.path().join("ignored.txt");
        std::fs::write(&ignore_path, "not a template").unwrap();

        let renderer = HandlebarsRenderer::new(dir.path()).unwrap();
        let mut ctx = serde_json::Map::new();
        let _ = ctx.insert(
            "title".to_string(),
            JsonValue::String("Hello World".to_string()),
        );
        let _ = ctx.insert(
            "content".to_string(),
            JsonValue::String("<p>Body</p>".to_string()),
        );

        let rendered =
            renderer.render("post", &JsonValue::Object(ctx)).unwrap();
        assert_eq!(
            rendered,
            "<h1>Hello World</h1><main><p>Body</p></main>"
        );
    }

    #[test]
    fn test_load_templates_non_existent_dir() {
        let path = PathBuf::from(
            "/non/existent/template/path/nucleusflow_123",
        );
        let result = HandlebarsRenderer::new(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_strict_mode() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path())
            .unwrap()
            .with_strict_mode(true);
        assert!(renderer.strict_mode);
    }

    #[test]
    fn test_with_partial_and_render() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path())
            .unwrap()
            .with_partial("header", "<header>{{site_name}}</header>")
            .unwrap();

        let tpl = "{{> header}}<h1>{{title}}</h1>";
        renderer
            .engine
            .write()
            .register_template_string("page", tpl)
            .unwrap();
        let _ = renderer
            .template_cache
            .write()
            .insert("page".to_string(), tpl.to_string());

        let mut ctx = serde_json::Map::new();
        let _ = ctx.insert(
            "site_name".to_string(),
            JsonValue::String("My Site".to_string()),
        );
        let _ = ctx.insert(
            "title".to_string(),
            JsonValue::String("Home".to_string()),
        );

        let res =
            renderer.render("page", &JsonValue::Object(ctx)).unwrap();
        assert_eq!(res, "<header>My Site</header><h1>Home</h1>");
    }

    #[test]
    fn test_with_partial_invalid_syntax() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path()).unwrap();
        let res = renderer.with_partial("bad", "{{#unclosed}}");
        assert!(res.is_err());
    }

    #[derive(Clone, Copy)]
    struct ReverseHelper;

    impl TemplateHelper for ReverseHelper {
        fn execute(
            &self,
            params: &[JsonValue],
            _ctx: &JsonValue,
        ) -> Result<JsonValue> {
            let s =
                params.first().and_then(|p| p.as_str()).unwrap_or("");
            Ok(JsonValue::String(s.chars().rev().collect()))
        }
        fn name(&self) -> &str {
            "reverse"
        }
    }

    #[test]
    fn test_with_helper_execution() {
        assert_eq!(ReverseHelper.name(), "reverse");
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path())
            .unwrap()
            .with_helper("reverse", ReverseHelper);

        let tpl = "{{reverse word}}";
        renderer
            .engine
            .write()
            .register_template_string("rev", tpl)
            .unwrap();
        let _ = renderer
            .template_cache
            .write()
            .insert("rev".to_string(), tpl.to_string());

        let mut ctx = serde_json::Map::new();
        let _ = ctx.insert(
            "word".to_string(),
            JsonValue::String("Rust".to_string()),
        );
        let res =
            renderer.render("rev", &JsonValue::Object(ctx)).unwrap();
        assert_eq!(res, "tsuR");
    }

    #[test]
    fn test_uppercase_helper() {
        let helper = helpers::UppercaseHelper;
        assert_eq!(helper.name(), "uppercase");

        let res = helper
            .execute(
                &[JsonValue::String("hello".to_string())],
                &JsonValue::Null,
            )
            .unwrap();
        assert_eq!(res, JsonValue::String("HELLO".to_string()));

        let err = helper.execute(&[], &JsonValue::Null);
        assert!(err.is_err());
    }

    #[test]
    fn test_validate_template_syntax() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path()).unwrap();

        assert!(renderer
            .validate_template("<h1>{{title}}</h1>")
            .is_ok());

        // Unmatched opening brace
        let unclosed = renderer.validate_template("<h1>{{title</h1>");
        assert!(unclosed.is_err());

        // Single unmatched opening brace (passes handlebars but caught by bracket validator)
        let single_open =
            renderer.validate_template("<h1>Hello { world</h1>");
        assert!(single_open.is_err());

        // Single unmatched closing brace (passes handlebars but caught by bracket validator)
        let single_close =
            renderer.validate_template("<h1>Hello } world</h1>");
        assert!(single_close.is_err());

        // Unmatched closing brace
        let extra_close =
            renderer.validate_template("<h1>{{title}}}</h1>");
        assert!(extra_close.is_err());

        // Invalid handlebars block
        let bad_block =
            renderer.validate_template("{{#each items}}{{/if}}");
        assert!(bad_block.is_err());
    }

    #[test]
    fn test_strict_mode_validate_and_render() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path())
            .unwrap()
            .with_strict_mode(true);

        let tpl = "<title>{{title}}</title>";
        renderer
            .engine
            .write()
            .register_template_string("strict_doc", tpl)
            .unwrap();
        let _ = renderer
            .template_cache
            .write()
            .insert("strict_doc".to_string(), tpl.to_string());

        let mut valid_ctx = serde_json::Map::new();
        let _ = valid_ctx.insert(
            "title".to_string(),
            JsonValue::String("Hello".to_string()),
        );

        assert!(renderer
            .validate(
                "strict_doc",
                &JsonValue::Object(valid_ctx.clone())
            )
            .is_ok());
        assert!(renderer
            .render("strict_doc", &JsonValue::Object(valid_ctx))
            .is_ok());

        let empty_ctx = serde_json::Map::new();
        assert!(renderer
            .validate(
                "strict_doc",
                &JsonValue::Object(empty_ctx.clone())
            )
            .is_err());
        assert!(renderer
            .render("strict_doc", &JsonValue::Object(empty_ctx))
            .is_err());
    }

    #[test]
    fn test_validate_missing_template() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path()).unwrap();
        let res = renderer.validate("non_existent", &JsonValue::Null);
        assert!(res.is_err());
    }

    #[test]
    fn test_render_error_invalid_expression() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path()).unwrap();
        // Missing template rendering
        let res =
            renderer.render("non_existent_template", &JsonValue::Null);
        assert!(res.is_err());
    }

    #[test]
    fn test_load_templates_invalid_syntax() {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("broken.hbs");
        std::fs::write(&bad_path, "{{#if}}unclosed").unwrap();
        let res = HandlebarsRenderer::new(dir.path());
        assert!(res.is_err());
    }

    #[derive(Clone, Copy)]
    struct NumberHelper;

    impl TemplateHelper for NumberHelper {
        fn execute(
            &self,
            _params: &[JsonValue],
            _ctx: &JsonValue,
        ) -> Result<JsonValue> {
            Ok(JsonValue::from(42))
        }
        fn name(&self) -> &str {
            "num"
        }
    }

    #[derive(Clone, Copy)]
    struct FailingHelper;

    impl TemplateHelper for FailingHelper {
        fn execute(
            &self,
            _params: &[JsonValue],
            _ctx: &JsonValue,
        ) -> Result<JsonValue> {
            Err(ProcessingError::internal_error("helper crashed"))
        }
        fn name(&self) -> &str {
            "fail"
        }
    }

    #[test]
    fn test_helper_variants() {
        assert_eq!(NumberHelper.name(), "num");
        assert_eq!(FailingHelper.name(), "fail");
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path())
            .unwrap()
            .with_helper("num", NumberHelper)
            .with_helper("fail", FailingHelper);

        let tpl = "{{num}}";
        let _ = renderer
            .engine
            .write()
            .register_template_string("num_doc", tpl);
        let _ = renderer
            .template_cache
            .write()
            .insert("num_doc".to_string(), tpl.to_string());
        let res = renderer.render("num_doc", &JsonValue::Null).unwrap();
        assert_eq!(res, "42");

        let fail_tpl = "{{fail}}";
        let _ = renderer
            .engine
            .write()
            .register_template_string("fail_doc", fail_tpl);
        let _ = renderer
            .template_cache
            .write()
            .insert("fail_doc".to_string(), fail_tpl.to_string());
        assert!(renderer.render("fail_doc", &JsonValue::Null).is_err());
    }

    #[test]
    fn test_validate_context_missing_from_cache() {
        let dir = tempdir().unwrap();
        let renderer = HandlebarsRenderer::new(dir.path()).unwrap();
        let res = renderer.validate_context("ghost", &JsonValue::Null);
        assert!(res.is_err());
    }
}

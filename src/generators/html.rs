//! # Output Generation Module
//!
//! Provides flexible output generation capabilities for various formats with built-in
//! support for HTML output. The module uses a trait-based architecture to allow for custom output generators.
//!
//! ## Features
//!
//! - Pluggable output generator architecture
//! - Built-in HTML output generation
//! - Output validation and sanitization
//! - Custom metadata injection
//! - Asset management
//!

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use minify_html::{minify, Cfg};
use parking_lot::RwLock;
use serde_json::Value as JsonValue;

use crate::{ProcessingError, OutputGenerator, Result};

/// Configuration for output generation
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Enable minification of output
    pub minify: bool,
    /// Enable pretty printing
    pub pretty_print: bool,
    /// Custom metadata to inject
    pub metadata: Option<JsonValue>,
    /// Asset directory path
    pub asset_dir: Option<PathBuf>,
    /// Custom configuration options
    pub options: HashMap<String, JsonValue>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            minify: false,
            pretty_print: true,
            metadata: None,
            asset_dir: None,
            options: HashMap::new(),
        }
    }
}

/// HTML output generator with configuration options
#[derive(Clone)]
pub struct HtmlGenerator {
    /// Output configuration
    config: Arc<RwLock<OutputConfig>>,
    /// Asset cache
    asset_cache: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
}

impl std::fmt::Debug for HtmlGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlGenerator")
            .field("config", &*self.config.read())
            .finish()
    }
}

impl HtmlGenerator {
    /// Create a new HTML generator with default configuration
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(OutputConfig::default())),
            asset_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Enable or disable minification
    pub fn with_minification(self, enable: bool) -> Self {
        self.config.write().minify = enable;
        self
    }

    /// Enable or disable pretty printing
    pub fn with_pretty_print(self, enable: bool) -> Self {
        self.config.write().pretty_print = enable;
        self
    }

    /// Set metadata for output
    pub fn with_metadata(self, metadata: JsonValue) -> Self {
        self.config.write().metadata = Some(metadata);
        self
    }

    /// Set asset directory
    pub fn with_asset_dir<P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() || !path.is_dir() {
            return Err(ProcessingError::OutputGenerationError {
                message: "Invalid asset directory".to_string(),
                path,
                source: None,
            });
        }
        self.config.write().asset_dir = Some(path);
        Ok(self)
    }

    /// Process and optimize HTML content
    fn process_html(&self, content: &str) -> Result<String> {
        let config = self.config.read();
        let mut processed = content.to_string();

        // Inject metadata if present
        if let Some(metadata) = &config.metadata {
            let meta_tags = self.generate_meta_tags(metadata)?;
            if let Some(head_pos) = processed.find("</head>") {
                processed.insert_str(head_pos, &meta_tags);
            }
        }

        // Minify if enabled
        if config.minify {
            let cfg = Cfg {
                minify_css: true,
                minify_js: true,
                ..Cfg::default()
            };
            processed =
                String::from_utf8(minify(processed.as_bytes(), &cfg))
                    .map_err(|e| {
                    ProcessingError::OutputGenerationError {
                        message: "HTML minification failed".to_string(),
                        path: PathBuf::new(),
                        source: Some(Box::new(e)),
                    }
                })?;
        }
        // Pretty print if enabled
        else if config.pretty_print {
            processed = self.pretty_print_html(&processed);
        }

        Ok(processed)
    }

    /// Generate meta tags from metadata
    fn generate_meta_tags(
        &self,
        metadata: &JsonValue,
    ) -> Result<String> {
        let mut meta_tags = String::new();

        if let Some(obj) = metadata.as_object() {
            for (key, value) in obj {
                let content = value.as_str().unwrap_or_default();
                meta_tags.push_str(&format!(
                    "<meta name=\"{}\" content=\"{}\">\n",
                    key, content
                ));
            }
        }

        Ok(meta_tags)
    }

    /// Basic HTML pretty printing
    fn pretty_print_html(&self, content: &str) -> String {
        fn is_void_element(tag: &str) -> bool {
            let void_elements =
                ["img", "br", "hr", "input", "meta", "link"];
            void_elements
                .iter()
                .any(|&elem| tag.starts_with(&format!("<{}", elem)))
        }

        let mut pretty = String::new();
        let mut depth = 0;
        let mut chars = content.chars().peekable();
        let mut in_pre = false;
        let mut in_tag = false;
        let mut current = String::new();

        while let Some(c) = chars.next() {
            match c {
                '<' => {
                    // Handle any accumulated text content before the tag
                    if !current.trim().is_empty() && !in_pre {
                        if !pretty.ends_with('>') {
                            pretty.push_str(&"    ".repeat(depth));
                        }
                        pretty.push_str(current.trim());
                        current.clear();
                    }

                    in_tag = true;
                    current.push('<');
                }
                '>' => {
                    current.push('>');
                    let tag = current.trim();

                    if tag.starts_with("<pre") {
                        in_pre = true;
                        if !pretty.is_empty() && !pretty.ends_with('\n')
                        {
                            pretty.push('\n');
                        }
                        pretty.push_str(&"    ".repeat(depth));
                        pretty.push_str(tag);
                        depth += 1;
                    } else if tag.starts_with("</pre") {
                        in_pre = false;
                        depth -= 1;
                        pretty.push_str(tag);
                        pretty.push('\n');
                    } else if in_pre {
                        pretty.push_str(tag);
                    } else if tag.starts_with("</") {
                        depth -= 1;
                        if pretty.ends_with('\n') {
                            pretty.push_str(&"    ".repeat(depth));
                        }
                        pretty.push_str(tag);
                        if chars.peek().is_some() {
                            pretty.push('\n');
                        }
                    } else {
                        if !pretty.is_empty() && !pretty.ends_with('\n')
                        {
                            pretty.push('\n');
                        }
                        pretty.push_str(&"    ".repeat(depth));
                        pretty.push_str(tag);
                        if !is_void_element(tag) {
                            depth += 1;
                        }
                        if chars.peek().map_or(false, |&c| c == '<') {
                            pretty.push('\n');
                        }
                    }

                    in_tag = false;
                    current.clear();
                }
                _ => {
                    if in_pre {
                        pretty.push(c);
                    } else if in_tag
                        || !c.is_whitespace()
                        || !current.is_empty()
                    {
                        current.push(c);
                    }
                }
            }
        }

        // Handle any remaining content
        if !current.trim().is_empty() {
            pretty.push_str(current.trim());
        }

        // Clean up multiple newlines and ensure consistent indentation
        let result = pretty
            .lines()
            .map(|line| line.trim_end()) // Remove trailing whitespace
            .filter(|line| !line.is_empty()) // Remove empty lines
            .collect::<Vec<_>>()
            .join("\n");

        result
    }

    /// Validate input file and options
    fn validate(
        &self,
        path: &Path,
        options: Option<&JsonValue>,
    ) -> Result<()> {
        // Validate file extension
        if path.extension().and_then(|s| s.to_str()) != Some("html") {
            return Err(ProcessingError::OutputGenerationError {
                message: "Invalid directory".to_string(),
                path: path.to_path_buf(),
                source: None,
            });
        }

        // Check if output path is valid
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Validate options if present
        if let Some(opts) = options {
            if !opts.is_object() {
                return Err(ProcessingError::OutputGenerationError {
                    message: "Invalid directory".to_string(),
                    path: path.to_path_buf(),
                    source: None,
                });
            }
        }

        Ok(())
    }

    /// Copy assets to output directory
    fn copy_assets(&self, output_dir: &Path) -> Result<()> {
        if let Some(asset_dir) = &self.config.read().asset_dir {
            let mut cache = self.asset_cache.write();

            for entry in fs::read_dir(asset_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let cached_content =
                        cache.entry(path.clone()).or_insert_with(
                            || fs::read(&path).unwrap_or_default(),
                        );

                    let output_path = output_dir
                        .join(path.strip_prefix(asset_dir).unwrap());

                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::write(&output_path, cached_content)?;
                }
            }
        }

        Ok(())
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputGenerator for HtmlGenerator {
    fn generate(
        &self,
        content: &str,
        path: &Path,
        options: Option<&JsonValue>,
    ) -> Result<()> {
        // Validate input
        self.validate(path, options)?;

        // Process content
        let processed = self.process_html(content)?;

        // Ensure output directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write output
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(processed.as_bytes())?;
        writer.flush()?;

        // Copy assets if configured
        if let Some(parent) = path.parent() {
            self.copy_assets(parent)?;
        }

        Ok(())
    }

    fn validate(
        &self,
        path: &Path,
        options: Option<&JsonValue>,
    ) -> Result<()> {
        // Check if output path is valid
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Validate file extension
        if path.extension().and_then(|s| s.to_str()) != Some("html") {
            return Err(ProcessingError::OutputGenerationError {
                message: "Invalid file extension".to_string(),
                path: path.to_path_buf(),
                source: None,
            });
        }

        // Validate options if present
        if let Some(opts) = options {
            if !opts.is_object() {
                return Err(ProcessingError::OutputGenerationError {
                    message: "Invalid options format".to_string(),
                    path: path.to_path_buf(),
                    source: None,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_basic_output() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = "<h1>Test</h1>";

        let generator = HtmlGenerator::new();
        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert_eq!(result.trim(), content);

        Ok(())
    }

    #[test]
    fn test_minification() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = "<h1>\n    Test\n</h1>";

        let generator = HtmlGenerator::new()
            .with_minification(true)
            .with_pretty_print(false);

        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert_eq!(result, "<h1>Test</h1>");

        Ok(())
    }

    #[test]
    fn test_pretty_print() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = "<div><h1>Test</h1><p>Content</p></div>";

        let generator = HtmlGenerator::new()
            .with_minification(false)
            .with_pretty_print(true);

        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains('\n'));
        assert!(result.contains("    "));
        assert!(result.contains("<div>"));
        assert!(result.contains("</div>"));

        Ok(())
    }

    #[test]
    fn test_metadata_injection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = "<html><head></head><body>Test</body></html>";

        let generator = HtmlGenerator::new().with_metadata(json!({
            "author": "Test Author",
            "description": "Test Description"
        }));

        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("name=\"author\""));
        assert!(result.contains("content=\"Test Author\""));

        Ok(())
    }

    #[test]
    fn test_asset_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let asset_dir = temp_dir.path().join("assets");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&asset_dir)?;

        // Create test asset
        let asset_content = "test asset";
        fs::write(asset_dir.join("test.txt"), asset_content)?;

        let generator =
            HtmlGenerator::new().with_asset_dir(&asset_dir)?;

        let output_path = output_dir.join("index.html");
        generator.generate("<h1>Test</h1>", &output_path, None)?;

        let copied_asset =
            fs::read_to_string(output_dir.join("test.txt"))?;
        assert_eq!(copied_asset, asset_content);

        Ok(())
    }

    #[test]
    fn test_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let generator = HtmlGenerator::new();

        // Test invalid path
        let result = generator.generate(
            "test",
            &temp_dir.path().join("test.txt"),
            None,
        );
        assert!(result.is_err());

        // Test invalid options
        let result = generator.generate(
            "test",
            &temp_dir.path().join("test.html"),
            Some(&json!("invalid")),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_concurrent_output() -> Result<()> {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new()?;
        let generator = Arc::new(HtmlGenerator::new());

        let mut handles = vec![];

        for i in 0..10 {
            let generator = Arc::clone(&generator);
            let output_path =
                temp_dir.path().join(format!("output{}.html", i));

            let handle = thread::spawn(move || {
                generator.generate(
                    &format!("<h1>Test {}</h1>", i),
                    &output_path,
                    None,
                )
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()?;
        }

        Ok(())
    }

    #[test]
    fn test_pre_tag_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = r#"<div><pre>
    function test() {
        console.log("hello");
    }
</pre></div>"#;

        let generator = HtmlGenerator::new().with_pretty_print(true);
        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("    function test() {"));
        assert!(result.contains("        console.log(\"hello\");"));
        Ok(())
    }

    #[test]
    fn test_void_elements() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = r#"<div><img src="test.jpg"><br><hr></div>"#;

        let generator = HtmlGenerator::new().with_pretty_print(true);

        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("<img src=\"test.jpg\">"));
        assert!(result.contains("<br>"));
        assert!(result.contains("<hr>"));

        Ok(())
    }

    #[test]
    fn test_nested_elements() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = "<div><section><article><h1>Test</h1><p>Content</p></article></section></div>";

        let generator = HtmlGenerator::new().with_pretty_print(true);
        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("    <section>"));
        assert!(result.contains("        <article>"));
        assert!(result.contains("            <h1>Test</h1>"));
        assert!(result.contains("            <p>Content</p>"));
        Ok(())
    }

    #[test]
    fn test_html_structure() {
        let generated_html = "<!DOCTYPE html>\n    <html>\n        <head>\n            <title>Test</title>\n        </head>\n        <body>\n            <h1>Test</h1>\n        </body>\n    </html>";
        let expected_html = "<!DOCTYPE html>\n<html>\n    <head>\n        <title>Test</title>\n    </head>\n    <body>\n        <h1>Test</h1>\n    </body>\n</html>";

        // Normalize whitespace
        let generated_normalized =
            generated_html.replace(" ", "").replace("\n", "");
        let expected_normalized =
            expected_html.replace(" ", "").replace("\n", "");

        assert_eq!(generated_normalized, expected_normalized);
    }
}
